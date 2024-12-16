#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use core::mem::replace;
use pi_null::Null;
use pi_slotmap::DefaultKey;

use crate::geometry::*;
use crate::node_state::*;
use crate::number::*;
use crate::style::*;
use crate::traits::*;
// use crate::grow_shrink::*;
use pi_heap::simple_heap::SimpleHeap;
use std::cmp::Ordering;

#[allow(dead_code)]
pub fn ppp() -> String {
    let mut s = String::from("");
    for _ in 0..unsafe { PC } {
        s.push_str("--");
    }
    for _ in 0..unsafe { PP } {
        s.push_str("**");
    }
    s
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharNode {
    // 字符
    pub ch: char,
    // margin
    // pub margin: Rect<Dimension>,
    // 字符大小
    pub size: Size<Dimension>,
    // 位置
    pub pos: Rect<f32>,
    // 字符id或单词的字符数量 ch==char::from(0)时，表示单词容器节点，此时ch_id_or_count表示该节点中的字符数量
    pub count: usize,
    pub ch_id: DefaultKey,
    // 字符在整个节点中的索引
    pub char_i: isize,
    // 如果是多字符文字中的某个字符，则存在一个上下文索引
    pub context_id: isize,
}

impl Default for CharNode {
    fn default() -> Self {
        CharNode {
            ch: char::from(0),
            size: Size {
                width: Dimension::Points(0.0),
                height: Dimension::Points(0.0),
            },
            pos: Rect {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
            count: 0,
            ch_id: DefaultKey::null(),
            char_i: -1,
            context_id: -1,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct INode {
    pub(crate) state: NodeState,
    // 文字节点
    pub text: Vec<CharNode>,
    // 如果是图文混排，代表在Vec<CharNode>中的位置
    pub char_index: usize,
    // 文字布局的缩放值， 放到其它地方去？TODO
    pub scale: f32,
    // 是否进行简单布局
    pub is_sample: bool,
}

impl INode {
    pub fn new(value: NodeState, char_index: usize) -> Self {
        INode {
            state: value | NodeState::ChildrenIndex,
            text: Vec::new(),
            char_index: char_index,
            scale: 1.0,
            is_sample: false,
        }
    }
}

impl Default for INode {
    fn default() -> Self {
        INode {
            state: NodeState::default() | NodeState::RNode,
            text: Vec::new(),
            char_index: 0,
            scale: 1.0,
            is_sample: false,
        }
    }
}

impl INode {
    pub fn is_vnode(&self) -> bool {
        self.state.contains(NodeState::VNode)
    }

    // 是否为真实节点
    pub fn is_rnode(&self) -> bool {
        self.state.contains(NodeState::RNode)
    }

    pub fn set_vnode(&mut self, b: bool) {
        if b {
            self.state |= NodeState::VNode;
        } else {
            self.state &= !NodeState::VNode;
        }
    }

    pub fn set_rnode(&mut self, b: bool) {
        if b {
            self.state |= NodeState::RNode;
        } else {
            self.state &= !NodeState::RNode;
        }
    }

    pub fn set_line_start_margin_zero(&mut self, b: bool) {
        if b {
            self.state |= NodeState::LineStartMarginZero;
        } else {
            self.state &= !NodeState::LineStartMarginZero;
        }
    }
    pub fn set_breakline(&mut self, b: bool) {
        if b {
            self.state |= NodeState::BreakLine;
        } else {
            self.state &= !NodeState::BreakLine;
        }
    }
}

// 计算时使用的上下文
pub struct CalcContext<K> {
    pub border_gap_size: Size<f32>,
    pub padding_gap_size: Size<f32>,
    // 布局容器的 最小大小
    pub min_size: Size<f32>,
    // 主轴的大小, 用于约束换行，该值需要参考节点设置的width或height，以及max_width或max_height, 如果都未设置，则该值为无穷大
    pub main: Number,
    // 交叉轴的大小
    pub cross: Number,
    // 主轴的大小, 用于判断是否折行
    pub main_line: f32,
    // 主轴的像素大小，该值需要参考width或height，以及min_width或min_height，用于子节点未将该节点撑得更大时，节点的主轴布局结果
    pub main_value: f32,
    // 交叉轴的像素大小，该值需要参考width或height，以及min_width或min_height，用于子节点未将该节点撑得更大时，节点的交叉轴布局结果
    pub cross_value: f32,
    // 统计子节点的 ChildrenAbs ChildrenNoAlignSelf ChildrenIndex 的数量
    pub state: NodeState,
    // 如果需要排序，则使用堆排序
    pub heap: SimpleHeap<OrderSort<K>>,
    /// 缓存的子节点数组
    pub temp: TempNode<K>,
    pub vnode: Vec<K>,
}

impl<K> CalcContext<K> {
    pub fn new(
        border_gap_size: Size<f32>,
        padding_gap_size: Size<f32>,
        flex: ContainerStyle,
        size: Size<Number>,
        min_size: Size<Number>,
        max_size: Size<Number>,
    ) -> Self {
        // 计算主轴和交叉轴，及大小
        let row = flex.flex_direction.is_row();
        let gap_size = border_gap_size + padding_gap_size;
        let (main, cross, max_main, min_main, min_cross, gap) = if row {
            (
                size.width,
                size.height,
                max_size.width,
                min_size.width,
                min_size.height,
                gap_size,
            )
        } else {
            (
                size.height,
                size.width,
                max_size.height,
                min_size.height,
                min_size.width,
                Size::new(gap_size.height, gap_size.width),
            )
        };
        let m = if flex.flex_wrap == FlexWrap::NoWrap {
            std::f32::INFINITY
        } else {
            max_calc(main, max_main).or_else(std::f32::INFINITY)
        };
        unsafe { PP += 1 };
        CalcContext {
            border_gap_size,
            padding_gap_size,
            min_size: Size::new(
                gap_size.width.max(min_size.width.or_else(0.0)),
                gap_size.height.max(min_size.height.or_else(0.0)),
            ),
            main,
            cross,
            main_line: 0.0f32.max(m - gap.width),
            main_value: 0.0f32.max(main.or_else(min_main.or_else(0.0)) - gap.width),
            cross_value: 0.0f32.max(cross.or_else(min_cross.or_else(0.0)) - gap.height),
            state: NodeState::default(),
            heap: SimpleHeap::new(Ordering::Less),
            temp: TempNode::<K>::new(flex, row),
            vnode: Vec::new(),
        }
    }
}

impl<K: Null + Clone> CalcContext<K> {
    // 文字的flex布局
    pub fn text_layout(
        &mut self,
        id: K,
        text: &mut Vec<CharNode>,
        line: &mut LineInfo,
        overflow_wrap: OverflowWrap,
    ) {
        out_any!(
            println,
            // log::trace,
            "text_layout, id:{:?}, text: {:?}",
            &id,
            &text
        );
        let is_overflow_wrap =
            overflow_wrap == OverflowWrap::Anywhere || overflow_wrap == OverflowWrap::BreakWord;
        let mut char_index = 0;
        let len = text.len();
        while char_index < len {
            let char_node = &mut text[char_index];

            // 如果是单词容器节点， 并且单词字符可以换行， 则需要将单词的每字符进行布局， 单词容器的位置设置为0(容器不再继续参与布局)
            if char_node.ch == char::from(0) && is_overflow_wrap {
                char_node.pos = Rect {
                    left: 0.0,
                    right: char_node.pos.right - char_node.pos.left,
                    top: 0.0,
                    bottom: char_node.pos.bottom - char_node.pos.top,
                };
                char_index += 1;
                continue;
            }

            let (main_d, cross_d) = self.temp.main_cross(
                if let Dimension::Points(r) = char_node.size.width {
                    r
                } else {
                    panic!("")
                },
                if let Dimension::Points(r) = char_node.size.height {
                    r
                } else {
                    panic!("")
                },
            );
            let mut info = RelNodeInfo {
                id: id.clone(),
                grow: 0.0,
                shrink: 0.0,
                main: main_d,
                cross: cross_d,
                margin_main: 0.0,
                margin_main_start: Number::default(),
                margin_main_end: Number::default(),
                margin_cross_start: Number::default(),
                margin_cross_end: Number::default(),
                align_self: AlignSelf::Auto,
                main_d: Dimension::Points(main_d),
                cross_d: Dimension::Points(cross_d),
                line_start_margin_zero: true,
                breakline: char_node.ch == char::from('\n'),
                min_main: Number::Undefined,
                max_main: Number::Undefined,
                main_result: 0.0,
                main_result_maybe_ok: false,
            };
            let start = info.margin_main_start.or_else(0.0);
            let end = info.margin_main_end.or_else(0.0);
            // 主轴auto时记录子节点实际大
            let line_start = if line.item.count == 0 {
                // 处理行首
                0.0
            } else {
                start
            };
            info.margin_main = start + end;
            line.main += info.main + line_start + end;

            self.add_vec(line, 0, info, TempNodeType::CharIndex(char_index));
            // 判断是否为单词容器
            if char_node.ch == char::from(0) {
                char_index += char_node.count;
            } else {
                char_index += 1;
            }
        }
    }
    // 添加到数组中，计算当前行的grow shrink 是否折行及折几行
    pub fn add_vec(
        &mut self,
        line: &mut LineInfo,
        _order: isize,
        info: RelNodeInfo<K>,
        temp: TempNodeType<K>,
    ) {
        //out_any!(log::trace, "add info:{:?}", info);
        line.add(self.main_line, &info);
        self.temp.rel_vec.push((info, temp));
    }
    // 添加到堆中
    pub fn add_heap(
        &mut self,
        line: &mut LineInfo,
        order: isize,
        info: RelNodeInfo<K>,
        temp: TempNodeType<K>,
    ) {
        line.add(self.main_line, &info);
        self.heap
            .push(OrderSort(order, self.heap.len(), info, temp));
    }
}

/// 临时节点类型
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum TempNodeType<K> {
    /// 固定大小节点
    None,
    /// Auto大小并已经计算完的节点
    AutoOk,
    /// Auto节点的临时节点信息及子节点数组
    R(TempNode<K>),
    /// 字符索引
    CharIndex(usize),
}
impl<K> Default for TempNodeType<K> {
    fn default() -> Self {
        TempNodeType::None
    }
}
// 排序节点
#[derive(Default, Clone, Debug)]
pub struct OrderSort<K>(
    pub isize,
    pub usize,
    pub RelNodeInfo<K>,
    pub TempNodeType<K>,
); // (order, index, Info, temp)
impl<K> Ord for OrderSort<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 > other.0 {
            Ordering::Greater
        } else if self.0 < other.0 {
            Ordering::Less
        } else if self.1 > other.1 {
            Ordering::Greater
        } else if self.1 < other.1 {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl<K> PartialOrd for OrderSort<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 > other.0 {
            Some(Ordering::Greater)
        } else if self.0 < other.0 {
            Some(Ordering::Less)
        } else if self.1 > other.1 {
            Some(Ordering::Greater)
        } else if self.1 < other.1 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl<K> PartialEq for OrderSort<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl<K> Eq for OrderSort<K> {}

/// 临时缓存节点的样式、大小和子节点数组
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct TempNode<K> {
    pub flex: ContainerStyle,
    pub abs_vec: Vec<(K, K, K, NodeState, bool)>, // (id, children_head, children_tail, state, is_text) 绝对定位的子节点数组
    pub rel_vec: Vec<(RelNodeInfo<K>, TempNodeType<K>)>, // 相对定位的子节点数组
    pub row: bool,
    pub children_percent: bool, // 子节点是否有百分比宽高
}

impl<K> Default for TempNode<K> {
    fn default() -> Self {
        Self {
            flex: ContainerStyle::default(),
            abs_vec: Vec::new(), // (id, children_head, children_tail, state, is_text) 绝对定位的子节点数组
            rel_vec: Vec::new(), // 相对定位的子节点数组
            row: Default::default(),
            children_percent: false,
        }
    }
}

impl<K> TempNode<K> {
    fn new(flex: ContainerStyle, row: bool) -> Self {
        TempNode {
            flex,
            row,
            abs_vec: Vec::new(),
            rel_vec: Vec::new(),
            children_percent: false,
        }
    }
    pub fn main_cross<T>(&self, w: T, h: T) -> (T, T) {
        if self.row {
            (w, h)
        } else {
            (h, w)
        }
    }

    // 用缓存的相对定位的子节点数组重建行信息
    pub fn reline(&mut self, main: f32, cross: f32) -> LineInfo {
        let mut line = LineInfo::default();
        if self.children_percent {
            for r in self.rel_vec.iter_mut() {
                // 修正百分比的大小
                if let Dimension::Percent(rr) = r.0.main_d {
                    r.0.main = main * rr;
                }

                // 修正百分比的大小
                if let Dimension::Percent(rr) = r.0.cross_d {
                    r.0.cross = cross * rr;
                }
                line.add(main, &r.0);
            }
        } else {
            for r in self.rel_vec.iter() {
                line.add(main, &r.0);
            }
        }
        unsafe { PP += 1 };
        out_any!(
            println,
            // log::trace,
            "{:?}reline: line:{:?}",
            ppp(),
            &line
        );
        line
    }

    // 多行的区间计算
    pub fn multi_calc(&self, cross: f32, split: f32, pos: &mut f32) -> (f32, f32) {
        let start = *pos;
        if self.flex.flex_wrap != FlexWrap::WrapReverse {
            let end = *pos + cross;
            *pos = end + split;
            (start, end)
        } else {
            let end = *pos - cross;
            *pos = end - split;
            (end, start)
        }
    }
}

/// 相对定位下缓存的节点信息
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub struct RelNodeInfo<K> {
    pub id: K,
    // 节点交叉轴尺寸， 与父的flex方向有关
    pub cross: f32,
    // 节点主轴方向 margin_start margin_end的大小
    pub margin_main: f32,
    // 节点主轴方向 margin_start的大小
    pub margin_main_start: Number,
    // 节点主轴方向 margin_end的大小
    pub margin_main_end: Number,
    // 节点交叉轴方向 margin_start的大小
    pub margin_cross_start: Number,
    // 节点交叉轴方向 margin_end的大小
    pub margin_cross_end: Number,
    // 节点的align_self
    pub align_self: AlignSelf,
    // 节点主轴大小
    pub main_d: Dimension,
    // 节点交叉轴大小
    pub cross_d: Dimension,
    // 如果该元素为行首，则margin_start为0
    pub line_start_margin_zero: bool,
    // 强制换行
    pub breakline: bool,
    // 节点grow的值
    pub(crate) grow: f32,
    // 节点shrink的值
    pub(crate) shrink: f32,
    // 节点主轴尺寸(受basis影响), 与父的flex方向有关
    pub(crate) main: f32,
    //主轴最小尺寸
    pub(crate) min_main: Number,
    // 主轴最大尺寸
    pub(crate) max_main: Number,
    // 主轴的计算结果
    pub(crate) main_result: f32,
    // 主轴的计算结果是否有效
    pub(crate) main_result_maybe_ok: bool,
}

/// 计算时统计的行信息
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub struct LineInfo {
    pub main: f32,            // 行内节点主轴尺寸的总值，不受basis影响
    pub cross: f32,           // 多行子节点交叉轴的像素的累计值
    pub item: LineItem,       // 当前计算的行margin_auto
    pub items: Vec<LineItem>, // 已计算的行
}
/// 行信息中每行条目
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub struct LineItem {
    pub count: usize,       // 行内节点总数量
    pub grow: f32,          // 行内节点grow的总值
    pub shrink: f32,        // 行内节点shrink的总值
    pub margin_auto: usize, // 行内节点主轴方向 margin=auto 的数量
    pub main: f32,          // 行内节点主轴尺寸的总值（包括size margin）
    pub cross: f32,         // 行内节点交叉轴尺寸的最大值
                            // grow_shrink_context: GrowShrinkContext, // 行内节点grow shrink的上下文
}

impl LineItem {
    // 将节点信息统计到行条目上
    fn merge<K>(&mut self, info: &RelNodeInfo<K>, line_start: bool) {
        self.count += 1;
        self.grow += info.grow;
        self.shrink += info.shrink;
        self.main += info.main;
        let mut cross = info.cross;
        if let Number::Defined(r) = info.margin_main_end {
            self.main += r;
        } else {
            self.margin_auto += 1;
        }

        if let Number::Defined(r) = info.margin_cross_start {
            cross += r;
        }

        if let Number::Defined(r) = info.margin_cross_end {
            cross += r;
        }

        if self.cross < cross {
            self.cross = cross;
        }
        if line_start && info.line_start_margin_zero {
            return;
        }
        if let Number::Defined(r) = info.margin_main_start {
            self.main += r;
        } else {
            self.margin_auto += 1;
        }
    }
}

impl LineInfo {
    // 添加到数组中，计算当前行的grow shrink 是否折行及折几行
    fn add<K>(&mut self, main: f32, info: &RelNodeInfo<K>) {
        out_any!(
            log::trace,
            "add, main: {:?}, {:?}, self.item: {:?}",
            main,
            info,
            &self.item
        );
        // 浮点误差判断是否折行
        if (self.item.count > 0 && self.item.main + info.main + info.margin_main - main > EPSILON)
            || info.breakline
        {
            self.cross += self.item.cross;
            out_any!(
                println,
                // log::trace,
                "breakline, self.cross:{:?}, self.item.cross: {:?}",
                self.cross,
                self.item.cross
            );
            let t = replace(&mut self.item, LineItem::default());
            self.items.push(t);
            self.item.merge(info, true);
        } else {
            self.item.merge(info, self.item.count == 0);
        }
    }
}

/// 获得相对定位节点对应的包含块containing block的大小及位置， 由内容区（content box）的边缘组成
pub(crate) fn rel_containing_block_size<T: LayoutR>(l: &T) -> Size<f32> {
    Size::new(
        l.rect().right
            - l.border().right
            - l.padding().right
            - l.rect().left
            - l.border().left
            - l.padding().left,
        l.rect().bottom
            - l.border().bottom
            - l.padding().bottom
            - l.rect().top
            - l.border().top
            - l.padding().top,
    )
}
/// https://developer.mozilla.org/zh-CN/docs/Web/CSS/Containing_block
/// 获得节点对应的包含块containing block，绝对定位节点由父内边距区（padding box）的边缘组成， 相对定位节点由父内容区（content box）的边缘组成
pub(crate) fn abs_containing_block_size<T: LayoutR>(l: &T) -> Size<f32> {
    Size::new(
        l.rect().right - l.border().right - l.rect().left - l.border().left,
        l.rect().bottom - l.border().bottom - l.rect().top - l.border().top,
    )
}

// 设置布局结果，返回是否变动两种内容区大小
pub fn set_layout_result<T, K, L: LayoutR>(
    layout: &mut L,
    notify: fn(&mut T, K, &L),
    notify_arg: &mut T,
    id: K,
    containing_block_size: Size<f32>,
    is_abs: bool,
    rect: Rect<f32>,
    border: &SideGap<Dimension>,
    padding: &SideGap<Dimension>,
) -> bool {
    unsafe {
        PC += 1;
        PP = 0
    };
    let old_size = layout.rect().size();
    let old_padding_box_size = old_size - layout.border().gap_size();
    let old_content_box_size = old_padding_box_size - layout.padding().gap_size();
    layout.set_absolute(is_abs);
    layout.set_rect(rect);
    layout.set_border(calc_gap_by_containing_block(&containing_block_size, border));
    layout.set_padding(calc_gap_by_containing_block(
        &containing_block_size,
        padding,
    ));
    notify(notify_arg, id, layout);
    layout.set_finish();
    let size = layout.rect().size();
    let padding_box_size = size - layout.border().gap_size();
    if !(eq_f32(padding_box_size.width, old_padding_box_size.width)
        && eq_f32(padding_box_size.height, old_padding_box_size.height))
    {
        return true;
    }
    let content_box_size = padding_box_size - layout.padding().gap_size();
    !(eq_f32(content_box_size.width, old_content_box_size.width)
        && eq_f32(content_box_size.height, old_content_box_size.height))
}

pub const EPSILON: f32 = std::f32::EPSILON * 1024.0;
#[inline]
pub fn eq_f32(v1: f32, v2: f32) -> bool {
    v1 == v2 || ((v2 - v1).abs() <= EPSILON)
}

// 节点的兄弟节点
pub fn node_iter<K: Null + Copy + Clone>(direction: Direction, next: K, prev: K) -> K {
    if direction != Direction::RTL {
        next
    } else {
        // 处理倒排的情况
        prev
    }
}

pub fn grow_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main + info.grow * per;
    // if let Number::Defined(r) = info.max_main {
    // 	size = size.min(r);
    // }
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + size + info.margin_main_end.or_else(0.0);
    (start, size)
}
pub fn grow_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main + info.grow * per;
    // if let Number::Defined(r) = info.max_main {
    // 	size = size.min(r);
    // }
    let start = *pos - info.margin_main_end.or_else(0.0) - size;
    *pos = start - info.margin_main_start.or_else(0.0);
    (start, size)
}
pub fn margin_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos + info.margin_main_start.or_else(per);
    *pos = start + info.main + info.margin_main_end.or_else(per);
    (start, info.main)
}
pub fn margin_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos - info.margin_main_end.or_else(per) - info.main;
    *pos = start - info.margin_main_end.or_else(per);
    (start, info.main)
}
pub fn shrink_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main - info.shrink as f32 * per;
    // if let Number::Defined(r) = info.min_main {
    // 	size = size.max(r);
    // }
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + size + info.margin_main_end.or_else(0.0);
    (start, size)
}
pub fn shrink_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main - info.shrink as f32 * per;
    // if let Number::Defined(r) = info.min_main {
    // 	size = size.max(r);
    // }
    let start = *pos - info.margin_main_end.or_else(0.0) - size;
    *pos = start - info.margin_main_start.or_else(0.0);
    (start, size)
}

pub fn min_max_calc(mut value: f32, min_value: Number, max_value: Number) -> f32 {
    if let Number::Defined(r) = min_value {
        value = value.max(r);
    }
    if let Number::Defined(r) = max_value {
        value = value.min(r);
    }
    value
}

pub fn max_calc(value: Number, max_value: Number) -> Number {
    if let (Number::Undefined, Number::Defined(_r)) = (value, max_value) {
        max_value
    } else {
        value
    }
}

pub fn main_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + info.main + info.margin_main_end.or_else(0.0) + per;
    (start, info.main)
}
pub fn main_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos - info.margin_main_end.or_else(0.0) - info.main;
    *pos = start - info.margin_main_start.or_else(0.0) - per;
    (start, info.main)
}
// 返回位置和大小
pub fn cross_calc<K>(
    info: &RelNodeInfo<K>,
    start: f32,
    end: f32,
    align_items: AlignItems,
    baseline: &mut Number,
) -> (f32, f32) {
    out_any!(
        println,
        // log::trace,
        "{:?}cross_calc, start:{:?}, end:{:?}, info:{:?}",
        ppp(),
        start,
        end,
        info
    );

    match info.align_self {
        AlignSelf::Auto => match align_items {
            AlignItems::FlexStart => align_start(start, end, info),
            AlignItems::FlexEnd => align_end(start, end, info),
            AlignItems::Center => align_center(start, end, info),
            _ if info.cross_d.is_undefined() => align_stretch(start, end, info),
            _ => align_baseline(start, end, info, baseline), // 不算完全支持baseline
        },
        AlignSelf::FlexStart => align_start(start, end, info),
        AlignSelf::FlexEnd => align_end(start, end, info),
        AlignSelf::Center => align_center(start, end, info),
        _ if info.cross_d.is_undefined() => align_stretch(start, end, info),
        _ => align_baseline(start, end, info, baseline), // 不算完全支持baseline
    }
}
// 返回位置和大小
pub fn align_start<K>(start: f32, _end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
    (start + info.margin_cross_start.or_else(0.0), info.cross)
    // if let Number::Defined(r) = info.margin_cross_start {
    //     (start + r, info.cross)
    // } else if let Number::Defined(r) = info.margin_cross_end {
    //     (end - r - info.cross, info.cross)
    // } else {
    //     ((start + end - info.cross) / 2.0, info.cross)
    // }
}
// 返回位置和大小
fn align_end<K>(_start: f32, end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
    (
        end - info.margin_cross_end.or_else(0.0) - info.cross,
        info.cross,
    )
    // if let Number::Defined(r) = info.margin_cross_end {
    //     (end - r - info.cross, info.cross)
    // } else if let Number::Defined(r) = info.margin_cross_start {
    //     (start + r, info.cross)
    // } else {
    //     ((start + end - info.cross) / 2.0, info.cross)
    // }
}
// 返回位置和大小
fn align_center<K>(start: f32, end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
    if let (Number::Defined(r), Number::Defined(rr)) =
        (info.margin_cross_start, info.margin_cross_end)
    {
        ((start + end - info.cross - r - rr) / 2.0 + r, info.cross)
    } else if let (Number::Defined(r), _) = (info.margin_cross_start, info.margin_cross_end) {
        (start + r, info.cross)
    } else if let (_, Number::Defined(rr)) = (info.margin_cross_start, info.margin_cross_end) {
        (end - rr - info.cross, info.cross)
    } else {
        ((start + end - info.cross) / 2.0, info.cross)
    }
}
// 返回位置和大小
fn align_stretch<K>(start: f32, end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
    let r = info.margin_cross_start.or_else(0.0);
    let rr = info.margin_cross_end.or_else(0.0);
    (start + r, end - r - rr)
}
// 返回位置和大小
fn align_baseline<K>(
    start: f32,
    _end: f32,
    info: &RelNodeInfo<K>,
    baseline: &mut Number,
) -> (f32, f32) {
    if let Number::Defined(b) = baseline {
        (*b - info.cross, info.cross)
    } else {
        let r = info.margin_cross_start.or_else(0.0);
        // 如果基线还未计算，则计算
        *baseline = Number::Defined(start + r + info.cross);
        (start + r, info.cross)
    }
}

// 获得计算区域(大小和位置)， 大小为None表示自动计算
pub fn calc_rect(
    start: Dimension,
    end: Dimension,
    size: Number,
    margin_start: Dimension,
    margin_end: Dimension,
    parent: f32,
    containing_block_width: f32,
    align: isize,
) -> (Number, f32) {
    let calc_size = if let Number::Defined(r) = size {
        r
    } else {
        // 通过明确的前后确定大小
        let mut rr = if let Dimension::Points(rr) = start {
            rr
        } else if let Dimension::Percent(rr) = start {
            parent * rr
        } else {
            return (
                Number::Undefined,
                if let Dimension::Points(rrr) = end {
                    parent - rrr - margin_end.resolve_value(containing_block_width)
                } else if let Dimension::Percent(rrr) = end {
                    parent - parent * rrr - margin_end.resolve_value(containing_block_width)
                } else {
                    0.0
                },
            );
        };
        let mut rrr = if let Dimension::Points(rrr) = end {
            rrr
        } else if let Dimension::Percent(rrr) = end {
            parent * rrr
        } else {
            return (
                Number::Undefined,
                margin_start.resolve_value(containing_block_width),
            );
        };

        rr += margin_start.resolve_value(containing_block_width);
        rrr += margin_end.resolve_value(containing_block_width);
        return (Number::Defined(parent - rr - rrr), rr);
    };

    let calc_start = if let Dimension::Points(rr) = start {
        // 定义了size，同时定义了start， end自动失效
        rr
    } else if let Dimension::Percent(rr) = start {
        parent * rr
    } else {
        let rrr = if let Dimension::Points(rrr) = end {
            rrr
        } else if let Dimension::Percent(rrr) = end {
            parent * rrr
        } else {
            if align == 0 {
                // 居中对齐
                let s = (parent - calc_size) * 0.5;
                return calc_margin(
                    s,
                    s + calc_size,
                    calc_size,
                    margin_start,
                    margin_end,
                    containing_block_width,
                );
            } else if align > 0 {
                // 后对齐
                return (
                    Number::Defined(calc_size),
                    parent - margin_end.resolve_value(containing_block_width) - calc_size,
                );
            } else {
                // 前对齐
                return (
                    Number::Defined(calc_size),
                    margin_start.resolve_value(containing_block_width),
                );
            }
        };
        return (
            Number::Defined(calc_size),
            parent - rrr - margin_end.resolve_value(containing_block_width) - calc_size,
        );
    };
    // size为Percent或Points、 start为Percent或Points
    (
        Number::Defined(calc_size),
        calc_start + margin_start.resolve_value(containing_block_width),
    )
}

// 计算间隙（边框或空白区）， css规范边距不能使用百分比，css规范padding空白区的百分比是相对于包含块的宽度
pub fn calc_gap_by_containing_block(
    containing_block_size: &Size<f32>, // 包含块的尺寸
    gap: &SideGap<Dimension>,
) -> SideGap<f32> {
    SideGap {
        left: gap.left.resolve_value(containing_block_size.width),
        top: gap.top.resolve_value(containing_block_size.width),
        right: gap.right.resolve_value(containing_block_size.width),
        bottom: gap.bottom.resolve_value(containing_block_size.width),
    }
}

/// 计算margin, margin=Auto时自动填充剩余空间， 两边都Auto时平分剩余空间
pub fn calc_margin(
    mut start: f32,
    mut end: f32,
    size: f32,
    margin_start: Dimension,
    margin_end: Dimension,
    containing_block_width: f32,
) -> (Number, f32) {
    if let Dimension::Points(r) = margin_start {
        start += r;
        end = start + size;
    } else if let Dimension::Percent(r) = margin_start {
        start += r * containing_block_width;
        end = start + size;
    } else if let Dimension::Points(r) = margin_end {
        end -= r;
        start = end - size;
    } else if let Dimension::Percent(r) = margin_end {
        end -= r * containing_block_width;
        start = end - size;
    } else {
        out_any!(
            println,
            // log::trace,
            "calc_margin auto=============end: {:?}, start:{:?}, size:{:?}",
            end,
            start,
            size
        );
        // 平分剩余大小
        let d = (end - start - size) / 2.0;
        start += d;
        end -= d;
    }
    (Number::Defined(end - start), start)
}

/// 在flex计算的区域中 根据pos的位置进行偏移
pub fn calc_pos(position_start: Dimension, position_end: Dimension, parent: f32, pos: f32) -> f32 {
    if let Dimension::Points(r) = position_start {
        pos + r
    } else if let Dimension::Percent(r) = position_start {
        pos + parent * r
    } else {
        if let Dimension::Points(r) = position_end {
            pos - r
        } else if let Dimension::Percent(r) = position_end {
            pos - parent * r
        } else {
            pos
        }
    }
}
/// 计算子节点的大小
pub fn calc_number(s: Dimension, parent: f32) -> Number {
    if let Dimension::Points(r) = s {
        Number::Defined(r)
    } else if let Dimension::Percent(r) = s {
        Number::Defined(parent * r)
    } else {
        Number::Undefined
    }
}

// 计算定位属性的节点的大小（margin）
pub fn calc_location_number(s: Dimension, parent: f32) -> Number {
    if let Dimension::Points(r) = s {
        Number::Defined(r)
    } else if let Dimension::Percent(r) = s {
        Number::Defined(parent * r)
    } else if let Dimension::Undefined = s {
        Number::Defined(0.0)
    } else {
        Number::Undefined
    }
}

pub fn calc_length(length: Number, min_value: Number, max_value: Number) -> Number {
    match length {
        Number::Defined(r) => Number::Defined(min_max_calc(r, min_value, max_value)),
        _ => return length,
    }
}
pub(crate) static mut PP: usize = 0;
pub(crate) static mut PC: usize = 0;
