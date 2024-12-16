/// 布局支持虚拟节点， 虚拟节点下的子节点提到上面来参与布局，这样能很好的支持图文混排的布局
/// 维护层次脏和多种样式脏，分离不同的样式， 分别处理布局参数、几何数据和材质，对绝对定位做单独的优化处理，尽量最小化布局计算。
/// 计算流程： 先根据层次脏，从根到叶依次处理脏节点。 根据不同的脏和是否绝对定位及布局信息走不同的分支。
/// 如果节点的size=Auto, 在绝对定位下并且设置了right和bottom, 则left-right和top-bottom来决定大小. 否则表明是子节点决定大小.
/// 子节点计算大小后, 如果节点是flex并且是相对定位, 并且grow或shrink不为0, 则会再次计算大小
/// 设脏的情况: 1. 如果节点是绝对定位, 则只设自身脏. 2. 相对定位下, 如果属性是容器值, 则设节点自身脏, 否则设父节点脏. 如果脏节点的size=Auto, 则向上传播脏, 直到父节点为绝对定位或size!=Auto.
/// 计算时, 如果节点为绝对定位, 先检查size=Auto. 如果size=Auto, 则先根据left-right等来确定大小,否则需要根据子节点来计算大小. 如果size!=Auto, 则可能根据父节点大小先计算自身的layout, 然后计算子节点布局.
/// 计算时, 节点为相对定位时, size!=Auto. 根据自身的layout, 来计算子节点布局.
/// 计算子节点布局时, 第一次遍历子节点, 如果相对定位子节点的大小为Auto, 则判断是否脏, 如果脏, 则需要递归计算大小. 第二次遍历时， 如果节点有grow_shrink并且计算后大小有变化, 或者有Stretch, 则需要再次计算该子节点布局.
/// 计算子节点布局时, 节点内部保留缓存计算中间值.
/// 在盒子模型中， size position margin，三者中size优先级最高。 首先就是确定size，优先级依次是：1明确指定，2通过left-right能计算出来，3子节点撑大。 在position中left top不指定值的话默认为0, right bottom为自动计算的填充值，比如right=ParentContentWidth-left-margin_left-width--margin_right。而magin=Auto是自动填充left-right和width中间的值，如果没有明确指定left和right，magin=Auto最后的值就是margin=0
/// 注意： 为了不反复计算自动大小，如果父节点的主轴为自动大小，则flex-wrap自动为NoWrap。这个和浏览器的实现不一致！
/// TODO aspect_ratio 要求width 或 height 有一个为auto，如果都被指定，则aspect_ratio被忽略
/// TODO 支持min_size max_size
/// TODO 支持gap 支持负值
/// TODO 支持自动缩小
/// TODO 浏览器会将多余的空间视为 1 的值。这意味着当你为其中一个 Flex 项的 flex-grow 设置为 0.5 时，浏览器会将剩余空间的一半添加到该项的大小中。
/// TODO 绝对定位下，如果能设置自身的中心点或锚点（center_x center_y，默认为50%），并且支持用宽高百分比来设置，然后用location来设置自身的位置，包括对齐，优先级高于left right top bottom。
/// TODO 还是要支持简单版本的grid网格布局
/// TODO 支持position: fixed， 其包含块根节点上（也就是viewport），而且自己的zIndexContext也在根节点上， 该zIndexContext比根节点上绝对定位的子节点的zIndexContext要高，

/// 浏览器版本的flex实现不合理的地方
/// 1、自动大小的容器，其大小受子节点大小计算的影响，flex-basis这个时候并没有参与计算，但浏览器版本行和列的实现不一致，列的情况下子节点的flex-basis会影响父容器的大小，行不会。
/// 2、自动计算主轴大小的容器，其折行属性应该为不折行，这样子节点顺序放置后，才好计算容器的主轴大小。浏览器版本就不是这么实现的
/// 3、如果A 包含 B，B包含C， A C 都有大小，B本身自动计算大小，这种情况下，浏览器的实现是B就不受A上的flex-basis grow shrink 影响，这样也不太合理。浏览器的计算似乎是从C先算B，然后不在二次计算B受的约束。 而正确的方式应该是先从A算B，发现B为自动大小，接着算C，反过来计算B的大小，然后受flex-basis影响，B大小变化后，再影响C的位置。
/// flex_basis_smaller_then_content_with_flex_grow_large_size

/// 注意事项：
/// 1. 根节点必须是区域（绝对定位， 绝对位置，绝对尺寸）
use pi_null::Null;
use std::ops::IndexMut;

// use map::vecmap::VecMap;

use crate::calc::*;
use crate::geometry::*;
use crate::layout_context::*;
use crate::node_state::NodeState;
use crate::style::*;
use crate::traits::*;
use pi_dirty::*;

pub struct Layout<'a, K: Null + Clone + Copy, S, T, L, I, R, LI: Get<K, Target = L>, LR: LayoutR>(
    pub LayoutContext<'a, K, S, T, L, I, R, LI, LR>,
);

impl<'a, K, S, T, L, I, R, LI, LR> Layout<'a, K, S, T, L, I, R, LI, LR>
where
    K: Null + Clone + Copy + Eq + PartialEq,
    S: TreeStorage<K>,
    L: FlexLayoutCombine,
    LI: Get<K, Target = L>,
    LR: LayoutR,
    I: IndexMut<K, Output = INode>,
    R: GetMut<K, Target = LR>,
{
    pub fn set_display(&mut self, id: K, dirty: &mut LayerDirty<K>, style: &L) {
        out_any!(log::trace, "set_display=====================, id:{:?}", id);
        let (layer, parent) = (
            self.0.tree.get_layer(id).map_or(usize::null(), |l| l),
            self.0.tree.get_up(id).map_or(K::null(), |up| up.parent()),
        );
        let i_node = &mut self.0.i_nodes[id];
        let state = i_node.state;
        if style.display() != Display::None {
            Self::calc_rect(style, i_node);
            Self::calc_abs(style, i_node);
            Self::calc_size_defined(&style, i_node);
            Self::set_self_dirty(dirty, id, parent, layer, i_node);
            self.set_parent(
                dirty,
                state,
                parent,
                true,
                style.align_self(),
                style.order(),
            );
        } else if self.0.tree.layer(id) > 0 {
            self.mark_children_dirty(dirty, parent);
        }
    }

    pub fn compute(&mut self, dirty: &mut LayerDirty<K>) {
        if dirty.count() > 0 {
            out_any!(log::trace, "compute, {:?}", &dirty);
        }
        for (id, _layer) in dirty.iter() {
            // println_any!("layout======{:?}, {:?}", id, _layer);
            let (_node, i_node) = match self.0.tree.get_layer(*id) {
                Some(n) => (n, &mut self.0.i_nodes[*id]),
                _ => continue,
            };
            out_any!(
                log::trace,
                " compute1, id: {:?} node:{:?}, layer:{:?}",
                id,
                i_node.state,
                _layer
            );
            let state = i_node.state;
            if !(state.contains(NodeState::SelfDirty) || state.contains(NodeState::ChildrenDirty)) {
                continue;
            }
            i_node
                .state
                .set_false(NodeState::ChildrenDirty | NodeState::SelfDirty);
            if i_node.state.contains(NodeState::VNode) {
                // 不在树上或虚拟节点
                continue;
            }
            let (child_head, child_tail) = self
                .0
                .tree
                .get_down(*id)
                .map_or((K::null(), K::null()), |down| (down.head, down.tail));

            unsafe {
                PC = 0;
                PP = 0
            };
            let is_text = i_node.text.len() > 0;
            // 忽略虚拟父节点， 找到包含块元素（一般是父节点）
            let mut parent = self.0.tree.get_up(*id).map_or(K::null(), |up| up.parent());
            while !parent.is_null() && self.0.i_nodes[parent].state.contains(NodeState::VNode) {
                parent = self
                    .0
                    .tree
                    .get_up(parent)
                    .map_or(K::null(), |up| up.parent());
            }
            if parent.is_null() {
                // 如果父容器为空
                let flex = ContainerStyle {
                    justify_content: JustifyContent::FlexStart,
                    align_content: AlignContent::FlexStart,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::NoWrap,
                    align_items: AlignItems::FlexStart,
                    row_gap: 0.0,
                    column_gap: 0.0,
                };
                self.0.abs_layout(
                    *id,
                    is_text,
                    child_head,
                    child_tail,
                    state,
                    Size::default(),
                    &flex,
                );
            } else if state.contains(NodeState::Abs) && state.contains(NodeState::SelfRect) {
                // 绝对定位且只需要计算自身大小的节点
                let style = self.0.style.get(parent);
                self.0.abs_layout(
                    *id,
                    is_text,
                    child_head,
                    child_tail,
                    state,
                    Size::default(),
                    &style.container_style(),
                );
            } else if state.contains(NodeState::Abs) {
                // 如果节点是绝对定位， 则重新计算自身的布局数据
                let layout = self.0.layout_map.get_mut(parent);
                let style = self.0.style.get(parent);
                self.0.abs_layout(
                    *id,
                    is_text,
                    child_head,
                    child_tail,
                    state,
                    abs_containing_block_size(&layout),
                    &style.container_style(),
                );
            } else {
                // 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先重新计算自身的布局数据，然后修改子节点的布局数据
                let layout = self.0.layout_map.get_mut(parent);
                self.0.rel_layout(
                    *id,
                    is_text,
                    child_head,
                    child_tail,
                    state,
                    rel_containing_block_size(&layout),
                );
            }
        }
        dirty.clear();
    }

    // 样式改变设置父节点
    fn set_parent(
        &mut self,
        dirty: &mut LayerDirty<K>,
        state: NodeState,
        parent: K,
        mark: bool,
        align_self: AlignSelf,
        order: isize,
    ) {
        if parent.is_null() {
            return;
        }
        let layer = self.0.tree.get_layer(parent).map_or(usize::null(), |l| l);
        let i_node = &mut self.0.i_nodes[parent];
        if !state.contains(NodeState::Abs) {
            i_node.state.set_false(NodeState::ChildrenAbs);
        }
        if !state.contains(NodeState::SelfRect) {
            i_node.state.set_false(NodeState::ChildrenRect);
        }
        if align_self != AlignSelf::Auto {
            i_node.state.set_false(NodeState::ChildrenNoAlignSelf);
        }
        if order != 0 {
            i_node.state.set_false(NodeState::ChildrenIndex);
        }
        if mark && !layer.is_null() {
            self.mark_children_dirty(dirty, parent)
        }
    }
    // 设置自身样式， 设自身脏，如果节点是size=auto并且不是绝对定位, 则继续设置其父节点ChildrenDirty脏
    pub fn set_self_style(&mut self, id: K, dirty: &mut LayerDirty<K>, style: &L) {
        if style.display() == Display::None {
            // 如果是隐藏
            return;
        }
        out_any!(
            log::trace,
            "set_self_style=====================, id:{:?}",
            id
        );
        let (layer, parent) = (
            self.0.tree.get_layer(id).map_or(usize::null(), |l| l),
            self.0.tree.get_up(id).map_or(K::null(), |up| up.parent()),
        );
        let i_node = &mut self.0.i_nodes[id];
        let parent = Self::set_self_dirty(dirty, id, parent, layer, i_node);
        if !parent.is_null() {
            self.mark_children_dirty(dirty, parent)
        }
    }

    // 设置会影响子节点布局的样式， 设children_dirty脏，如果节点是size=auto并且不是绝对定位, 则继续设置其父节点ChildrenDirty脏
    pub fn set_children_style(&mut self, dirty: &mut LayerDirty<K>, id: K, style: &L) {
        if style.display() == Display::None {
            // 如果是隐藏
            return;
        }
        out_any!(
            log::trace,
            "set_children_style=====================, id:{:?}",
            id
        );
        self.mark_children_dirty(dirty, id)
    }
    // 设置一般样式， 设父节点脏
    pub fn set_normal_style(&mut self, dirty: &mut LayerDirty<K>, id: K, style: &L) {
        if style.display() == Display::None {
            // 如果是隐藏
            return;
        }
        let parent = self.0.tree.get_up(id).map_or(K::null(), |up| up.parent());
        let i_node = &self.0.i_nodes[id];
        let state = i_node.state;
        out_any!(
            log::trace,
            "set_normal_style=====================, id:{:?} state:{:?}",
            id,
            i_node.state
        );
        self.set_parent(
            dirty,
            state,
            parent,
            true,
            style.align_self(),
            style.order(),
        );
    }
    // 设置区域 pos margin size
    pub fn set_rect(
        &mut self,
        dirty: &mut LayerDirty<K>,
        id: K,
        is_abs: bool,
        is_size: bool,
        style: &L,
    ) {
        if style.display() == Display::None {
            // 如果是隐藏
            return;
        }
        let (layer, parent) = (
            self.0.tree.get_layer(id).map_or(usize::null(), |l| l),
            self.0.tree.get_up(id).map_or(K::null(), |up| up.parent()),
        );
        let i_node = &mut self.0.i_nodes[id];
        if is_abs {
            Self::calc_abs(style, i_node);
        }
        if is_size {
            Self::calc_size_defined(style, i_node);
        }

        Self::set_self_dirty(dirty, id, parent, layer, i_node);
        let _is_rect = Self::calc_rect(&style, i_node);
        // 如果是绝对定位，则仅设置自身脏
        let mark = if style.position_type() == PositionType::Absolute {
            false
        } else {
            true
        };
        out_any!(
            log::trace,
            "set rect dirty=====================, id:{:?} state:{:?}",
            id,
            i_node.state
        );
        let state = i_node.state;
        self.set_parent(
            dirty,
            state,
            parent,
            mark,
            style.align_self(),
            style.order(),
        );
    }

    // // 设置节点脏, 如果节点是size=auto并且不是绝对定位, 则返回父节点id，需要继续设置其父节点脏
    // fn set_children_dirty(dirty: &mut LayerDirty, id: usize, n: &Node, i_node: &mut INode) -> usize {
    // 	if !i_node.state.children_dirty() {
    // 		i_node.state.children_dirty_true();
    // 		if n.layer() > 0 {
    // 			if !i_node.state.self_dirty() {
    // 				dirty.mark(id, n.layer());
    // 			}
    // 			if i_node.state.vnode() || !(i_node.state.size_defined() || i_node.state.abs()) {
    // 				return n.parent();
    // 			}
    // 		}
    // 	}
    //     0
    // }
    // 设置节点children_dirty脏, 如果节点是size=auto并且不是绝对定位,也不是虚拟节点, 则继续设置其父节点children_dirty脏
    fn mark_children_dirty(&mut self, dirty: &mut LayerDirty<K>, mut id: K) {
        while !id.is_null() {
            let i_node = &mut self.0.i_nodes[id];
            let layer = self.0.tree.get_layer(id).map_or(usize::null(), |l| l);

            out_any!(
                log::trace,
                "mark_children_dirty, id:{:?}, state:{:?}",
                id,
                i_node.state
            );

            if i_node.state.contains(NodeState::ChildrenDirty) || layer.is_null() {
                break;
            }

            if !i_node.state.contains(NodeState::VNode) {
                i_node.state.set_true(NodeState::ChildrenDirty);
                if !i_node.state.contains(NodeState::SelfDirty) {
                    dirty.mark(id, layer);
                }
            }

            if i_node.state.contains(NodeState::VNode)
                || !(i_node.state.contains(NodeState::SizeDefined)
                    && i_node.state.contains(NodeState::Abs))
            {
                id = self.0.tree.get_up(id).map_or(K::null(), |up| up.parent())
            } else {
                break;
            }
        }
    }

    // 计算是否绝对区域
    fn calc_abs(style: &L, n: &mut INode) -> bool {
        if style.position_type() == PositionType::Absolute {
            n.state.set_true(NodeState::Abs);
            true
        } else {
            n.state.set_false(NodeState::Abs);
            false
        }
    }
    // 计算是否绝对区域
    fn calc_rect(style: &L, n: &mut INode) -> bool {
        if style.position_left().is_points()
            && style.position_top().is_points()
            && style.margin_left().is_points()
            && style.margin_top().is_points()
            && style.width().is_points()
            && style.height().is_points()
        {
            n.state.set_true(NodeState::SelfRect);
            true
        } else {
            n.state.set_false(NodeState::SelfRect);
            false
        }
    }
    // 计算是否大小已经定义
    fn calc_size_defined(style: &L, n: &mut INode) -> bool {
        if style.width().is_defined() && style.height().is_defined() {
            n.state.set_true(NodeState::SizeDefined);
            true
        } else {
            n.state.set_false(NodeState::SizeDefined);
            false
        }
    }
    // 设置节点自身脏, 如果节点是size=auto并且不是绝对定位, 则返回父节点id，需要继续设置其父节点脏
    fn set_self_dirty(
        dirty: &mut LayerDirty<K>,
        id: K,
        parent: K,
        layer: usize,
        i_node: &mut INode,
    ) -> K {
        out_any!(
            log::trace,
            "set_self_dirty, id: {:?}, self_dirty:{:?}, children_dirty:{:?}",
            id,
            i_node.state.contains(NodeState::SelfDirty),
            i_node.state.contains(NodeState::ChildrenDirty)
        );
        if !i_node.state.contains(NodeState::VNode) && !i_node.state.contains(NodeState::SelfDirty)
        {
            i_node.state.set_true(NodeState::SelfDirty);
            if !layer.is_null() {
                if !i_node.state.contains(NodeState::ChildrenDirty) {
                    dirty.mark(id, layer);
                }
                if i_node.state.contains(NodeState::VNode)
                    || !(i_node.state.contains(NodeState::SizeDefined)
                        && i_node.state.contains(NodeState::Abs))
                {
                    return parent;
                }
            }
        }
        K::null()
    }
}
