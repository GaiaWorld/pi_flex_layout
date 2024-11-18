#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use crate::number::Number;

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignItems {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    SpaceBetween,
    SpaceAround,
}


#[derive(Copy, Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Inherit,
    LTR,
    RTL,
}


#[derive(Copy, Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Display {
    #[default]
    Flex,
    None,
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    pub(crate) fn is_row(self) -> bool {
        self == FlexDirection::Row || self == FlexDirection::RowReverse
    }

    pub(crate) fn is_column(self) -> bool {
        self == FlexDirection::Column || self == FlexDirection::ColumnReverse
    }

    pub(crate) fn is_reverse(self) -> bool {
        self == FlexDirection::RowReverse || self == FlexDirection::ColumnReverse
    }
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}


#[derive(Copy, Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}


#[derive(Copy, Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PositionType {
    #[default]
    Relative,
    Absolute,
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    #[default]
    Wrap,
    WrapReverse,
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum Dimension {
    #[default]
    Undefined,
    Auto,
    Points(f32),
    Percent(f32),
}

impl Dimension {
    pub(crate) fn resolve_value(self, parent: f32) -> f32 {
		if let Dimension::Points(points) = self {
			points
		} else if let Dimension::Percent(percent) = self {
			parent * percent
		} else {
			0.0
		}
    }

    pub(crate) fn is_defined(self) -> bool {
		if let Dimension::Points(_) = self {
			true
		} else if let Dimension::Percent(_) = self {
			true
		} else {
			false
		}
    }
    pub(crate) fn is_undefined(self) -> bool {
		if let Dimension::Points(_) = self {
			false
		} else if let Dimension::Percent(_) = self {
			false
		} else {
			true
		}
    }
    pub(crate) fn is_points(self) -> bool {
		if let Dimension::Points(_) = self {
			true
		} else {
			false
		}
    }
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


/// 用来设置是否应该在一个本来不能断开的字符串中插入换行符，以防止文本溢出其行向盒
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum OverflowWrap {
    #[default]
	Normal,
	Anywhere,
	BreakWord,
}