#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

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
    Grid,
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
    Fixed,
}

#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexWrap {
    #[default]
    NoWrap,
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

/// 用来设置是否应该在一个本来不能断开的字符串中插入换行符，以防止文本溢出其行向盒
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum OverflowWrap {
    #[default]
    Normal,
    Anywhere,
    BreakWord,
}

#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub struct ContainerStyle {
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub row_gap: f32,
    pub column_gap: f32,
}
