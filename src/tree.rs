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
/// TODO min_size max_size 仅作用在size上， 需要确认是否参与grow shrink的计算，

/// 浏览器版本的flex实现不合理的地方
/// 1、自动大小的容器，其大小受子节点大小计算的影响，flex-basis这个时候并没有参与计算，但浏览器版本行和列的实现不一致，列的情况下子节点的flex-basis会影响父容器的大小，行不会。
/// flex_basis_unconstraint_column
/// 2、自动计算主轴大小的容器，其折行属性应该为不折行，这样子节点顺序放置后，才好计算容器的主轴大小。浏览器版本就不是这么实现的
/// 3、如果A 包含 B，B包含C， A C 都有大小，B本身自动计算大小，这种情况下，浏览器的实现是B就不受A上的flex-basis grow shrink 影响，这样也不太合理。浏览器的计算似乎是从C先算B，然后不在二次计算B受的约束。 而正确的方式应该是先从A算B，发现B为自动大小，接着算C，反过来计算B的大小，然后受flex-basis影响，B大小变化后，再影响C的位置。
/// flex_basis_smaller_then_content_with_flex_grow_large_size

/// 注意事项：
/// 1. 根节点必须是区域（绝对定位， 绝对位置，绝对尺寸）
/// 2.
use pi_null::Null;
use std::ops::IndexMut;

// use map::vecmap::VecMap;

use crate::calc::*;
use crate::style::*;
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
        out_any!(
            log::trace,
            "set_display=====================, id:{:?}",
            id
        );
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
            out_any!(log::trace, "compute: {:?}", &dirty);
        }
        for (id, _layer) in dirty.iter() {
            // println_any!("layout======{:?}, {:?}", id, _layer);
            let (_node, i_node) = match self.0.tree.get_layer(*id) {
                Some(n) => (n, &mut self.0.i_nodes[*id]),
                _ => continue,
            };
            out_any!(log::trace, "    calc: {:?} children_dirty:{:?} self_dirty:{:?} children_abs:{:?} children_rect:{:?} children_no_align_self:{:?} children_index:{:?} vnode:{:?} abs:{:?} size_defined:{:?}, layer:{:?}", id, i_node.state.children_dirty(), i_node.state.self_dirty(), i_node.state.children_abs(), i_node.state.children_rect(), i_node.state.children_no_align_self(), i_node.state.children_index(), i_node.state.vnode(), i_node.state.abs(), i_node.state.size_defined(), _layer);
            let state = i_node.state;
            if !(state.self_dirty() || state.children_dirty()) {
                continue;
            }
            i_node.state.set_false(&INodeState::new(
                INodeStateType::ChildrenDirty as usize + INodeStateType::SelfDirty as usize,
            ));
            if i_node.state.vnode() {
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
            if state.abs() {
                let i_node = &self.0.i_nodes[*id];
                let mut parent = self.0.tree.get_up(*id).map_or(K::null(), |up| up.parent());
                while !parent.is_null() && self.0.i_nodes[parent].state.vnode() {
                    parent = self
                        .0
                        .tree
                        .get_up(parent)
                        .map_or(K::null(), |up| up.parent());
                }
                // 如果节点是绝对定位， 则重新计算自身的布局数据
                let (parent_size, flex) = if !i_node.state.self_rect() {
					#[cfg(debug_assertions)]
					if parent.is_null() {
						out_any!(panic, "node is root, but is not absolute rect, entity: {:?}, {:?}", id, _layer);
					}
                    // 如果节点自身不是绝对区域，则需要获得父容器的内容大小
                    let layout = self.0.layout_map.get_mut(parent);
                    let style = self.0.style.get(parent);
                    (get_content_size(&layout), style.container_style())
                } else {
                    (
                        (0.0, 0.0),
                        ContainerStyle {
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::NoWrap,
                            align_items: AlignItems::FlexStart,
                        },
                    )
                };
                self.0.abs_layout(
                    *id,
                    is_text,
                    child_head,
                    child_tail,
                    state,
                    parent_size,
                    &flex,
                );
            } else {
                // 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先重新计算自身的布局数据，然后修改子节点的布局数据
                self.0
                    .rel_layout(*id, is_text, child_head, child_tail, state);
            }
        }
        dirty.clear();
    }
    // 样式改变设置父节点
    fn set_parent(
        &mut self,
        dirty: &mut LayerDirty<K>,
        state: INodeState,
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
        if !state.abs() {
            i_node.state.children_abs_false();
        }
        if !state.self_rect() {
            i_node.state.children_rect_false();
        }
        if align_self != AlignSelf::Auto {
            i_node.state.children_no_align_self_false();
        }
        if order != 0 {
            i_node.state.children_index_false();
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
    pub fn set_rect(&mut self, dirty: &mut LayerDirty<K>, id: K, is_abs: bool, is_size: bool, style: &L) {
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
    pub fn mark_children_dirty(&mut self, dirty: &mut LayerDirty<K>, mut id: K) {
        while !id.is_null() {
            let i_node = &mut self.0.i_nodes[id];
			let layer = self.0.tree.get_layer(id).map_or(usize::null(), |l| l);

            out_any!(log::trace, "mark_children_dirty, id:{:?}, self_dirty:{:?}, size_defined:{:?}, abs:{:?}, vnode:{:?}, children_dirty: {:?}", id, i_node.state.self_dirty(),i_node.state.size_defined(), i_node.state.abs(), i_node.state.vnode(), i_node.state.children_dirty());

            if i_node.state.children_dirty() || layer.is_null() {
                break;
            }

            if !i_node.state.vnode() {
                i_node.state.children_dirty_true();
                if !i_node.state.self_dirty() {
                    dirty.mark(id, layer);
                }
            }

            if i_node.state.vnode() || !(i_node.state.size_defined() && i_node.state.abs()) {
                id = self.0.tree.get_up(id).map_or(K::null(), |up| up.parent())
            } else {
                break;
            }
        }
    }

    // 计算是否绝对区域
    fn calc_abs(style: &L, n: &mut INode) -> bool {
        if style.position_type() == PositionType::Absolute {
            n.state.abs_true();
            true
        } else {
            n.state.abs_false();
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
            n.state.self_rect_true();
            true
        } else {
            n.state.self_rect_false();
            false
        }
    }
    // 计算是否大小已经定义
    fn calc_size_defined(style: &L, n: &mut INode) -> bool {
        if style.width().is_defined() && style.height().is_defined() {
            n.state.size_defined_true();
            true
        } else {
            n.state.size_defined_false();
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
            i_node.state.self_dirty(),
            i_node.state.children_dirty()
        );
        if !i_node.state.vnode() && !i_node.state.self_dirty() {
            i_node.state.self_dirty_true();
            if !layer.is_null() {
                if !i_node.state.children_dirty() {
                    dirty.mark(id, layer);
                }
                if i_node.state.vnode() || !(i_node.state.size_defined() && i_node.state.abs()) {
                    return parent;
                }
            }
        }
        K::null()
    }
}
