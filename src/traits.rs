use pi_slotmap_tree::Down;
use pi_slotmap_tree::Up;

use crate::geometry::*;
use crate::number::*;
use crate::style::*;

pub trait LayoutR {
    // 取到布局属性
    fn rect(&self) -> &Rect<f32>;
    fn border(&self) -> &SideGap<f32>;
    fn padding(&self) -> &SideGap<f32>;
    // 是否为绝对定位，绝对定位节点的父包含块必须使用内边距区（padding box）
    fn absolute(&self) -> bool;

    // 设置布局属性
    fn set_rect(&mut self, v: Rect<f32>);
    fn set_border(&mut self, v: SideGap<f32>);
    fn set_padding(&mut self, v: SideGap<f32>);
    fn set_absolute(&mut self, b: bool);

    /// 布局属性设置完成会调用此方法
    fn set_finish(&mut self);
}

pub trait FlexLayoutStyle {
    fn width(&self) -> Dimension;
    fn height(&self) -> Dimension;

    fn margin_top(&self) -> Dimension;
    fn margin_right(&self) -> Dimension;
    fn margin_bottom(&self) -> Dimension;
    fn margin_left(&self) -> Dimension;

    fn padding_top(&self) -> Dimension;
    fn padding_right(&self) -> Dimension;
    fn padding_bottom(&self) -> Dimension;
    fn padding_left(&self) -> Dimension;

    fn position_top(&self) -> Dimension;
    fn position_right(&self) -> Dimension;
    fn position_bottom(&self) -> Dimension;
    fn position_left(&self) -> Dimension;

    fn border_top(&self) -> Dimension;
    fn border_right(&self) -> Dimension;
    fn border_bottom(&self) -> Dimension;
    fn border_left(&self) -> Dimension;

    fn display(&self) -> Display;
    fn position_type(&self) -> PositionType;
    fn direction(&self) -> Direction;

    fn flex_direction(&self) -> FlexDirection;
    fn flex_wrap(&self) -> FlexWrap;
    fn justify_content(&self) -> JustifyContent;
    fn align_items(&self) -> AlignItems;
    fn align_content(&self) -> AlignContent;
    fn row_gap(&self) -> f32;
    fn column_gap(&self) -> f32;

    fn order(&self) -> isize;
    fn flex_basis(&self) -> Dimension;
    fn flex_grow(&self) -> f32;
    fn flex_shrink(&self) -> f32;
    fn align_self(&self) -> AlignSelf;

    fn overflow(&self) -> Overflow;
    fn min_width(&self) -> Dimension;
    fn min_height(&self) -> Dimension;
    fn max_width(&self) -> Dimension;
    fn max_height(&self) -> Dimension;
    fn aspect_ratio(&self) -> Number;

    fn overflow_wrap(&self) -> OverflowWrap;
    fn auto_reduce(&self) -> bool;
}

pub trait Get<K> {
    type Target;
    fn get(&self, k: K) -> Self::Target;
}

pub trait GetMut<K> {
    type Target;
    fn get_mut(&mut self, k: K) -> Self::Target;
}

pub trait TreeStorage<K> {
    fn get_up(&self, id: K) -> Option<Up<K>>;
    fn get_down(&self, id: K) -> Option<Down<K>>;

    fn up(&self, id: K) -> Up<K>;
    fn down(&self, id: K) -> Down<K>;

    fn get_layer(&self, k: K) -> Option<usize>;
    fn layer(&self, k: K) -> usize;
}

pub trait FlexLayoutCombine: FlexLayoutStyle {
    fn container_style(&self) -> ContainerStyle;
    fn margin(&self) -> SideGap<Dimension>;
    fn border(&self) -> SideGap<Dimension>;
    fn padding(&self) -> SideGap<Dimension>;
    fn position(&self) -> Rect<Dimension>;
}

impl<T: FlexLayoutStyle> FlexLayoutCombine for T {
    fn container_style(&self) -> ContainerStyle {
        ContainerStyle {
            flex_direction: self.flex_direction(),
            flex_wrap: self.flex_wrap(),
            justify_content: self.justify_content(),
            align_items: self.align_items(),
            align_content: self.align_content(),
            row_gap: self.row_gap(),
            column_gap: self.column_gap(),
        }
    }
    fn margin(&self) -> SideGap<Dimension> {
        SideGap {
            left: self.margin_left(),
            right: self.margin_right(),
            top: self.margin_top(),
            bottom: self.margin_bottom(),
        }
    }
    fn padding(&self) -> SideGap<Dimension> {
        SideGap {
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
    fn border(&self) -> SideGap<Dimension> {
        SideGap {
            left: self.border_left(),
            right: self.border_right(),
            top: self.border_top(),
            bottom: self.border_bottom(),
        }
    }
}
