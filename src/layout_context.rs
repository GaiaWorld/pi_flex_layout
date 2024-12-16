//! layout_context和layout, 是用来隔离计算和存储的， 以后将layout合并进layout_context中

// LayoutContext{

//     abs_layout // 绝对布局，一般从此开始计算
//             auto_children_layout // 没有定义宽高，则自动布局计算后，set_layout_result
//                     set_layout_result
//             set_layout // 有定义宽高，则直接set_layout

//     rel_layout // 不影响自身及所在容器的相对布局，一般从此开始计算
//             set_layout

//     auto_children_layout // 自动子节点布局， 如果布局本身不会改变自身节点，则fix为true，子节点就直接布局，否则需要缓存临时节点信息
//             do_layout //

//     temp_line_layout // 临时节点按行信息进行实际布局
//             abs_layout
//             layout_temp_node

//     set_layout //
//             set_layout_result
//             do_layout // 如果内容宽高有改变，则调用自身的子节点布局方法

//     children_layout // 子节点布局，生成临时节点并且统计行信息
//             abs_layout
//             auto_children_layout // 如果子节点没有定义宽高，则自动布局计算

//     do_layout // 遍历子节点，统计行信息，进行布局
//             text_layout // 如果是文字节点，则调用文字布局方法
//             children_layout // 非文字子节点的布局，临时节点及行信息统计
//             temp_line_layout

//     layout_temp_node // 布局临时节点
//             set_layout_result
//             temp_line_layout // 如果有子节点，则重建行信息后进行布局
//             set_layout // 确定大小的节点，需要进一步布局

// }

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use core::mem::replace;
use pi_null::Null;

use crate::calc::*;
use crate::geometry::*;
use crate::node_state::*;
use crate::number::*;
use crate::style::*;
use crate::traits::*;
use std::marker::PhantomData;
// use crate::grow_shrink::*;
use std::ops::IndexMut;

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
    // 每个子节点根据 justify-content align-items align-self，来计算main cross的位置和大小
    fn item_calc(
        &mut self,
        temp: &mut TempNode<K>,
        start: &mut usize,
        end: usize,
        content_box_size: Size<f32>,
        cross_start: f32,
        cross_end: f32,
        mut pos: f32,
        split: f32,
        calc: fn(info: &RelNodeInfo<K>, per: f32, pos: &mut f32) -> (f32, f32),
    ) {
        let mut baseline = Number::Undefined;
        if temp.row {
            while *start < end {
                let (info, temp_type) = unsafe { temp.rel_vec.get_unchecked_mut(*start) };
                *start += 1;
                let main = calc(info, split, &mut pos);
                let cross = cross_calc(
                    info,
                    cross_start,
                    cross_end,
                    temp.flex.align_items,
                    &mut baseline,
                );
                self.layout_temp_node(info.id, main, cross, temp_type, content_box_size);
            }
        } else {
            while *start < end {
                let (info, temp_type) = unsafe { temp.rel_vec.get_unchecked_mut(*start) };
                *start += 1;
                let main = calc(info, split, &mut pos);
                let cross = cross_calc(
                    info,
                    cross_start,
                    cross_end,
                    temp.flex.align_items,
                    &mut baseline,
                );
                self.layout_temp_node(info.id, cross, main, temp_type, content_box_size);
            }
        }
    }

    /// 绝对定位下的布局，如果size=auto， 会先调用子节点的布局
    pub(crate) fn abs_layout(
        &mut self,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        state: NodeState,
        containing_block_size: Size<f32>,
        flex: &ContainerStyle,
    ) {
        let style = &self.style.get(id);
        out_any!(
            println,
            // log::trace,
            "abs_layout, id:{:?}, containing_block: {:?}, style: {:?}, display: {:?}",
            id,
            containing_block_size,
            style,
            style.display()
        );
        if style.display() == Display::None {
            return;
        }
        // 判断是否宽高居中
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
        // // flex.flex_wrap == FlexWrap::WrapReverse会时的交叉轴布局反向
        // if flex.flex_wrap == FlexWrap::WrapReverse {
        //     a2 = -a2;
        // }
        // 根据行方向调整对齐方向
        let (walign, halign) = if flex.flex_direction.is_row() {
            (a1, a2)
        } else {
            (a2, a1)
        };
        let (min_width, max_width, min_height, max_height) = (
            calc_number(style.min_width(), containing_block_size.width),
            calc_number(style.max_width(), containing_block_size.width),
            calc_number(style.min_height(), containing_block_size.height),
            calc_number(style.max_height(), containing_block_size.height),
        );
        // 计算大小和位置
        let (w, x) = calc_rect(
            style.position_left(),
            style.position_right(),
            calc_length(
                calc_number(style.width(), containing_block_size.width),
                min_width,
                max_width,
            ),
            style.margin_left(),
            style.margin_right(),
            containing_block_size.width,
            containing_block_size.width,
            walign,
        );
        let (h, y) = calc_rect(
            style.position_top(),
            style.position_bottom(),
            calc_length(
                calc_number(style.height(), containing_block_size.height),
                min_height,
                max_height,
            ),
            style.margin_top(),
            style.margin_bottom(),
            containing_block_size.height,
            containing_block_size.width,
            halign,
        );

        out_any!(
            println,
            // log::trace,
            "abs_layout, id:{:?} size:{:?} walign: {:?}, halign: {:?} position:{:?}, margin: {:?}, flex_direction {:?}, w: {:?}, x: {:?}, h: {:?}, y: {:?}",
            id,
            (style.width(), style.height()),
			walign,
			halign,
            style.position(),
			style.margin(),
			flex.flex_direction,
			w,x,
			h,y
        );

        if w == Number::Undefined || h == Number::Undefined {
            // 根据子节点计算大小
            let direction = style.direction();
            let pos = style.position();
            let margin = style.margin();
            let border = style.border();
            let padding = style.padding();

            let mut cache = CalcContext::new(
                calc_gap_by_containing_block(&containing_block_size, &border).gap_size(),
                calc_gap_by_containing_block(&containing_block_size, &padding).gap_size(),
                style.container_style(),
                Size::new(
                    calc_length(w, min_width, max_width),
                    calc_length(h, min_height, max_height),
                ),
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let (size, _r) = self.auto_children_layout(
                &mut cache,
                true,
                id,
                is_text,
                child_head,
                child_tail,
                state.contains(NodeState::ChildrenIndex),
                direction,
            );
            out_any!(
                println,
                // log::trace,
                "calc_rect: id: {:?}, size:{:?}",
                id,
                size
            );
            // 再次计算区域
            let (w, x) = calc_rect(
                pos.left,
                pos.right,
                Number::Defined(size.width),
                margin.left,
                margin.right,
                containing_block_size.width,
                containing_block_size.width,
                walign,
            );
            let (h, y) = calc_rect(
                pos.top,
                pos.bottom,
                Number::Defined(size.height),
                margin.top,
                margin.bottom,
                containing_block_size.height,
                containing_block_size.width,
                halign,
            );

            let mut layout = self.layout_map.get_mut(id);
            // 设置布局的值
            set_layout_result(
                &mut layout,
                self.notify.clone(),
                self.notify_arg,
                id,
                containing_block_size,
                true,
                Rect::new(x, y, w.or_else(0.0), h.or_else(0.0)),
                &border,
                &padding,
            );
        } else {
            self.set_layout(
                id,
                is_text,
                child_head,
                child_tail,
                style.container_style(),
                style.direction(),
                style.border(),
                style.padding(),
                state,
                containing_block_size,
                true,
                Rect::new(x, y, w.or_else(0.0), h.or_else(0.0)),
            );
        };
    }

    /// 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先检查自身的布局数据，然后修改子节点的布局数据
    pub(crate) fn rel_layout(
        &mut self,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        state: NodeState,
        containing_block_size: Size<f32>,
    ) {
        let style = &self.style.get(id);
        out_any!(
            println,
            // log::trace,
            "rel_layout, id:{:?}, style: {:?}, display: {:?}",
            id,
            style,
            style.display()
        );
        if style.display() == Display::None {
            return;
        }
        let rect = self.layout_map.get_mut(id).rect().clone();
        self.set_layout(
            id,
            is_text,
            child_head,
            child_tail,
            style.container_style(),
            style.direction(),
            style.border(),
            style.padding(),
            state,
            containing_block_size,
            false,
            rect,
        );
    }
    /// 布局临时节点
    fn layout_temp_node(
        &mut self,
        id: K,
        width: (f32, f32),
        height: (f32, f32),
        temp: &mut TempNodeType<K>,
        containing_block_size: Size<f32>,
    ) {
        let i_node = &mut self.i_nodes[id];
        if let TempNodeType::CharIndex(r) = temp {
            // 文字布局
            let cnode = &mut i_node.text[*r];
            cnode.pos = Rect {
                left: width.0,
                top: height.0,
                right: width.0 + width.1, // TODO
                bottom: height.0 + height.1,
            };
            out_any!(
                println,
                // log::trace,
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
        let is_abs = i_node.state.contains(NodeState::Abs);
        let (child_head, child_tail) = self
            .tree
            .get_down(id)
            .map_or((K::null(), K::null()), |down| (down.head(), down.tail()));
        let state = i_node.state;
        i_node
            .state
            .set_false(NodeState::ChildrenDirty | NodeState::SelfDirty);

        let x = calc_pos(
            s.position_left(),
            s.position_right(),
            containing_block_size.width,
            width.0,
        );
        let y = calc_pos(
            s.position_top(),
            s.position_bottom(),
            containing_block_size.height,
            height.0,
        );
        // 设置布局的值
        if let TempNodeType::R(t) = temp {
            // 有Auto的节点需要父确定大小，然后自身的temp重计算及布局
            let mut layout = self.layout_map.get_mut(id);
            set_layout_result(
                &mut layout,
                self.notify,
                self.notify_arg,
                id,
                containing_block_size,
                is_abs,
                Rect::new(x, y, width.1, height.1),
                &border,
                &padding,
            );
            let padding_box_size = abs_containing_block_size(&layout);
            let content_box_size = rel_containing_block_size(&layout);
            let mc = t.main_cross(content_box_size.width, content_box_size.height);
            let line = t.reline(mc.0, mc.1);
            // 如果有临时缓存子节点数组
            self.temp_line_layout(
                t,
                padding_box_size,
                Size::new(mc.0, mc.1),
                mc.0,
                mc.1,
                &line,
            );
        } else if let TempNodeType::None = temp {
            // 确定大小的节点，需要进一步布局
            let is_text = i_node.text.len() > 0 && !state.contains(NodeState::VNode);
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
                containing_block_size,
                is_abs,
                Rect::new(x, y, width.1, height.1),
            );
        } else {
            // 有Auto的节点在计算阶段已经将自己的子节点都布局了，节点自身等待确定位置
            let mut layout = self.layout_map.get_mut(id);
            set_layout_result(
                &mut layout,
                self.notify,
                self.notify_arg,
                id,
                containing_block_size,
                is_abs,
                Rect::new(x, y, width.1, height.1),
                &border,
                &padding,
            );
        }
    }

    // 自动布局，计算宽高， 如果is_fix为false则返回Temp。宽度或高度auto、宽度或高度undefined的节点会进入此方法
    fn auto_children_layout(
        &mut self,
        cache: &mut CalcContext<K>,
        is_fix: bool, // 自身节点是否为固定大小,
        id: K,
        is_text: bool,
        child_head: K,
        child_tail: K,
        children_index: bool,
        direction: Direction,
    ) -> (Size<f32>, TempNodeType<K>) {
        out_any!(
            println,
            // log::trace,
            "{:?}auto_children_layout1: id:{:?} head:{:?} tail:{:?} is_notify:{:?}",
            ppp(),
            id,
            child_head,
            child_tail,
            is_fix
        );
        self.do_layout(
            cache,
            is_fix,
            id,
            is_text,
            child_head,
            child_tail,
            children_index,
            direction,
        );
        out_any!(
            println,
            // log::trace,
            "{:?}auto_children_layout2: id:{:?}, size:{:?}, is_row: {:?}, is_fix: {:?}",
            ppp(),
            id,
            (cache.main_value, cache.cross_value),
            cache.temp.row,
            is_fix
        );
        let (w, h) = cache.temp.main_cross(cache.main_value, cache.cross_value);
        (
            // 按照盒子模型， 返回宽高，该宽高包括了边框和空白
            Size::new(w, h) + cache.border_gap_size + cache.padding_gap_size,
            if is_fix {
                TempNodeType::AutoOk
            } else {
                // 则将布局的中间数组暂存下来
                TempNodeType::R(replace(&mut cache.temp, TempNode::default()))
            },
        )
    }
    /// 分文字和非文字情况，非文字则先统计行信息。is_notify为true时，则进行计算最终布局大小
    fn do_layout(
        &mut self,
        cache: &mut CalcContext<K>,
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
            println,
            // log::trace,
            "{:?}do_layout1, id:{:?} is_notify:{:?}, is_text: {:?}, child_head: {:?}, , child_tail: {:?}, children_index: {:?}, direction: {:?},",
            ppp(),
            id,
            is_notify,
            is_text, child_head, child_tail, children_index, direction
        );

        if is_text {
            let i_node = &mut self.i_nodes[id];
            let style = self.style.get(id);
            cache.text_layout(id, &mut i_node.text, &mut line, style.overflow_wrap());
        } else {
            self.children_layout(
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
            println,
            // log::trace,
            "{:?}do_layout2, id:{:?} line:{:?}, vec:{:?}, Cache:{:?}",
            ppp(),
            id,
            &line,
            &cache.temp.rel_vec,
            (cache.main, cache.cross, cache.main_value, cache.cross_value)
        );
        if children_index {
            // 从堆中添加到数组上
            while let Some(OrderSort(_, _, info, temp)) = cache.heap.pop() {
                cache.temp.rel_vec.push((info, temp))
            }
        }
        // 如果自动大小， 则计算实际大小
        if !cache.main.is_defined() {
            cache.main_value = line.main.max(cache.main_value);
        }
        if !cache.cross.is_defined() {
            // self.cross1 = line.cross + line.item.cross; ？？？
            cache.cross_value = line.cross.max(cache.cross_value);
        }
        // 记录节点的子节点的统计信息
        // let node = &tree[id];
        let i_node = &mut self.i_nodes[id];
        i_node.state.set_false(NodeState::default());
        i_node.state.set_true(cache.state);
        out_any!(
            println,
            // log::trace,
            "do_layout3: id:{:?}, main_cross:{:?}",
            id,
            (cache.main, cache.cross, cache.main_value, cache.cross_value)
        );

        // 根据is_notify决定是否继续计算
        if is_notify {
            let (w, h) = cache.temp.main_cross(cache.main_value, cache.cross_value);
            let size = Size::new(w, h);
            self.temp_line_layout(
                &mut cache.temp,
                size + cache.padding_gap_size,
                size,
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

    // 子节点布局，如果is_notify，并且子节点是绝对定位，则直接布局。 否则统计行信息
    fn children_layout(
        &mut self,
        cache: &mut CalcContext<K>,
        is_notify: bool,
        line: &mut LineInfo,
        mut child: K,
        children_index: bool,
        direction: Direction,
    ) {
        let padding_box_size = cache.min_size - cache.border_gap_size;
        let content_box_size = cache.min_size - cache.border_gap_size - cache.padding_gap_size;
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
            if i_node.state.contains(NodeState::Abs) {
                if i_node.state.contains(NodeState::SelfRect) {
                    // 绝对区域不需计算
                    child = node_iter(direction, next, prev);
                    continue;
                }
                cache.state.set_false(NodeState::ChildrenRect);
                let id = child;
                child = node_iter(direction, next, prev);
                let state = i_node.state;
                i_node
                    .state
                    .set_false(NodeState::ChildrenDirty | NodeState::SelfDirty);
                let is_text = i_node.text.len() > 0;
                if is_notify {
                    self.abs_layout(
                        id,
                        is_text,
                        child_head,
                        child_tail,
                        state,
                        padding_box_size,
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
                println,
            // log::trace,
                "children_layout1, id:{:?}, next: {:?}, style: {:?}, is_vnode: {:?}, is_notify: {:?}",
                child,
				next,
                &style,
				i_node.state.contains(NodeState::VNode),
				is_notify
            );
            if style.display() == Display::None {
                self.layout_map.get_mut(child); // 取布局结果的目的是为了在布局结果不存在的情况下插入默认值
                child = node_iter(direction, next, prev);
                continue;
            }
            if !i_node.state.contains(NodeState::SelfRect) {
                cache.state.set_false(NodeState::ChildrenRect);
            }
            cache.state.set_false(NodeState::ChildrenAbs);
            let id = child;
            child = node_iter(direction, next, prev);
            if i_node.state.contains(NodeState::VNode) {
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
                self.children_layout(cache, is_notify, line, child, children_index, direction);
                cache.vnode.push(id);
                if is_notify {
                    self.notify.clone()(self.notify_arg, id, &mut self.layout_map.get_mut(id));
                }
                continue;
            }
            let order = style.order();
            if order != 0 {
                cache.state.set_false(NodeState::ChildrenIndex);
            }
            if style.align_self() != AlignSelf::Auto {
                cache.state.set_false(NodeState::ChildrenNoAlignSelf);
            }
            // flex布局时， 如果子节点的宽高未定义，则根据子节点的布局进行计算。如果子节点的宽高为百分比，并且父节点对应宽高未定义，则为0
            let w = calc_number(style.width(), content_box_size.width);
            let h = calc_number(style.height(), content_box_size.height);
            let basis = style.flex_basis();
            out_any!(
                println,
            // log::trace,
            "children_layout2, id: {:?}, padding_box_size:{:?}, Cache:{:?}, is_row:{:?}, w:{:?}, h:{:?}, basis: {:?}", id, padding_box_size, (cache.min_size, cache.main, cache.cross), cache.temp.row, w, h, basis);
            let (mut main, cross) = cache.temp.main_cross(w, h);
            let (mut main_d, cross_d) = cache.temp.main_cross(style.width(), style.height());
            if basis.is_defined() {
                main = calc_number(
                    basis,
                    cache
                        .temp
                        .main_cross(content_box_size.width, content_box_size.height)
                        .0,
                );
                main_d = basis;
            }
            let (min_width, max_width, min_height, max_height) = (
                calc_number(style.min_width(), content_box_size.width),
                calc_number(style.max_width(), content_box_size.width),
                calc_number(style.min_height(), content_box_size.height),
                calc_number(style.max_height(), content_box_size.height),
            );

            let (max_main, max_cross) = cache.temp.main_cross(max_width, max_height);
            let (min_main, min_cross) = cache.temp.main_cross(min_width, min_height);
            // margin_main margin_cross 应该用content_box_size.width算
            let ((margin_main_start, margin_main_end), (margin_cross_start, margin_cross_end)) =
                cache.temp.main_cross(
                    (
                        calc_location_number(style.margin_left(), content_box_size.width),
                        calc_location_number(style.margin_right(), content_box_size.width),
                    ),
                    (
                        calc_location_number(style.margin_top(), content_box_size.width),
                        calc_location_number(style.margin_bottom(), content_box_size.width),
                    ),
                );
            let mut info = RelNodeInfo {
                id,
                grow: style.flex_grow(),
                shrink: style.flex_shrink(),
                main: min_max_calc(main.or_else(0.0), min_main, max_main),
                cross: min_max_calc(cross.or_else(0.0), min_cross, max_cross),
                margin_main: 0.0,
                margin_main_start,
                margin_main_end,
                margin_cross_start,
                margin_cross_end,
                align_self: style.align_self(),
                main_d,
                cross_d,
                line_start_margin_zero: i_node.state.contains(NodeState::LineStartMarginZero),
                breakline: i_node.state.contains(NodeState::BreakLine),
                min_main,
                max_main,
                main_result: 0.0,
                main_result_maybe_ok: false,
            };
            out_any!(
                println,
                // log::trace,
                "children_layout3,info:{:?}, ",
                &info
            );
            let temp = if main == Number::Undefined || cross == Number::Undefined {
                // 需要计算子节点大小
                let direction = style.direction();
                let border = style.border();
                let padding = style.padding();
                // 子节点大小是否不会改变， 如果不改变则直接布局
                let mut fix = true;
                // 主轴有3种情况后面可能会被改变大小
                if main_d.is_undefined() {
                    fix = style.flex_grow() == 0.0 // &&basis.is_undefined() 
                        && style.flex_shrink() == 0.0;
                }
                // 交叉轴有2种情况后面可能会被改变大小
                if fix && cross_d.is_undefined() {
                    fix = style.align_self() != AlignSelf::Stretch
                        && cache.temp.flex.align_items != AlignItems::Stretch;
                }
                out_any!(
                    println,
                    // log::trace,
                    "{:?}children_layout4: id:{:?} fix:{:?} size:{:?} next:{:?}",
                    ppp(),
                    id,
                    fix,
                    (w, h, main, cross),
                    child
                );
                let (w, h) = cache.temp.main_cross(main, cross);
                let children_index = i_node.state.contains(NodeState::ChildrenIndex);
                let is_text = i_node.text.len() > 0;
                let mut cache_new = CalcContext::new(
                    calc_gap_by_containing_block(&content_box_size, &border).gap_size(),
                    calc_gap_by_containing_block(&content_box_size, &padding).gap_size(),
                    style.container_style(),
                    Size::new(
                        calc_length(w, min_width, max_width),
                        calc_length(h, min_height, max_height),
                    ),
                    Size::new(min_width, min_height),
                    Size::new(max_width, max_height),
                );
                out_any!(
                    println,
                    // log::trace,
                    "children_layout5 cache_new: {:?}",
                    (
                        cache_new.min_size,
                        cache_new.main,
                        cache_new.main_value,
                        cache_new.cross,
                        cache_new.cross_value,
                        cache_new.main_line
                    )
                );
                // cache.main_line =
                // max_calc(w, max_width);
                // max_calc(h, max_height);
                let (size, r) = self.auto_children_layout(
                    &mut cache_new,
                    fix,
                    id,
                    is_text,
                    child_head,
                    child_tail,
                    children_index,
                    direction,
                );
                let mc = cache.temp.main_cross(size.width, size.height);
                info.main = mc.0;
                info.cross = mc.1;
                // info.main = min_max_calc(mc.0, min_main, max_main);
                // info.cross = min_max_calc(mc.1, min_cross, max_cross);
                r
            } else {
                // 确定大小的节点， TempType为None
                out_any!(
                    println,
                    // log::trace,
                    "children_layout6: id:{:?} size:{:?} next:{:?}",
                    id,
                    (main, cross),
                    child
                );
                TempNodeType::None
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

            if let Dimension::Percent(_r) = info.main_d {
                cache.temp.children_percent = true;
            } else if let Dimension::Percent(_r) = info.cross_d {
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
            out_any!(
                println,
                // log::trace,
                "children_layout7,line:{:?}, ",
                &line
            );
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
        border: SideGap<Dimension>,
        padding: SideGap<Dimension>,
        state: NodeState,
        containing_block_size: Size<f32>,
        is_abs: bool,
        rect: Rect<f32>,
    ) {
        out_any!(
            println,
            // log::trace,
            "{:?}set_layout: containing_block_size:{:?} id:{:?} head:{:?} tail:{:?} state:{:?}",
            ppp(),
            containing_block_size,
            id,
            child_head,
            child_tail,
            state
        );
        // 设置布局的值
        let mut layout = self.layout_map.get_mut(id);
        let r = if state.contains(NodeState::SelfDirty)
            || !eq_f32(layout.rect().left, rect.left)
            || !eq_f32(layout.rect().right, rect.right)
            || !eq_f32(layout.rect().top, rect.top)
            || !eq_f32(layout.rect().bottom, rect.bottom)
        {
            set_layout_result(
                &mut layout,
                self.notify,
                self.notify_arg,
                id,
                containing_block_size,
                is_abs,
                rect,
                &border,
                &padding,
            )
        } else {
            false
        };
        // 如果子节点不脏， 则检查r及子节点状态，判断是否要递归布局子节点
        if !state.contains(NodeState::ChildrenDirty) {
            if !r {
                return;
            }
            if state.contains(NodeState::ChildrenRect)
                && (state.contains(NodeState::ChildrenAbs)
                    || (state.contains(NodeState::ChildrenNoAlignSelf)
                        && (flex.flex_direction == FlexDirection::Row
                            || flex.flex_direction == FlexDirection::Column)
                        && flex.flex_wrap == FlexWrap::NoWrap
                        && flex.justify_content == JustifyContent::FlexStart
                        && flex.align_items == AlignItems::FlexStart))
            {
                // 节点的宽高变化不影响子节点的布局，还可进一步优化仅交叉轴大小变化
                return;
            }
        }
        let size = rect.size();
        // 宽高变动重新布局
        let mut cache = CalcContext::new(
            layout.border().gap_size(),
            layout.padding().gap_size(),
            flex,
            Size::new(Number::Defined(size.width), Number::Defined(size.height)),
            Size::new(Number::Defined(size.width), Number::Defined(size.height)),
            Size::new(Number::Undefined, Number::Undefined),
        );
        self.do_layout(
            &mut cache,
            true,
            id,
            is_text,
            child_head,
            child_tail,
            state.contains(NodeState::ChildrenIndex),
            direction,
        );
    }

    /// 临时节点按行信息进行实际布局
    fn temp_line_layout(
        &mut self,
        temp: &mut TempNode<K>,
        padding_box_size: Size<f32>,
        content_box_size: Size<f32>,
        main: f32,
        cross: f32,
        line: &LineInfo,
    ) {
        out_any!(
            println,
            // log::trace,
            "{:?}temp_line_layout: style:{:?} content_box_size:{:?} main_cross:{:?}, line:{:?}",
            ppp(),
            &temp.flex,
            content_box_size,
            (main, cross),
            line
        );
        // 处理abs_vec
        for e in temp.abs_vec.iter() {
            self.abs_layout(e.0, e.4, e.1, e.2, e.3, padding_box_size, &temp.flex);
        }
        let normal = !temp.flex.flex_direction.is_reverse();
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
                content_box_size,
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
                            content_box_size,
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
                        content_box_size,
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
                println,
                // log::trace,
                "temp_line_layout1, item: {:?}, split: {:?}, pos: {:?}",
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
                content_box_size,
                cross_start,
                cross_end,
                normal,
            );
        }
        out_any!(
            println,
            // log::trace,
            "temp_line_layout2, item: {:?}, split: {:?}, pos: {:?}, cross:{:?}",
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
            content_box_size,
            cross_start,
            cross_end,
            normal,
        );
    }

    /// 处理单行的节点布局
    fn temp_single_line(
        &mut self,
        temp: &mut TempNode<K>,
        main: f32,
        item: &LineItem,
        start: &mut usize,
        count: usize,
        content_box_size: Size<f32>,
        cross_start: f32,
        cross_end: f32,
        normal: bool,
    ) {
        if count == 0 {
            return;
        }
        out_any!(
            println,
            // log::trace,
            "{:?}temp_single_line1: normal:{:?} content_box_size:{:?}, cross:{:?} start_end:{:?} main:{:?}",
            ppp(),
            normal,
            content_box_size,
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
        let pos = if normal { 0.0 } else { main };
        // 浮点误差计算
        if main - item.main > EPSILON {
            // 表示需要放大
            if item.grow > 0.0 {
                // if item.grow_shrink_context.grow_weight > 0.0 {
                // grow 填充
                let split = (main - item.main) / item.grow;
                self.item_calc(
                    temp,
                    start,
                    end,
                    content_box_size,
                    cross_start,
                    cross_end,
                    pos,
                    split,
                    if normal { grow_calc } else { grow_calc_reverse },
                );
                return;
            } else if item.margin_auto > 0 {
                // margin_auto 填充
                let split = (main - item.main) / item.margin_auto as f32;
                self.item_calc(
                    temp,
                    start,
                    end,
                    content_box_size,
                    cross_start,
                    cross_end,
                    pos,
                    split,
                    if normal {
                        margin_calc
                    } else {
                        margin_calc_reverse
                    },
                );
                return;
            }
        } else if EPSILON < item.main - main {
            if item.shrink > 0.0 {
                // 表示需要收缩
                let split = (item.main - main) / item.shrink;
                self.item_calc(
                    temp,
                    start,
                    end,
                    content_box_size,
                    cross_start,
                    cross_end,
                    pos,
                    split,
                    if normal {
                        shrink_calc
                    } else {
                        shrink_calc_reverse
                    },
                );
                return;
            }
        }
        let (pos, split) = match temp.flex.justify_content {
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
            println,
            // log::trace,
            "{:?}temp_single_line2 calc: pos:{:?} split:{:?}",
            ppp(),
            pos,
            split
        );
        self.item_calc(
            temp,
            start,
            end,
            content_box_size,
            cross_start,
            cross_end,
            pos,
            split,
            if normal { main_calc } else { main_calc_reverse },
        );
    }
}
