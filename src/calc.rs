#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use core::mem::replace;
use pi_null::Null;
use pi_slotmap::{DefaultKey, Key};
use pi_slotmap_tree::{Down, Up};

use std::marker::PhantomData;
// use map::vecmap::VecMap;
use crate::geometry::*;
use crate::number::*;
use crate::style::*;
use core::cmp::Ord;
use std::cmp::Ordering;
use std::ops::IndexMut;
// use pi_tree::Tree;
use pi_heap::simple_heap::SimpleHeap;

#[allow(dead_code)]
fn ppp() -> String {
    let mut s = String::from("");
    for _ in 0..unsafe { PC } {
        s.push_str("--");
    }
    for _ in 0..unsafe { PP } {
        s.push_str("**");
    }
    s
}
// 每个子节点根据 justify-content align-items align-self，来计算main cross的位置和大小
macro_rules! item_calc {
    ($self:ident, $temp:ident, $start:ident, $end:ident, $content_size:ident, $cross_start:ident, $cross_end:ident, $normal:ident, $pos:ident, $split:ident, $main_calc:ident, $main_calc_reverse:ident) => {
        let ai = $temp.flex.align_items;
        if $normal {
            if $temp.row {
                while *$start < $end {
                    let (info, temp) = unsafe { $temp.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    $self.layout_node(info.id, main, cross, temp, $content_size);
                }
            } else {
                while *$start < $end {
                    let (info, temp) = unsafe { $temp.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    $self.layout_node(info.id, cross, main, temp, $content_size);
                }
            }
        } else {
            if $temp.row {
                while *$start < $end {
                    let (info, temp) = unsafe { $temp.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc_reverse(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    $self.layout_node(info.id, main, cross, temp, $content_size);
                }
            } else {
                while *$start < $end {
                    let (info, temp) = unsafe { $temp.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc_reverse(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    $self.layout_node(info.id, cross, main, temp, $content_size);
                }
            }
        }
    };
}

macro_rules! make_func {
    ($name:ident, $type:ident) => {
        $crate::paste::item! {
            pub fn $name(&self) -> bool {
                (self.0 & INodeStateType::$type as usize) != 0
            }

            #[allow(dead_code)]
            pub fn [<$name _true>](&mut self) {
                self.0 |= INodeStateType::$type as usize
            }

            #[allow(dead_code)]
            pub fn [<$name _false>](&mut self) {
                self.0 &= !(INodeStateType::$type as usize)
            }
            #[allow(dead_code)]
            pub fn [<$name _set>](&mut self, v: bool) {
                if v {
                    self.0 |= INodeStateType::$type as usize
                }else {
                    self.0 &= !(INodeStateType::$type as usize)
                }

            }
        }
    };
}

macro_rules! make_impl {
    ($struct:ident) => {
        impl $struct {
            pub(crate) fn new(s: usize) -> Self {
                INodeState(s)
            }
            make_func!(children_dirty, ChildrenDirty);
            make_func!(self_dirty, SelfDirty);
            make_func!(children_abs, ChildrenAbs);
            make_func!(children_rect, ChildrenRect); // 相对定位大小由自身确定
            make_func!(self_rect, SelfRect);
            make_func!(children_no_align_self, ChildrenNoAlignSelf);
            make_func!(children_index, ChildrenIndex);
            make_func!(vnode, VNode);
            make_func!(rnode, RNode);
            make_func!(abs, Abs);
            make_func!(size_defined, SizeDefined);
            make_func!(line_start_margin_zero, LineStartMarginZero);
            make_func!(breakline, BreakLine);
            pub(crate) fn set_true(&mut self, other: &Self) {
                self.0 |= other.0;
            }
            pub(crate) fn set_false(&mut self, other: &Self) {
                self.0 &= !other.0
            }
        }
    };
}

// // 布局计算结果
// #[derive(Clone, Debug, PartialEq)]
// pub struct LayoutR {
//     pub rect: Rect<f32>,
//     pub border: Rect<f32>,
//     pub padding: Rect<f32>,
// }

pub trait LayoutR {
    // 取到布局属性
    fn rect(&self) -> &Rect<f32>;
    fn border(&self) -> &Rect<f32>;
    fn padding(&self) -> &Rect<f32>;

    // 设置布局属性
    fn set_rect(&mut self, v: Rect<f32>);
    fn set_border(&mut self, v: Rect<f32>);
    fn set_padding(&mut self, v: Rect<f32>);

    /// 布局属性设置完成会调用此方法
    fn set_finish(&mut self);
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct INodeState(usize);
make_impl!(INodeState);

//节点状态
pub enum INodeStateType {
    ChildrenDirty = 1,        // 子节点布局需要重新计算
    SelfDirty = 2,            // 自身布局需要重新计算
    ChildrenAbs = 4,          // 子节点是否都是绝对坐标， 如果是，则本节点的自动大小为0.0
    ChildrenNoAlignSelf = 16, // 子节点没有设置align_self
    ChildrenIndex = 32,       // 子节点是否为顺序排序

    VNode = 64, // 是否为虚拟节点, 虚拟节点下只能放叶子节点

    Abs = 128,                  // 是否为绝对坐标
    SizeDefined = 512,          // 是否为根据子节点自动计算大小
    LineStartMarginZero = 1024, // 如果该元素为行首，则margin_start为0
    BreakLine = 2048,           // 强制换行

    RNode = 4096, // 真实节点

    ChildrenRect = 8192,
    SelfRect = 16384, // 自身区域不受父节点或子节点影响
}
// // TODO max min aspect_ratio， RectStyle也可去掉了. 将start end改为left right。 将数据结构统一到标准结构下， 比如Rect Size Point
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct CharNode {
//     pub ch: char,                // 字符
//     pub margin_start: f32, // margin
//     pub size: (f32, f32),        // 字符大小
//     pub pos: (f32, f32),         // 位置
//     pub ch_id_or_count: usize,   // 字符id或单词的字符数量
//     pub base_width: f32,         // font_size 为32 的字符宽度
// 	pub char_i: isize,// 字符在整个节点中的索引
// 	pub context_id: isize, // 如果是多字符文字中的某个字符，则存在一个容易索引
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharNode {
    pub ch: char,                // 字符
    pub margin: Rect<Dimension>, // margin
    pub size: Size<Dimension>,   // 字符大小
    pub pos: Rect<f32>,          // 位置
    pub count: usize, // 字符id或单词的字符数量 ch==char::from(0)时，表示单词容器节点，此时ch_id_or_count表示该节点中的字符数量
    pub ch_id: DefaultKey,
    pub char_i: isize,     // 字符在整个节点中的索引
    pub context_id: isize, // 如果是多字符文字中的某个字符，则存在一个上下文索引
}

impl Default for CharNode {
    fn default() -> Self {
        CharNode {
            ch: char::from(0),
            margin: Rect {
                top: Dimension::Points(0.0),
                right: Dimension::Points(0.0),
                bottom: Dimension::Points(0.0),
                left: Dimension::Points(0.0),
            },
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
    pub state: INodeState,
    pub text: Vec<CharNode>, // 文字节点
    pub char_index: usize,   // 如果是图文混排，代表在Vec<CharNode>中的位置
    pub scale: f32,          // 文字布局的缩放值， 放到其它地方去？TODO
    pub is_sample: bool,     // 是否进行简单布局
}

impl INode {
    pub fn new(value: INodeStateType, char_index: usize) -> Self {
        INode {
            state: INodeState::new(value as usize + INodeStateType::ChildrenIndex as usize),
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
            state: INodeState::new(
                INodeStateType::ChildrenAbs as usize
                    + INodeStateType::ChildrenRect as usize
                    + INodeStateType::ChildrenNoAlignSelf as usize
                    + INodeStateType::ChildrenIndex as usize
                    + INodeStateType::RNode as usize,
            ),
            text: Vec::new(),
            char_index: 0,
            scale: 1.0,
            is_sample: false,
        }
    }
}

impl INode {
    pub fn is_vnode(&self) -> bool {
        self.state.vnode()
    }

    // 是否为真实节点
    pub fn is_rnode(&self) -> bool {
        self.state.rnode()
    }

    pub fn set_vnode(&mut self, vnode: bool) {
        if vnode {
            self.state.vnode_true();
        } else {
            self.state.vnode_false();
        }
    }

    pub fn set_rnode(&mut self, rnode: bool) {
        if rnode {
            self.state.rnode_true();
        } else {
            self.state.rnode_false();
        }
    }

    pub fn set_line_start_margin_zero(&mut self, b: bool) {
        if b {
            self.state.line_start_margin_zero_true();
        } else {
            self.state.line_start_margin_zero_false();
        }
    }
    pub fn set_breakline(&mut self, b: bool) {
        if b {
            self.state.breakline_true();
        } else {
            self.state.breakline_false();
        }
    }
}

// impl Default for LayoutR {
//     fn default() -> LayoutR {
//         LayoutR {
//             rect: Rect {
//                 left: 0.0,
//                 right: 0.0,
//                 top: 0.0,
//                 bottom: 0.0,
//             },
//             border: Rect {
//                 left: 0.0,
//                 right: 0.0,
//                 top: 0.0,
//                 bottom: 0.0,
//             },
//             padding: Rect {
//                 left: 0.0,
//                 right: 0.0,
//                 top: 0.0,
//                 bottom: 0.0,
//             },
//         }
//     }
// }

// impl LayoutR {
//     // 从LayoutR上获得节点的内容区大小
//     pub(crate) fn get_content_size(&self) -> (f32, f32) {
//         (
//             self.rect.right
//                 - self.rect.left
//                 - self.border.left
//                 - self.border.right
//                 - self.padding.left
//                 - self.padding.right,
//             self.rect.bottom
//                 - self.rect.top
//                 - self.border.top
//                 - self.border.bottom
//                 - self.padding.top
//                 - self.padding.bottom,
//         )
//     }
// }

pub(crate) fn get_content_size<T: LayoutR>(l: &T) -> (f32, f32) {
    (
        l.rect().right
            - l.rect().left
            - l.border().left
            - l.border().right
            - l.padding().left
            - l.padding().right,
        l.rect().bottom
            - l.rect().top
            - l.border().top
            - l.border().bottom
            - l.padding().top
            - l.padding().bottom,
    )
}

// 计算时使用的临时数据结构
struct Cache<K> {
    // size: Size<Number>,
    size1: (f32, f32), // 最小大小
    main: Number, // 主轴的大小, 用于约束换行，该值需要参考节点设置的width或height，以及max_width或max_height, 如果都未设置，则该值为无穷大
    cross: Number, // 交叉轴的大小
    main_line: f32, // 主轴的大小, 用于判断是否折行

    main_value: f32, // 主轴的像素大小，该值需要参考width或height，以及min_width或min_height，用于子节点未将该节点撑得更大时，节点的主轴布局结果
    cross_value: f32, // 交叉轴的像素大小，该值需要参考width或height，以及min_width或min_height，用于子节点未将该节点撑得更大时，节点的交叉轴布局结果

    state: INodeState, // 统计子节点的 ChildrenAbs ChildrenNoAlignSelf ChildrenIndex

    heap: SimpleHeap<OrderSort<K>>,
    temp: Temp<K>, // 缓存的子节点数组
    vnode: Vec<K>,
}

impl<K: Null + Clone> Cache<K> {
    // 文字的flex布局
    fn text_layout(
        &mut self,
        id: K,
        text: &mut Vec<CharNode>,
        line: &mut LineInfo,
        mut char_index: usize,
    ) {
        out_any!(
            log::trace,
            "text_layout, id:{:?}, text: {:?}",
            &id,
            &text
        );
        let len = text.len();
        while char_index < len {
            let r = &text[char_index];
            let (main_d, cross_d) = self.temp.main_cross(
                if let Dimension::Points(r) = r.size.width {
                    r
                } else {
					panic!("")
				},
                if let Dimension::Points(r) = r.size.height {
                    r
                } else {
					panic!("")
				},
            );
            let margin = self.temp.main_cross(
                (
                    Dimension::Points(if let Dimension::Points(r) = r.margin.left {
						r
					} else {
						panic!("")
					}),
                    Dimension::Points(0.0),
                ),
                (Dimension::Points(0.0), Dimension::Points(0.0)),
            );
            let mut info = RelNodeInfo {
                id: id.clone(),
                grow: 0.0,
                shrink: 0.0,
                main: main_d,
                cross: cross_d,
                margin_main: 0.0,
                margin_main_start: calc_location_number((margin.0).0, self.main_value),
                margin_main_end: calc_location_number((margin.0).1, self.main_value),
                margin_cross_start: calc_location_number((margin.1).0, self.cross_value),
                margin_cross_end: calc_location_number((margin.1).1, self.cross_value),
                align_self: AlignSelf::Auto,
                main_d: Dimension::Points(main_d),
                cross_d: Dimension::Points(cross_d),
                line_start_margin_zero: true,
                breakline: r.ch == char::from('\n'),
                // min_main: Number::Undefined,
                // max_main: Number::Undefined,
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
            self.add_vec(line, 0, info, TempType::CharIndex(char_index));
            // 判断是否为单词容器
            if r.ch == char::from(0) {
                char_index += r.count;
            } else {
                char_index += 1;
            }
        }
    }
    // 添加到数组中，计算当前行的grow shrink 是否折行及折几行
    fn add_vec(
        &mut self,
        line: &mut LineInfo,
        _order: isize,
        info: RelNodeInfo<K>,
        temp: TempType<K>,
    ) {
        //out_any!(log::trace, "add info:{:?}", info);
        line.add(self.main_line, &info);
        self.temp.rel_vec.push((info, temp));
    }
    // 添加到堆中
    fn add_heap(
        &mut self,
        line: &mut LineInfo,
        order: isize,
        info: RelNodeInfo<K>,
        temp: TempType<K>,
    ) {
        line.add(self.main_line, &info);
        self.heap
            .push(OrderSort(order, self.heap.len(), info, temp));
    }
}
#[derive(Clone, PartialEq, PartialOrd, Debug)]
enum TempType<K> {
    None,
    Ok,
    R(Temp<K>),
    CharIndex(usize),
}
impl<K> Default for TempType<K> {
    fn default() -> Self {
        TempType::None
    }
}
// 排序节点
#[derive(Default, Clone, Debug)]
struct OrderSort<K>(isize, usize, RelNodeInfo<K>, TempType<K>); // (order, index, Info, temp)
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

//临时缓存的节点样式、大小和子节点数组
#[derive(Clone, PartialEq, PartialOrd, Debug)]
struct Temp<K> {
    flex: ContainerStyle,
    row: bool,
    abs_vec: Vec<(K, K, K, INodeState, bool)>, // (id, children_head, children_tail, state, is_text) 绝对定位的子节点数组
    rel_vec: Vec<(RelNodeInfo<K>, TempType<K>)>, // 相对定位的子节点数组
    children_percent: bool,                    // 子节点是否有百分比宽高
}

impl<K> Default for Temp<K> {
    fn default() -> Self {
        Self {
            flex: ContainerStyle::default(),
            row: Default::default(),
            abs_vec: Vec::new(), // (id, children_head, children_tail, state, is_text) 绝对定位的子节点数组
            rel_vec: Vec::new(), // 相对定位的子节点数组
            children_percent: false,
        }
    }
}
// //容器样式
// #[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
// pub(crate) struct ContainerStyle {
//     pub(crate) flex_direction: FlexDirection,
//     pub(crate) flex_wrap: FlexWrap,
//     pub(crate) justify_content: JustifyContent,
//     pub(crate) align_items: AlignItems,
//     pub(crate) align_content: AlignContent,
// }
//相对定位下缓存的节点信息
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct RelNodeInfo<K> {
    id: K,
    grow: f32,                    // 节点grow的值
    shrink: f32,                  // 节点shrink的值
    main: f32,                    // 节点主轴尺寸(受basis影响)
    cross: f32,                   // 节点交叉轴尺寸
    margin_main: f32,             // 节点主轴方向 margin_start margin_end的大小
    margin_main_start: Number,    // 节点主轴方向 margin_start的大小
    margin_main_end: Number,      // 节点主轴方向 margin_end的大小
    margin_cross_start: Number,   // 节点交叉轴方向 margin_start的大小
    margin_cross_end: Number,     // 节点交叉轴方向 margin_end的大小
    align_self: AlignSelf,        // 节点的align_self
    main_d: Dimension,            // 节点主轴大小
    cross_d: Dimension,           // 节点交叉轴大小
    line_start_margin_zero: bool, // 如果该元素为行首，则margin_start为0
    breakline: bool,              // 强制换行
                                  // min_main: Number,  //主轴最小尺寸
                                  // max_main: Number, // 主轴最大尺寸
}

// 计算时统计的行信息
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct LineInfo {
    main: f32,            // 行内节点主轴尺寸的总值，不受basis影响
    cross: f32,           // 多行子节点交叉轴的像素的累计值
    item: LineItem,       // 当前计算的行margin_auto
    items: Vec<LineItem>, // 已计算的行
}
//行信息中每行条目
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct LineItem {
    count: usize,       // 行内节点总数量
    grow: f32,          // 行内节点grow的总值
    shrink: f32,        // 行内节点shrink的总值
    margin_auto: usize, // 行内节点主轴方向 margin=auto 的数量
    main: f32,          // 行内节点主轴尺寸的总值（包括size margin）
    cross: f32,         // 行内节点交叉轴尺寸的最大值
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

impl<K> Cache<K> {
    fn new(
        flex: ContainerStyle,
        size: Size<Number>,
        min_size: Size<Number>,
        max_width: Number,
        max_height: Number,
    ) -> Self {
        // 计算主轴和交叉轴，及大小
        let row = flex.flex_direction == FlexDirection::Row
            || flex.flex_direction == FlexDirection::RowReverse;
        let (main, cross, max_main, min_main, min_cross) = if row {
            (
                size.width,
                size.height,
                max_width,
                min_size.width,
                min_size.height,
            )
        } else {
            (
                size.height,
                size.width,
                max_height,
                min_size.height,
                min_size.width,
            )
        };
        let m = if flex.flex_wrap == FlexWrap::NoWrap {
            std::f32::INFINITY
        } else {
            max_calc(main, max_main).or_else(std::f32::INFINITY)
        };
        unsafe { PP += 1 };
        Cache {
            // size,
            size1: (min_size.width.or_else(0.0), min_size.height.or_else(0.0)),
            main,
            cross,
            main_line: m,
            main_value: min_main.or_else(0.0),
            cross_value: min_cross.or_else(0.0),
            state: INodeState::new(
                INodeStateType::ChildrenAbs as usize
                    + INodeStateType::ChildrenRect as usize
                    + INodeStateType::ChildrenNoAlignSelf as usize
                    + INodeStateType::ChildrenIndex as usize,
            ),
            heap: SimpleHeap::new(Ordering::Less),
            temp: Temp::<K>::new(flex, row),
            vnode: Vec::new(),
        }
    }
}

impl<K> Temp<K> {
    fn new(flex: ContainerStyle, row: bool) -> Self {
        Temp {
            flex,
            row,
            abs_vec: Vec::new(),
            rel_vec: Vec::new(),
            children_percent: false,
        }
    }
    fn main_cross<T>(&self, w: T, h: T) -> (T, T) {
        if self.row {
            (w, h)
        } else {
            (h, w)
        }
    }

    // 用缓存的相对定位的子节点数组重建行信息
    fn reline(&mut self, main: f32, cross: f32) -> LineInfo {
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
        out_any!(log::trace, "{:?}reline: line:{:?}", ppp(), &line);
        line
    }

    // 多行的区间计算
    fn multi_calc(&self, cross: f32, split: f32, pos: &mut f32) -> (f32, f32) {
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
                log::trace,
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

pub trait Get<K> {
    type Target;
    fn get(&self, k: K) -> Self::Target;
}

pub trait GetMut<K> {
    type Target;
    fn get_mut(&mut self, k: K) -> Self::Target;
}

/// 布局上下文
pub struct LayoutContext<
    'a,
    K: Null + Clone + Copy,
    S,
    T,
    L,
    I,
    R,
    LI: Get<K, Target = L>,
    LR: LayoutR,
> {
    pub mark: PhantomData<L>,
    pub i_nodes: &'a mut I,
    pub layout_map: &'a mut R,
    pub notify_arg: &'a mut T,
    pub notify: fn(&mut T, K, &LR),
    pub tree: &'a S,
    pub style: &'a LI,
}

pub trait TreeStorage<K> {
    fn get_up(&self, id: K) -> Option<Up<K>>;
    fn get_down(&self, id: K) -> Option<Down<K>>;

    fn up(&self, id: K) -> Up<K>;
    fn down(&self, id: K) -> Down<K>;

    fn get_layer(&self, k: K) -> Option<usize>;
    fn layer(&self, k: K) -> usize;
}

impl<'a, K, S, T, L, I, R, LI, LR> LayoutContext<'a, K, S, T, L, I, R, LI, LR>
where
    K: Null + Clone + Copy,
    S: TreeStorage<K>,
    L: FlexLayoutCombine,
    LI: Get<K, Target = L>,
    LR: LayoutR,
    I: IndexMut<K, Output = INode>,
    R: GetMut<K, Target = LR>,
{
    /// 绝对定位下的布局，如果size=auto， 会先调用子节点的布局
    pub(crate) fn abs_layout(
        &mut self,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        state: INodeState,
        parent_size: (f32, f32),
        flex: &ContainerStyle,
    ) {
        let style = &self.style.get(id);
        out_any!(log::trace, "abs_layout, id:{:?}, parent_size: {:?}, style: {:?}, display: {:?}", id, parent_size, style, style.display());
        if style.display() == Display::None {
            return;
        }
		let a1 = if JustifyContent::Center == flex.justify_content {
			0
		} else if JustifyContent::FlexEnd == flex.justify_content {
			1
		} else {
			-1
		};
		let mut a2 = if AlignItems::Center == flex.align_items {
			0
		} else if AlignItems::FlexEnd == flex.align_items {
			1
		} else {
			-1
		};
		if AlignSelf::Center == style.align_self() {
			a2 = 0;
		} else if AlignSelf::FlexEnd == style.align_self() {
			a2 = 1;
		}
		// flex.flex_wrap == FlexWrap::WrapReverse会时的交叉轴布局反向
		if flex.flex_wrap == FlexWrap::WrapReverse {
			a2 = -a2;
		}

        let (walign, halign) = if flex.flex_direction == FlexDirection::Row
            || flex.flex_direction == FlexDirection::RowReverse
        {
            (a1, a2)
        } else {
            (a2, a1)
        };

        let mut w = calc_rect(
            style.position_left(),
            style.position_right(),
            style.width(),
            style.margin_left(),
            style.margin_right(),
            parent_size.0,
            state.children_abs(),
            walign,
        );
        let mut h = calc_rect(
            style.position_top(),
            style.position_bottom(),
            style.height(),
            style.margin_top(),
            style.margin_bottom(),
            parent_size.1,
            state.children_abs(),
            halign,
        );

        let (min_width, max_width, min_height, max_height) = (
            calc_number(style.min_width(), parent_size.0),
            calc_number(style.max_width(), parent_size.0),
            calc_number(style.min_height(), parent_size.1),
            calc_number(style.max_height(), parent_size.1),
        );
		out_any!(
            log::trace,
            "abs_layout, id:{:?} size:{:?} walign: {:?}, halign: {:?} position:{:?}, margin: {:?}, flex_direction {:?}, w: {:?}, h: {:?}",
            id,
            (style.width(), style.height()),
			walign,
			halign,
            style.position(),
			style.margin(),
			flex.flex_direction,
			w,
			h
        );

        if w.0 == Number::Undefined || h.0 == Number::Undefined {
            // 根据子节点计算大小
            let direction = style.direction();
            let pos = style.position();
            let margin = style.margin();
            let border = style.border();
            let padding = style.padding();
            let ww = style.calc_horizontal_content_size(w.0);
            let hh = style.calc_vertical_content_size(h.0);
            let mut cache = Cache::new(
                style.container_style(),
                Size {
                    width: ww,
                    height: hh,
                },
                Size {
                    width: calc_length(ww, min_width),
                    height: calc_length(hh, min_height),
                },
                style.calc_horizontal_content_size(max_width),
                style.calc_vertical_content_size(max_height),
            );

            let (ww, hh, _r) = self.auto_layout(
                &mut cache,
                true,
                id,
                is_text,
                child_head,
                child_tail,
                state.children_index(),
                direction,
                &border,
                &padding,
            );
            out_any!(log::trace, "calc_rect: id: {:?}, hh:{:?}", id, hh);
            // 再次计算区域
            w = calc_rect(
                pos.left,
                pos.right,
                Dimension::Points(ww),
                margin.left,
                margin.right,
                parent_size.0,
                false,
                walign,
            );
            h = calc_rect(
                pos.top,
                pos.bottom,
                Dimension::Points(hh),
                margin.top,
                margin.bottom,
                parent_size.1,
                false,
                halign,
            );

            let mut layout = self.layout_map.get_mut(id);
            // 设置布局的值
            set_layout_result(
                &mut layout,
                self.notify.clone(),
                self.notify_arg,
                id,
                (w.1, h.1),
                (
                    min_max_calc(w.0.or_else(0.0), min_width, max_width),
                    min_max_calc(h.0.or_else(0.0), min_height, max_height),
                ),
                &border,
                &padding,
            );
        } else {
            let flex = style.container_style();
            self.set_layout(
                id,
                is_text,
                child_head,
                child_tail,
                flex,
                style.direction(),
                style.border(),
                style.padding(),
                state,
                (w.1, h.1),
                (
                    min_max_calc(w.0.or_else(0.0), min_width, max_width),
                    min_max_calc(h.0.or_else(0.0), min_height, max_height),
                ),
            );
        };
    }

    // 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先检查自身的布局数据，然后修改子节点的布局数据
    pub(crate) fn rel_layout(
        &mut self,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        state: INodeState,
    ) {
        let style = &self.style.get(id);
        out_any!(log::trace, "rel_layout, id:{:?}, style: {:?}, display: {:?}", id, style, style.display());
        if style.display() == Display::None {
            return;
        }
        let flex = style.container_style();
        let direction = style.direction();
        let border = style.border();
        let padding = style.padding();
        let rect = self.layout_map.get_mut(id).rect().clone();
        self.set_layout(
            id,
            is_text,
            child_head,
            child_tail,
            flex,
            direction,
            border,
            padding,
            state,
            (rect.left, rect.top),
            (rect.right - rect.left, rect.bottom - rect.top),
        );
    }

    fn layout_node(
        &mut self,
        id: K,
        width: (f32, f32),
        height: (f32, f32),
        temp: &mut TempType<K>,
        parent_size: (f32, f32),
    ) {
        let i_node = &mut self.i_nodes[id];
        if let TempType::CharIndex(r) =  temp {
			// 文字布局
			let cnode = &mut i_node.text[*r];
			cnode.pos = Rect {
				left: width.0,
				top: height.0,
				right: width.0 + width.1, // TODO
				bottom: height.0 + height.1,
			};
			out_any!(
				log::trace,
				"set_layout text: {:?}, {:?}",
				Rect {
					left: width.0,
					top: height.0,
					right: width.0 + width.1, // TODO
					bottom: height.0 + height.1,
				},
				cnode.ch
			);
			return;
        }
        let s = self.style.get(id);
        let flex = s.container_style();
        let direction = s.direction();
        let border = s.border();
        let padding = s.padding();
        let (child_head, child_tail) = self
            .tree
            .get_down(id)
            .map_or((K::null(), K::null()), |down| (down.head(), down.tail()));
        let state = i_node.state;
        i_node.state.set_false(&INodeState::new(
            INodeStateType::ChildrenDirty as usize + INodeStateType::SelfDirty as usize,
        ));

        let x = calc_pos(
            s.position_left(),
            s.position_right(),
            parent_size.0,
            width.0,
        );
        let y = calc_pos(
            s.position_top(),
            s.position_bottom(),
            parent_size.1,
            height.0,
        );
        // 设置布局的值
        if let TempType::R(t) = temp {
			// 有Auto的节点需要父确定大小，然后自身的temp重计算及布局
			let mut layout = self.layout_map.get_mut(id);
			set_layout_result(
				&mut layout,
				self.notify,
				self.notify_arg,
				id,
				(x, y),
				(width.1, height.1),
				&border,
				&padding,
			);
			let s = get_content_size(&mut layout);
			let mc = t.main_cross(s.0, s.1);
			let line = t.reline(mc.0, mc.1);
			// 如果有临时缓存子节点数组
			self.temp_layout(t, s, mc.0, mc.1, &line);
		} else if let TempType::None = temp {
			// 确定大小的节点，需要进一步布局
			let is_text = i_node.text.len() > 0 && !state.vnode(); //i_node.text.len() > 0 && !state.vnode();
			self.set_layout(
				id,
				is_text,
				child_head,
				child_tail,
				flex,
				direction,
				border,
				padding,
				state,
				(x, y),
				(width.1, height.1),
			);
		} else {
			// 有Auto的节点在计算阶段已经将自己的子节点都布局了，节点自身等待确定位置
			let mut layout = self.layout_map.get_mut(id);
			set_layout_result(
				&mut layout,
				self.notify,
				self.notify_arg,
				id,
				(x, y),
				(width.1, height.1),
				&border,
				&padding,
			);
        }
    }

    // 自动布局，计算宽高， 如果is_notify则返回Temp(宽度或高度auto、宽度或高度undefined的节点会进入此方法)
    fn auto_layout(
        &mut self,
        cache: &mut Cache<K>,
        is_notify: bool,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        children_index: bool,
        direction: Direction,
        border: &Rect<Dimension>,
        padding: &Rect<Dimension>,
    ) -> (f32, f32, TempType<K>) {
        out_any!(
            log::trace,
            "{:?}auto_layout1: id:{:?} head:{:?} tail:{:?} is_notify:{:?}",
            ppp(),
            id,
            child_head,
            child_tail,
            is_notify
        );
        self.do_layout(
            cache,
            is_notify,
            id,
            is_text,
            child_head,
            child_tail,
            children_index,
            direction,
        );
        out_any!(
            log::trace,
            "{:?}auto_layout2: id:{:?}, size:{:?}",
            ppp(),
            id,
            (cache.main_value, cache.cross_value)
        );
        let (w, h) = cache.temp.main_cross(cache.main_value, cache.cross_value);
        (
            calc_size_from_content(w, border.left, border.right, padding.left, padding.right),
            calc_size_from_content(h, border.top, border.bottom, padding.top, padding.bottom),
            if is_notify {
                TempType::Ok
            } else {
                // 则将布局的中间数组暂存下来
                TempType::R(replace(&mut cache.temp, Temp::default()))
            },
        )
    }
    fn do_layout(
        &mut self,
        cache: &mut Cache<K>,
        is_notify: bool,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        children_index: bool,
        direction: Direction,
    ) {
        let mut line = LineInfo::default();
        out_any!(
            log::trace,
            "{:?}do layout1, id:{:?} is_notify:{:?}, is_text: {:?}",
            ppp(),
            id,
            is_notify,
			is_text
        );

        if is_text {
            let i_node = &mut self.i_nodes[id];
            cache.text_layout(id, &mut i_node.text, &mut line, 0);
        } else {
            self.node_layout(
                cache,
                is_notify && cache.main.is_defined() && cache.cross.is_defined(),
                &mut line,
                if direction != Direction::RTL {
                    child_head
                } else {
                    child_tail
                },
                children_index,
                direction,
            );
        }
        line.cross += line.item.cross;

        out_any!(
            log::trace,
            "{:?}do layout2, id:{:?} line:{:?}, vec:{:?}",
            ppp(),
            id,
            &line,
            &cache.temp.rel_vec
        );
        if children_index {
            // 从堆中添加到数组上
            while let Some(OrderSort(_, _, info, temp)) = cache.heap.pop() {
                cache.temp.rel_vec.push((info, temp))
            }
        }
        // 如果自动大小， 则计算实际大小
        if !cache.main.is_defined() {
            cache.main_value = f32::max(line.main, cache.main_value);
        }
        if !cache.cross.is_defined() {
            // self.cross1 = line.cross + line.item.cross; ？？？
            cache.cross_value = f32::max(line.cross, cache.cross_value);
        }
        // 记录节点的子节点的统计信息
        // let node = &tree[id];
        let i_node = &mut self.i_nodes[id];
        i_node.state.set_false(&INodeState::new(
            INodeStateType::ChildrenAbs as usize
                + INodeStateType::ChildrenRect as usize
                + INodeStateType::ChildrenNoAlignSelf as usize
                + INodeStateType::ChildrenIndex as usize,
        ));
        i_node.state.set_true(&cache.state);
        // 根据is_notify决定是否继续计算
        if is_notify {
            let m_c = cache.temp.main_cross(cache.main_value, cache.cross_value);
            self.temp_layout(
                &mut cache.temp,
                m_c,
                cache.main_value,
                cache.cross_value,
                &line,
            );

            for v in cache.vnode.iter() {
                // 通知虚拟节点，布局改变
                let mut l = self.layout_map.get_mut(v.clone());
                l.set_finish();
                self.notify.clone()(self.notify_arg, v.clone(), &mut l);
            }
        }
    }

    // 节点的flex布局
    fn node_layout(
        &mut self,
        cache: &mut Cache<K>,
        is_notify: bool,
        line: &mut LineInfo,
        mut child: K,
        children_index: bool,
        direction: Direction,
    ) {
        // LayoutKey { entity: Id(LocalVersion(21474836532)), text_index: 18446744073709551615 }
        while !child.is_null() {
            let (next, prev) = self
                .tree
                .get_up(child)
                .map_or((K::null(), K::null()), |up| (up.next(), up.prev()));
            let (child_head, child_tail) = self
                .tree
                .get_down(child)
                .map_or((K::null(), K::null()), |down| (down.head(), down.tail()));
            let i_node = &mut self.i_nodes[child];
            if i_node.state.abs() {
                if i_node.state.self_rect() {
                    // 绝对区域不需计算
                    child = node_iter(direction, next, prev);
                    continue;
                }
                cache.state.children_rect_false();
                let id = child;
                child = node_iter(direction, next, prev);
                let state = i_node.state;
                i_node.state.set_false(&INodeState::new(
                    INodeStateType::ChildrenDirty as usize + INodeStateType::SelfDirty as usize,
                ));
                let is_text = i_node.text.len() > 0;
                if is_notify {
                    self.abs_layout(
                        id,
                        is_text,
                        child_head,
                        child_tail,
                        state,
                        cache.size1,
                        &cache.temp.flex,
                    );
                } else {
                    cache
                        .temp
                        .abs_vec
                        .push((id, child_head, child_tail, state, is_text));
                }
                continue;
            }
            let style = self.style.get(child);
            out_any!(
                log::trace,
                "node_layout, id:{:?}, next: {:?}, style: {:?}, display: {:?}, is_vnode: {:?}, is_notify: {:?}",
                child,
				next,
                &style,
				style.display(),
				i_node.state.vnode(),
				is_notify
            );
            if style.display() == Display::None {
				self.layout_map.get_mut(child); // 取布局结果的目的是为了在布局结果不存在的情况下插入默认值
                child = node_iter(direction, next, prev);
                continue;
            }
            if !i_node.state.self_rect() {
                cache.state.children_rect_false();
            }
            cache.state.children_abs_false();
            let id = child;
            child = node_iter(direction, next, prev);
            let vnode = i_node.state.vnode();
            if vnode {
                // 如果是虚拟节点， 则遍历其子节点， 加入到列表中
                let (child_head, child_tail) = self
                    .tree
                    .get_down(id)
                    .map_or((K::null(), K::null()), |down| (down.head(), down.tail()));
                let child = if direction != Direction::RTL {
                    child_head
                } else {
                    child_tail
                };
                self.node_layout(cache, is_notify, line, child, children_index, direction);
                cache.vnode.push(id);
                if is_notify {
                	self.notify.clone()(self.notify_arg, id, &mut self.layout_map.get_mut(id));
                }
                continue;
            }
            let order = style.order();
            if order != 0 {
                cache.state.children_index_false();
            }
            if style.align_self() != AlignSelf::Auto {
                cache.state.children_no_align_self_false();
            }
            // flex布局时， 如果子节点的宽高未定义，则根据子节点的布局进行计算。如果子节点的宽高为百分比，并且父节点对应宽高未定义，则为0
            let w = calc_number(style.width(), cache.size1.0);
            let h = calc_number(style.height(), cache.size1.1);
            out_any!(log::trace, "id: {:?}, parent_size:{:?}", id, cache.size1);
            let basis = style.flex_basis();
            let (main_d, cross_d) = cache.temp.main_cross(style.width(), style.height());

            let (min_width, max_width, min_height, max_height) = (
                calc_number(style.min_width(), cache.main_value),
                calc_number(style.max_width(), cache.main_value),
                calc_number(style.min_height(), cache.cross_value),
                calc_number(style.max_height(), cache.cross_value),
            );
            let (max_main, max_cross) = cache.temp.main_cross(max_width, max_height);
            let (min_main, min_cross) = cache.temp.main_cross(min_width, min_height);
            let margin = cache.temp.main_cross(
                (style.margin_left(), style.margin_right()),
                (style.margin_top(), style.margin_bottom()),
            );
            out_any!(log::trace, "main1,id:{:?}, main1:{:?}, main_d: {:?}, size: {:?}, min_main: {:?}, max_main: {:?}", id, cache.main_value, main_d, (style.width(), style.height()), min_main, max_main);
            let mut info = RelNodeInfo {
                id,
                grow: style.flex_grow(),
                shrink: style.flex_shrink(),
                main: min_max_calc(main_d.resolve_value(cache.main_value), min_main, max_main),
                cross: min_max_calc(
                    cross_d.resolve_value(cache.cross_value),
                    min_cross,
                    max_cross,
                ),
                margin_main: 0.0,
                margin_main_start: calc_location_number((margin.0).0, cache.main_value),
                margin_main_end: calc_location_number((margin.0).1, cache.main_value),
                margin_cross_start: calc_location_number((margin.1).0, cache.cross_value),
                margin_cross_end: calc_location_number((margin.1).1, cache.cross_value),
                align_self: style.align_self(),
                main_d: main_d,
                cross_d: cross_d,
                line_start_margin_zero: i_node.state.line_start_margin_zero(),
                breakline: i_node.state.breakline(),
                // min_main: min_main,
                // max_main: max_main,
            };
            let temp = if w == Number::Undefined || h == Number::Undefined {
                // 需要计算子节点大小
                let flex = style.container_style();
                let direction = style.direction();
                let border = style.border();
                let padding = style.padding();
                // 子节点大小是否不会改变， 如果不改变则直接布局
                let mut fix = true;
                // 主轴有3种情况后面可能会被改变大小
                if main_d.is_undefined() {
                    fix = basis.is_undefined()
                        && style.flex_grow() == 0.0
                        && style.flex_shrink() == 0.0;
                }
                //  交叉轴有2种情况后面可能会被改变大小
                if fix && cross_d.is_undefined() {
                    fix = style.align_self() != AlignSelf::Stretch
                        && cache.temp.flex.align_items != AlignItems::Stretch;
                }
                out_any!(
                    log::trace,
                    "{:?}calc size: id:{:?} fix:{:?} size:{:?} next:{:?}",
                    ppp(),
                    id,
                    fix,
                    (w, h),
                    child
                );
                let n_children_index = i_node.state.children_index();
                let is_text = i_node.text.len() > 0;
                let w =
                    calc_content_size(w, border.left, border.right, padding.left, padding.right);
                let h =
                    calc_content_size(h, border.top, border.bottom, padding.top, padding.bottom);
                let mut cache_new = Cache::new(
                    flex,
                    Size {
                        width: w,
                        height: h,
                    },
                    Size {
                        width: calc_length(w, min_width),
                        height: calc_length(h, min_height),
                    },
                    calc_content_size(
                        max_width,
                        border.left,
                        border.right,
                        padding.left,
                        padding.right,
                    ),
                    calc_content_size(
                        max_height,
                        border.top,
                        border.bottom,
                        padding.top,
                        padding.bottom,
                    ),
                );
                out_any!(
                    log::trace,
                    "cache, main_line: {:?}, id: {:?}",
                    cache_new.main_line,
                    id
                );
                // cache.main_line =
                // max_calc(w, max_width);
                // max_calc(h, max_height);
                let (ww, hh, r) = self.auto_layout(
                    &mut cache_new,
                    fix,
                    id,
                    is_text,
                    child_head,
                    child_tail,
                    n_children_index,
                    direction,
                    &border,
                    &padding,
                );
                let mc = cache.temp.main_cross(ww, hh);
                info.main = min_max_calc(mc.0, min_main, max_main);
                info.cross = min_max_calc(mc.1, min_cross, max_cross);
                r
            } else {
                // 确定大小的节点， TempType为None
                //out_any!(log::trace, "static size: id:{:?} size:{:?} next:{:?}", id, (w, h), child);
                TempType::None
            };
            let start = info.margin_main_start.or_else(0.0);
            let end = info.margin_main_end.or_else(0.0);
            // 主轴auto时记录子节点实际大
            let line_start = if line.item.count == 0 && info.line_start_margin_zero {
                // 处理行首
                0.0
            } else {
                start
            };
            info.margin_main = start + end;
            line.main += info.main + line_start + end;
            if let Dimension::Points(r) = basis {
                // 如果有basis, 则修正main
				info.main = r;
                info.main_d = basis;
			} else if let Dimension::Percent(r) = basis {
				info.main = cache.main_value * r;
        		info.main_d = basis;
			}

			if let Dimension::Percent(_r) = info.main_d {
				cache.temp.children_percent = true;
			} else if let Dimension::Percent(_r) = info.cross_d{
				cache.temp.children_percent = true;
			}
            // 设置shrink的大小
            info.shrink *= info.main;
            if children_index {
                // 如果需要排序，调用不同的添加方法
                cache.add_vec(line, order, info, temp);
            } else {
                cache.add_heap(line, order, info, temp);
            };
        }
    }

    // 设置节点的布局数据，如果内容宽高有改变，则调用自身的子节点布局方法
    fn set_layout(
        &mut self,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        flex: ContainerStyle,
        direction: Direction,
        border: Rect<Dimension>,
        padding: Rect<Dimension>,
        state: INodeState,
        pos: (f32, f32),
        size: (f32, f32),
    ) {
        out_any!(log::trace, 
			"{:?}set_layout: pos:{:?} size:{:?} id:{:?} head:{:?} tail:{:?} children_dirty:{:?} self_dirty:{:?} children_rect:{:?} children_abs:{:?}",
			ppp(),
			pos,
			size,
			id,
			child_head,
			child_tail,
			state.children_dirty(),
			state.self_dirty(),
			state.children_rect(),
			state.children_abs()
		);

        // 设置布局的值
        let mut layout = self.layout_map.get_mut(id);
        let r = if state.self_dirty()
            || layout.rect().left != pos.0
            || layout.rect().top != pos.1
            || layout.rect().right - layout.rect().left != size.0
            || layout.rect().bottom - layout.rect().top != size.1
        {
            set_layout_result(
                &mut layout,
                self.notify,
                self.notify_arg,
                id,
                pos,
                size,
                &border,
                &padding,
            )
        } else {
            LayoutSize::None
        };
        // 递归布局子节点
        let rr = if state.children_dirty() {
            get_content_size(&mut layout)
        } else {
            if let LayoutSize::Size(rr) = r {
				if state.children_rect()
					&& (state.children_abs()
						|| (state.children_no_align_self()
							&& (flex.flex_direction == FlexDirection::Row
								|| flex.flex_direction == FlexDirection::Column)
							&& flex.flex_wrap == FlexWrap::NoWrap
							&& flex.justify_content == JustifyContent::FlexStart
							&& flex.align_items == AlignItems::FlexStart))
				{
					// 节点的宽高变化不影响子节点的布局，还可进一步优化仅交叉轴大小变化
					return;
				}
				rr
            } else {
				return;
			}
        };
        // 宽高变动重新布局
        let mut cache = Cache::new(
            flex,
            Size {
                width: Number::Defined(rr.0),
                height: Number::Defined(rr.1),
            },
            Size {
                width: Number::Defined(rr.0),
                height: Number::Defined(rr.1),
            },
            Number::Undefined,
            Number::Undefined,
        );
        self.do_layout(
            &mut cache,
            true,
            id,
            is_text,
            child_head,
            child_tail,
            state.children_index(),
            direction,
        );
    }

    // 实际进行子节点布局
    fn temp_layout(
        &mut self,
        temp: &mut Temp<K>,
        size: (f32, f32),
        main: f32,
        cross: f32,
        line: &LineInfo,
    ) {
        out_any!(
            log::trace,
            "{:?}layout: style:{:?} size:{:?} main_cross:{:?}",
            ppp(),
            &temp.flex,
            size,
            (main, cross)
        );
        // 处理abs_vec
        for e in temp.abs_vec.iter() {
            self.abs_layout(e.0, e.4, e.1, e.2, e.3, size, &temp.flex);
        }
        let normal = temp.flex.flex_direction == FlexDirection::Row
            || temp.flex.flex_direction == FlexDirection::Column;
        let mut start = 0;
        // 根据行列信息，对每个节点布局
        if line.items.len() == 0 {
            // 单行处理
            self.temp_single_line(
                temp,
                main,
                &line.item,
                &mut start,
                temp.rel_vec.len(),
                size,
                0.0,
                cross,
                normal,
            );
            return;
        }

        // 多行布局，计算开始位置和分隔值
        let (mut pos, split) = match temp.flex.align_content {
            AlignContent::FlexStart => {
                if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                    (0.0, 0.0)
                } else {
                    (cross, 0.0)
                }
            }
            AlignContent::FlexEnd => {
                if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                    (cross - line.cross, 0.0)
                } else {
                    (line.cross, 0.0)
                }
            }
            AlignContent::Center => {
                if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                    ((cross - line.cross) / 2.0, 0.0)
                } else {
                    ((cross + line.cross) / 2.0, 0.0)
                }
            }
            AlignContent::SpaceBetween => {
                if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                    if line.items.len() > 0 {
                        (0.0, (cross - line.cross) / line.items.len() as f32)
                    } else {
                        ((cross - line.cross) / 2.0, 0.0)
                    }
                } else {
                    if line.items.len() > 0 {
                        (cross, (cross - line.cross) / line.items.len() as f32)
                    } else {
                        ((cross + line.cross) / 2.0, 0.0)
                    }
                }
            }
            AlignContent::SpaceAround => {
                let s = (cross - line.cross) / (line.items.len() + 1) as f32;
                if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                    (s / 2.0, s)
                } else {
                    (cross - s / 2.0, s)
                }
            }
            _ => {
                if line.cross - cross > EPSILON {
                    if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                        (0.0, 0.0)
                    } else {
                        (cross, 0.0)
                    }
                } else {
                    // 伸展， 平分交叉轴
                    let mut pos = if temp.flex.flex_wrap != FlexWrap::WrapReverse {
                        0.0
                    } else {
                        cross
                    };
                    let cross = cross / (line.items.len() + 1) as f32;
                    for item in line.items.iter() {
                        let (cross_start, cross_end) = temp.multi_calc(cross, 0.0, &mut pos);
                        self.temp_single_line(
                            temp,
                            main,
                            &item,
                            &mut start,
                            item.count,
                            size,
                            cross_start,
                            cross_end,
                            normal,
                        );
                    }
                    let (cross_start, cross_end) = temp.multi_calc(cross, 0.0, &mut pos);
                    self.temp_single_line(
                        temp,
                        main,
                        &line.item,
                        &mut start,
                        line.item.count,
                        size,
                        cross_start,
                        cross_end,
                        normal,
                    );
                    return;
                }
            }
        };
        for item in line.items.iter() {
            out_any!(
                log::trace,
                "single_line!!, item: {:?}, split: {:?}, pos: {:?}",
                item,
                split,
                pos
            );
            let (cross_start, cross_end) = temp.multi_calc(item.cross, split, &mut pos);
            self.temp_single_line(
                temp,
                main,
                &item,
                &mut start,
                item.count,
                size,
                cross_start,
                cross_end,
                normal,
            );
        }
        out_any!(
            log::trace,
            "single_line!!, item: {:?}, split: {:?}, pos: {:?}, cross:{:?}",
            &line.item,
            split,
            pos,
            line.cross
        );
        let (cross_start, cross_end) = temp.multi_calc(line.item.cross, split, &mut pos);
        self.temp_single_line(
            temp,
            main,
            &line.item,
            &mut start,
            line.item.count,
            size,
            cross_start,
            cross_end,
            normal,
        );
    }

    // 处理单行的节点布局
    fn temp_single_line(
        &mut self,
        temp: &mut Temp<K>,
        main: f32,
        item: &LineItem,
        start: &mut usize,
        count: usize,
        content_size: (f32, f32),
        cross_start: f32,
        cross_end: f32,
        normal: bool,
    ) {
        if count == 0 {
            return;
        }
        out_any!(
            log::trace,
            "{:?}single_line: normal:{:?} content_size:{:?}, cross:{:?} start_end:{:?} main:{:?}",
            ppp(),
            normal,
            content_size,
            (cross_start, cross_end),
            (*start, count),
            (main, item.main)
        );
        let first = unsafe { temp.rel_vec.get_unchecked_mut(*start) };
        if first.0.line_start_margin_zero {
            // 修正行首的margin
            first.0.margin_main_start = Number::Defined(0.0);
        }
        let end = *start + count;
        let mut pos = if normal { 0.0 } else { main };
        // 浮点误差计算
        if main - item.main > EPSILON {
            // 表示需要放大
            if item.grow > 0.0 {
                // grow 填充
                let split = (main - item.main) / item.grow;
                item_calc!(
                    self,
                    temp,
                    start,
                    end,
                    content_size,
                    cross_start,
                    cross_end,
                    normal,
                    pos,
                    split,
                    grow_calc,
                    grow_calc_reverse
                );
                return;
            } else if item.margin_auto > 0 {
                // margin_auto 填充
                let split = (main - item.main) / item.margin_auto as f32;
                item_calc!(
                    self,
                    temp,
                    start,
                    end,
                    content_size,
                    cross_start,
                    cross_end,
                    normal,
                    pos,
                    split,
                    margin_calc,
                    margin_calc_reverse
                );
                return;
            }
        } else if EPSILON < item.main - main {
            if item.shrink > 0.0 {
                // 表示需要收缩
                let split = (item.main - main) / item.shrink;
                item_calc!(
                    self,
                    temp,
                    start,
                    end,
                    content_size,
                    cross_start,
                    cross_end,
                    normal,
                    pos,
                    split,
                    shrink_calc,
                    shrink_calc_reverse
                );
                return;
            }
        }
        let (mut pos, split) = match temp.flex.justify_content {
            JustifyContent::FlexStart => {
                if normal {
                    (0.0, 0.0)
                } else {
                    (main, 0.0)
                }
            }
            JustifyContent::FlexEnd => {
                if normal {
                    (main - item.main, 0.0)
                } else {
                    (item.main, 0.0)
                }
            }
            JustifyContent::Center => {
                if normal {
                    ((main - item.main) / 2.0, 0.0)
                } else {
                    ((main + item.main) / 2.0, 0.0)
                }
            }
            JustifyContent::SpaceBetween => {
                if normal {
                    if item.count > 1 {
                        (0.0, (main - item.main) / (item.count - 1) as f32)
                    } else {
                        ((main - item.main) / 2.0, 0.0)
                    }
                } else {
                    if item.count > 1 {
                        (main, (main - item.main) / (item.count - 1) as f32)
                    } else {
                        ((main - item.main) / 2.0, 0.0)
                    }
                }
            }
            JustifyContent::SpaceAround => {
                let s = (main - item.main) / item.count as f32;
                if normal {
                    (s / 2.0, s)
                } else {
                    (main - s / 2.0, s)
                }
            }
            _ => {
                let s = (main - item.main) / (item.count + 1) as f32;
                if normal {
                    (s, s)
                } else {
                    (main - s, s)
                }
            }
        };
        out_any!(
            log::trace,
            "{:?}main calc: pos:{:?} split:{:?}",
            ppp(),
            pos,
            split
        );
        item_calc!(
            self,
            temp,
            start,
            end,
            content_size,
            cross_start,
            cross_end,
            normal,
            pos,
            split,
            main_calc,
            main_calc_reverse
        );
    }
}

// // 绝对定位下的布局，如果size=auto， 会先调用子节点的布局
// pub(crate) fn abs_layout<T>(
//     tree: &IdTree<u32>,
//     i_nodes: &mut impl IndexMut<usize, Output = INode>,
//     rect_style_map: &impl Index<usize, Output = RectStyle>,
//     other_style_map: &impl Index<usize, Output = OtherStyle>,
//     layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
//     notify: fn(&mut T, usize, &LayoutR),
//     notify_arg: &mut T,
//     id: usize,
//     is_text: bool,
//     child_head: usize,
//     child_tail: usize,
//     state: INodeState,
//     parent_size: (f32, f32),
//     flex: &ContainerStyle,
// ) {

// }

#[derive(PartialEq, Debug)]
enum LayoutSize {
    None,
    Size((f32, f32)),
}

// 设置布局结果
fn set_layout_result<T, K, L: LayoutR>(
    layout: &mut L,
    notify: fn(&mut T, K, &L),
    notify_arg: &mut T,
    id: K,
    pos: (f32, f32),
    size: (f32, f32),
    border: &Rect<Dimension>,
    padding: &Rect<Dimension>,
) -> LayoutSize {
    unsafe {
        PC += 1;
        PP = 0
    };
    let old_rect = layout.rect().clone();
    let old_w = old_rect.right
        - layout.border().right
        - layout.padding().right
        - (old_rect.left + layout.border().left + layout.padding().left);
    let old_h = old_rect.bottom
        - layout.border().bottom
        - layout.padding().bottom
        - (old_rect.top + layout.border().top + layout.padding().top);
    layout.set_rect(Rect {
        left: pos.0,
        top: pos.1,
        right: pos.0 + size.0,
        bottom: pos.1 + size.1,
    });
    layout.set_border(calc_border_padding(border, size.0, size.1));
    layout.set_padding(calc_border_padding(padding, size.0, size.1));
    notify(notify_arg, id, layout);
    layout.set_finish();
    let new_pos1 = (
        layout.rect().left + layout.border().left + layout.padding().left,
        layout.rect().top + layout.border().top + layout.padding().top,
    );
    let new_pos2 = (
        layout.rect().right - layout.border().right - layout.padding().right,
        layout.rect().bottom - layout.border().bottom - layout.padding().bottom,
    );
    let size = (new_pos2.0 - new_pos1.0, new_pos2.1 - new_pos1.1);
    if eq_f32(size.0, old_w) && eq_f32(size.1, old_h) {
        LayoutSize::None
    } else {
        LayoutSize::Size(size)
    }
}

const EPSILON: f32 = std::f32::EPSILON * 1024.0;
#[inline]
fn eq_f32(v1: f32, v2: f32) -> bool {
    v1 == v2 || ((v2 - v1).abs() <= EPSILON)
}

// 节点的兄弟节点
fn node_iter<K: Null + Copy + Clone>(direction: Direction, next: K, prev: K) -> K {
    if direction != Direction::RTL {
        next
    } else {
        // 处理倒排的情况
        prev
    }
}

fn grow_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main + info.grow * per;
    // if let Number::Defined(r) = info.max_main {
    // 	size = size.min(r);
    // }
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + size + info.margin_main_end.or_else(0.0);
    (start, size)
}
fn grow_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main + info.grow * per;
    // if let Number::Defined(r) = info.max_main {
    // 	size = size.min(r);
    // }
    let start = *pos - info.margin_main_end.or_else(0.0) - size;
    *pos = start - info.margin_main_start.or_else(0.0);
    (start, size)
}
fn margin_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos + info.margin_main_start.or_else(per);
    *pos = start + info.main + info.margin_main_end.or_else(per);
    (start, info.main)
}
fn margin_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos - info.margin_main_end.or_else(per) - info.main;
    *pos = start - info.margin_main_end.or_else(per);
    (start, info.main)
}
fn shrink_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main - info.shrink as f32 * per;
    // if let Number::Defined(r) = info.min_main {
    // 	size = size.max(r);
    // }
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + size + info.margin_main_end.or_else(0.0);
    (start, size)
}
fn shrink_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let size = info.main - info.shrink as f32 * per;
    // if let Number::Defined(r) = info.min_main {
    // 	size = size.max(r);
    // }
    let start = *pos - info.margin_main_end.or_else(0.0) - size;
    *pos = start - info.margin_main_start.or_else(0.0);
    (start, size)
}

fn min_max_calc(mut value: f32, min_value: Number, max_value: Number) -> f32 {
    if let Number::Defined(r) = min_value {
        value = value.max(r);
    }
    if let Number::Defined(r) = max_value {
        value = value.min(r);
    }
    value
}

fn max_calc(value: Number, max_value: Number) -> Number {
	if let (Number::Undefined, Number::Defined(_r)) = (value, max_value) {
		max_value
	} else {
		value
	}
}

fn main_calc<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + info.main + info.margin_main_end.or_else(0.0) + per;
    (start, info.main)
}
fn main_calc_reverse<K>(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos - info.margin_main_end.or_else(0.0) - info.main;
    *pos = start - info.margin_main_start.or_else(0.0) - per;
    (start, info.main)
}
// 返回位置和大小
fn cross_calc<K>(
    info: &RelNodeInfo<K>,
    start: f32,
    end: f32,
    align_items: AlignItems,
) -> (f32, f32) {
    out_any!(
        log::trace,
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
            _ => align_start(start, end, info), // 不支持baseline
        },
        AlignSelf::FlexStart => align_start(start, end, info),
        AlignSelf::FlexEnd => align_end(start, end, info),
        AlignSelf::Center => align_center(start, end, info),
        _ if info.cross_d.is_undefined() => align_stretch(start, end, info),
        _ => align_start(start, end, info), // 不支持baseline
    }
}
// 返回位置和大小
fn align_start<K>(start: f32, end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
	if let Number::Defined(r) = info.margin_cross_start {
		(start + r, info.cross)
	} else if let Number::Defined(r) = info.margin_cross_end {
		(end - r - info.cross, info.cross)
	} else {
		((start + end - info.cross) / 2.0, info.cross)
	}
}
// 返回位置和大小
fn align_end<K>(start: f32, end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
	if let Number::Defined(r) = info.margin_cross_end {
		(end - r - info.cross, info.cross)
	} else if let Number::Defined(r) = info.margin_cross_start {
		(start + r, info.cross)
	} else {
		((start + end - info.cross) / 2.0, info.cross)
	}
}
// 返回位置和大小
fn align_center<K>(start: f32, end: f32, info: &RelNodeInfo<K>) -> (f32, f32) {
	if let (Number::Defined(r), Number::Defined(rr)) = (info.margin_cross_start, info.margin_cross_end) {
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

// 获得计算区域(大小和位置)， 大小为None表示自动计算
fn calc_rect(
    start: Dimension,
    end: Dimension,
    size: Dimension,
    margin_start: Dimension,
    margin_end: Dimension,
    parent: f32,
    _children_abs: bool,
    align: isize,
) -> (Number, f32) {
	let calc_size = if let Dimension::Points(r) = size {
		r
	} else if let Dimension::Percent(r) = size {
		parent * r
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
					parent - rrr - margin_end.resolve_value(parent)
				} else if let Dimension::Percent(rrr) = end {
					parent - parent * rrr - margin_end.resolve_value(parent)
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
			return (Number::Undefined, margin_start.resolve_value(parent));
		};

		rr += margin_start.resolve_value(parent);
		rrr += margin_end.resolve_value(parent);
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
				return calc_margin(s, s + calc_size, calc_size, margin_start, margin_end, parent);
			} else if align > 0 {
				// 后对齐
				return (
					Number::Defined(calc_size),
					parent - margin_end.resolve_value(parent) - calc_size,
				);
			} else {
				// 前对齐
				return (Number::Defined(calc_size), margin_start.resolve_value(parent));
			}
		};
		return (
			Number::Defined(calc_size),
			parent - rrr - margin_end.resolve_value(parent) - calc_size,
		);
	};
	// size为Percent或Points、 start为Percent或Points
	return (
		Number::Defined(calc_size),
		calc_start + margin_start.resolve_value(parent),
	);
	
    // // 左右对齐
    // let rrr = match end {
    //     Dimension::Points(rrr) => rrr,
    //     Dimension::Percent(rrr) => parent * rrr,
    //     _ => {
    //         // 前对齐
    //         return (Number::Defined(r), rr + margin_start.resolve_value(parent));
    //     }
    // };
    // calc_margin(rr, parent - rrr, r, margin_start, margin_end, parent)
}

#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub struct ContainerStyle {
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
}
pub trait FlexLayoutCombine: FlexLayoutStyle {
    fn container_style(&self) -> ContainerStyle;
    fn margin(&self) -> Rect<Dimension>;
    fn padding(&self) -> Rect<Dimension>;
    fn position(&self) -> Rect<Dimension>;
    fn border(&self) -> Rect<Dimension>;
    fn calc_horizontal_content_size(&self, size: Number) -> Number;
    fn calc_vertical_content_size(&self, size: Number) -> Number;
}

impl<T: FlexLayoutStyle> FlexLayoutCombine for T {
    fn container_style(&self) -> ContainerStyle {
        ContainerStyle {
            flex_direction: self.flex_direction(),
            flex_wrap: self.flex_wrap(),
            justify_content: self.justify_content(),
            align_items: self.align_items(),
            align_content: self.align_content(),
        }
    }
    fn margin(&self) -> Rect<Dimension> {
        Rect {
            left: self.margin_left(),
            right: self.margin_right(),
            top: self.margin_top(),
            bottom: self.margin_bottom(),
        }
    }
    fn padding(&self) -> Rect<Dimension> {
        Rect {
            left: self.padding_left(),
            right: self.padding_right(),
            top: self.padding_top(),
            bottom: self.padding_bottom(),
        }
    }
    fn position(&self) -> Rect<Dimension> {
        Rect {
            left: self.position_left(),
            right: self.position_right(),
            top: self.position_top(),
            bottom: self.position_bottom(),
        }
    }
    fn border(&self) -> Rect<Dimension> {
        Rect {
            left: self.border_left(),
            right: self.border_right(),
            top: self.border_top(),
            bottom: self.border_bottom(),
        }
    }

    fn calc_horizontal_content_size(&self, size: Number) -> Number {
        calc_content_size(
            size,
            self.border_left(),
            self.border_right(),
            self.padding_right(),
            self.padding_right(),
        )
    }

    fn calc_vertical_content_size(&self, size: Number) -> Number {
        calc_content_size(
            size,
            self.border_top(),
            self.border_bottom(),
            self.padding_top(),
            self.padding_bottom(),
        )
    }
}

// 根据宽高获得内容宽高
fn calc_content_size(
    size: Number,
    b_start: Dimension,
    b_end: Dimension,
    p_start: Dimension,
    p_end: Dimension,
) -> Number {
	if let Number::Defined(r) = size {
		Number::Defined(
            r - b_start.resolve_value(r)
                - b_end.resolve_value(r)
                - p_start.resolve_value(r)
                - p_end.resolve_value(r),
        )
	} else {
		size
	}
}
// 根据内容宽高计算宽高
fn calc_size_from_content(
    mut points: f32,
    b_start: Dimension,
    b_end: Dimension,
    p_start: Dimension,
    p_end: Dimension,
) -> f32 {
    let mut p = 0.0;
    percent_calc(b_start, &mut points, &mut p);
    percent_calc(b_end, &mut points, &mut p);
    percent_calc(p_start, &mut points, &mut p);
    percent_calc(p_end, &mut points, &mut p);
    reverse_calc(points, p)
}
// 根据固定值和百分比反向计算大小
fn reverse_calc(points: f32, percent: f32) -> f32 {
    if percent >= 1.0 {
        // 防止百分比大于100%
        points
    } else {
        points / (1.0 - percent)
    }
}
fn percent_calc(d: Dimension, points: &mut f32, percent: &mut f32) -> bool {
	if let Dimension::Points(r) = d {
		*points += r;
	} else if let Dimension::Percent(r) = d {
		*percent += r;
	} else {
		return false;
	}
	true
}

// 已经确定了布局的区域， 需要计算布局中的border和padding
#[inline]
fn calc_border_padding(s: &Rect<Dimension>, w: f32, h: f32) -> Rect<f32> {
    Rect {
        left: s.left.resolve_value(w),
        right: s.right.resolve_value(w),
        top: s.top.resolve_value(h),
        bottom: s.bottom.resolve_value(h),
    }
}

// 计算margin, margin=Auto时自动填充剩余空间， 两边都Auto时平分剩余空间
fn calc_margin(
    mut start: f32,
    mut end: f32,
    size: f32,
    margin_start: Dimension,
    margin_end: Dimension,
    parent: f32,
) -> (Number, f32) {
	if let Dimension::Points(r) = margin_start {
		start += r;
		end = start + size;
	} else if let Dimension::Percent(r) = margin_start {
		start += r * parent;
		end = start + size;
	} else if let Dimension::Points(r) = margin_end {
		end -= r;
        start = end - size;
	} else if let Dimension::Percent(r) = margin_end {
		end -= r * parent;
        start = end - size;
	} else {
		out_any!(
			log::trace,
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

// 在flex计算的区域中 根据pos的位置进行偏移
fn calc_pos(position_start: Dimension, position_end: Dimension, parent: f32, pos: f32) -> f32 {
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
// 计算子节点的大小
fn calc_number(s: Dimension, parent: f32) -> Number {
	if let Dimension::Points(r) = s {
		Number::Defined(r)
	} else if let Dimension::Percent(r) = s {
		Number::Defined(parent * r)
	} else {
		Number::Undefined
	}
}

// 计算定位属性的节点的大小（margin）
fn calc_location_number(s: Dimension, parent: f32) -> Number {
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

fn calc_length(length: Number, min_length: Number) -> Number {
	if let (Number::Undefined, Number::Defined(_)) = (length, min_length) {
		min_length
	} else if let (Number::Defined(l1), Number::Defined(l2)) = (length, min_length) {
		if l1 > l2 {
			length
		} else {
			min_length
		}
	} else {
		length
	}
}
pub(crate) static mut PP: usize = 0;
pub(crate) static mut PC: usize = 0;
