#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
use pi_enum_default_macro::EnumDefault;

use crate::geometry::{Rect, Size};
use crate::number::Number;

#[derive(EnumDefault, Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(EnumDefault, Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

impl Default for AlignContent {
    fn default() -> AlignContent {
        AlignContent::Stretch
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Inherit,
    LTR,
    RTL,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Inherit
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Display {
    Flex,
    None,
}

impl Default for Display {
    fn default() -> Display {
        Display::Flex
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> FlexDirection {
        FlexDirection::Row
    }
}

// impl FlexDirection {
//     pub(crate) fn is_row(self) -> bool {
//         self == FlexDirection::Row || self == FlexDirection::RowReverse
//     }

//     pub(crate) fn is_column(self) -> bool {
//         self == FlexDirection::Column || self == FlexDirection::ColumnReverse
//     }

//     pub(crate) fn is_reverse(self) -> bool {
//         self == FlexDirection::RowReverse || self == FlexDirection::ColumnReverse
//     }
// }

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> JustifyContent {
        JustifyContent::FlexStart
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
}

impl Default for Overflow {
    fn default() -> Overflow {
        Overflow::Visible
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PositionType {
    Relative,
    Absolute,
}

impl Default for PositionType {
    fn default() -> PositionType {
        PositionType::Relative
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl Default for FlexWrap {
    fn default() -> FlexWrap {
        FlexWrap::Wrap
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize, EnumDefault)]
pub enum Dimension {
    Undefined,
    Auto,
    Points(f32),
    Percent(f32),
}

// impl Default for Dimension {
//     fn default() -> Dimension {
//         Dimension::Points(0.0)
//     }
// }

impl Dimension {
    pub(crate) fn resolve_value(self, parent: f32) -> f32 {
		if let Dimension::Points(points) = self {
			points
		} else if let Dimension::Percent(percent) = self {
			parent * percent
		} else {
			0.0
		}

        // match self {
        //     Dimension::Points(points) => points,
        //     Dimension::Percent(percent) => parent * percent,
        //     _ => 0.0,
        // }
    }
    // pub(crate) fn resolve(self, parent_width: Number) -> Number {
    //     match self {
    //         Dimension::Points(points) => Number::Defined(points),
    //         Dimension::Percent(percent) => parent_width * percent,
    //         _ => Number::Undefined,
    //     }
    // }

    pub(crate) fn is_defined(self) -> bool {
		if let Dimension::Points(_) = self {
			true
		} else if let Dimension::Percent(_) = self {
			true
		} else {
			false
		}
        // match self {
        //     Dimension::Points(_) => true,
        //     Dimension::Percent(_) => true,
        //     _ => false,
        // }
    }
    pub(crate) fn is_undefined(self) -> bool {
		if let Dimension::Points(_) = self {
			false
		} else if let Dimension::Percent(_) = self {
			false
		} else {
			true
		}

        // match self {
        //     Dimension::Points(_) => false,
        //     Dimension::Percent(_) => false,
        //     _ => true,
        // }
    }
    pub(crate) fn is_points(self) -> bool {
		if let Dimension::Points(_) = self {
			true
		} else {
			false
		}
        // match self {
        //     Dimension::Points(_) => true,
        //     _ => false,
        // }
    }
}

impl Default for Rect<Dimension> {
    fn default() -> Rect<Dimension> {
        Rect {
            left: Default::default(),
            right: Default::default(),
            top: Default::default(),
            bottom: Default::default(),
        }
    }
}

impl Default for Size<Dimension> {
    fn default() -> Size<Dimension> {
        Size {
            width: Dimension::Undefined,
            height: Dimension::Undefined,
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
}


/// 用来设置是否应该在一个本来不能断开的字符串中插入换行符，以防止文本溢出其行向盒
#[derive(Debug, Clone, Copy, EnumDefault, PartialEq, Serialize, Deserialize)]
pub enum OverflowWrap {
	Normal,
	Anywhere,
	BreakWord,
}
// ContainerStyle {
// 	flex_direction: self.
// }

// flex_direction: s.flex_direction,
// flex_wrap: s.flex_wrap,
// justify_content: s.justify_content,
// align_items: s.align_items,
// align_content: s.align_content,

// #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
// pub struct RectStyle {
//     pub margin: Rect<Dimension>,
//     pub size: Size<Dimension>,
// }

// impl Default for RectStyle {
//     fn default() -> RectStyle {
//         RectStyle {
//             margin: Default::default(), // dom默认为undefined， 性能考虑，这里默认0.0
//             size: Default::default(),
//         }
//     }
// }

// #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
// pub struct OtherStyle {
//     pub display: Display,
//     pub position_type: PositionType,
//     pub direction: Direction,

//     pub flex_direction: FlexDirection,
//     pub flex_wrap: FlexWrap,
//     pub justify_content: JustifyContent,
//     pub align_items: AlignItems,
//     pub align_content: AlignContent,

//     pub order: isize,
//     pub flex_basis: Dimension,
//     pub flex_grow: f32,
//     pub flex_shrink: f32,
//     pub align_self: AlignSelf,

//     pub overflow: Overflow,
//     pub position: Rect<Dimension>,
//     pub padding: Rect<Dimension>,
//     pub border: Rect<Dimension>,
//     pub min_size: Size<Dimension>,
//     pub max_size: Size<Dimension>,
//     pub aspect_ratio: Number,

// }

// impl Default for OtherStyle {
//     fn default() -> OtherStyle {
//         OtherStyle {
//             display: Default::default(),
//             position_type: Default::default(),
//             direction: Default::default(),
//             flex_direction: Default::default(),
//             flex_wrap: Default::default(),
//             overflow: Default::default(),
//             align_items: Default::default(), // dom默认为stretch， 性能考虑，这里默认flex_start
//             align_self: Default::default(),
//             align_content: Default::default(),
//             justify_content: Default::default(),
//             position: Default::default(),
//             padding: Default::default(),
//             border: Default::default(),
//             flex_grow: 0.0,
//             flex_shrink: 0.0,  // dom默认为1.0， 性能考虑，这里默认0.0
//             order: 0,
//             flex_basis: Dimension::Auto,
//             min_size: Default::default(),
//             max_size: Default::default(),
//             aspect_ratio: Default::default(),
//         }
//     }
// }

// impl OtherStyle {
//     // pub(crate) fn min_main_size(&self, direction: FlexDirection) -> Dimension {
//     //     match direction {
//     //         FlexDirection::Row | FlexDirection::RowReverse => self.min_size.width,
//     //         FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.height,
//     //     }
//     // }

//     // pub(crate) fn max_main_size(&self, direction: FlexDirection) -> Dimension {
//     //     match direction {
//     //         FlexDirection::Row | FlexDirection::RowReverse => self.max_size.width,
//     //         FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.height,
//     //     }
//     // }

//     // pub(crate) fn min_cross_size(&self, direction: FlexDirection) -> Dimension {
//     //     match direction {
//     //         FlexDirection::Row | FlexDirection::RowReverse => self.min_size.height,
//     //         FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.width,
//     //     }
//     // }

//     // pub(crate) fn max_cross_size(&self, direction: FlexDirection) -> Dimension {
//     //     match direction {
//     //         FlexDirection::Row | FlexDirection::RowReverse => self.max_size.height,
//     //         FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.width,
//     //     }
//     // }

//     // pub(crate) fn align_self(&self, parent: &OtherStyle) -> AlignSelf {
//     //     if self.align_self == AlignSelf::Auto {
//     //         match parent.align_items {
//     //             AlignItems::FlexStart => AlignSelf::FlexStart,
//     //             AlignItems::FlexEnd => AlignSelf::FlexEnd,
//     //             AlignItems::Center => AlignSelf::Center,
//     //             AlignItems::Baseline => AlignSelf::Baseline,
//     //             AlignItems::Stretch => AlignSelf::Stretch,
//     //         }
//     //     } else {
//     //         self.align_self
//     //     }
//     // }
// }

// #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
// pub struct Style {
//     pub display: Display,
//     pub position_type: PositionType,
//     pub direction: Direction,

//     pub flex_direction: FlexDirection,
//     pub flex_wrap: FlexWrap,
//     pub justify_content: JustifyContent,
//     pub align_items: AlignItems,
//     pub align_content: AlignContent,

//     pub order: isize,
//     pub flex_basis: Dimension,
//     pub flex_grow: f32,
//     pub flex_shrink: f32,
//     pub align_self: AlignSelf,

//     pub overflow: Overflow,
//     pub position: Rect<Dimension>,
//     pub margin: Rect<Dimension>,
//     pub padding: Rect<Dimension>,
//     pub border: Rect<Dimension>,
//     pub size: Size<Dimension>,
//     pub min_size: Size<Dimension>,
//     pub max_size: Size<Dimension>,
// 	pub aspect_ratio: Number,
// 	pub line_start_margin: Number, // 行首的margin_start
// }

// impl Default for Style {
//     fn default() -> Style {
//         Style {
//             display: Default::default(),
//             position_type: Default::default(),
//             direction: Default::default(),
//             flex_direction: Default::default(),
//             flex_wrap: Default::default(),
//             overflow: Default::default(),
//             align_items: Default::default(), // dom默认为stretch， 性能考虑，这里默认flex_start
//             align_self: Default::default(),
//             align_content: Default::default(),
//             justify_content: Default::default(),
//             position: Default::default(),
//             margin: Default::default(), // dom默认为undefined， 性能考虑，这里默认0.0
//             padding: Default::default(),
//             border: Default::default(),
//             flex_grow: 0.0,
//             flex_shrink: 0.0,  // dom默认为1.0， 性能考虑，这里默认0.0
//             order: 0,
//             flex_basis: Dimension::Auto,
//             size: Default::default(),
//             min_size: Default::default(),
//             max_size: Default::default(),
// 			aspect_ratio: Default::default(),
// 			line_start_margin: Default::default(),
//         }
//     }
// }

// impl Style {
//     pub(crate) fn min_main_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.min_size.width,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.height,
//         }
//     }

//     pub(crate) fn max_main_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.max_size.width,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.height,
//         }
//     }

//     pub(crate) fn main_margin_start(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.start,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.top,
//         }
//     }

//     pub(crate) fn main_margin_end(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.end,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.bottom,
//         }
//     }

//     pub(crate) fn cross_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.size.height,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.size.width,
//         }
//     }

//     pub(crate) fn min_cross_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.min_size.height,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.width,
//         }
//     }

//     pub(crate) fn max_cross_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.max_size.height,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.width,
//         }
//     }

//     pub(crate) fn cross_margin_start(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.top,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.start,
//         }
//     }

//     pub(crate) fn cross_margin_end(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.bottom,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.end,
//         }
//     }

//     pub(crate) fn align_self(&self, parent: &Style) -> AlignSelf {
//         if self.align_self == AlignSelf::Auto {
//             match parent.align_items {
//                 AlignItems::FlexStart => AlignSelf::FlexStart,
//                 AlignItems::FlexEnd => AlignSelf::FlexEnd,
//                 AlignItems::Center => AlignSelf::Center,
//                 AlignItems::Baseline => AlignSelf::Baseline,
//                 AlignItems::Stretch => AlignSelf::Stretch,
//             }
//         } else {
//             self.align_self
//         }
//     }
// }

// #[test]
// fn test(){
// 	use map;
// 	use slab;
// 	let mut vec = Vec::new();

// 	let time = std::time::Instant::now();
// 	for i in 0..1000000 {
// 		vec.push(Some(Style::default()));
// 	}
// 	let r = None;
// 	out_any!(log::trace, "size:{:?}", std::mem::size_of_val(&r));
// 	out_any!(log::trace, "size:{:?}", std::mem::size_of::<Option<usize>>());
// 	vec.push(r);
// 	out_any!(log::trace, "{:?}", std::time::Instant::now() - time);

// 	let mut vec = map::vecmap::VecMap::new();
// 	let time = std::time::Instant::now();
// 	for i in 1..1000001 {
// 		vec.insert(i, Style::default());
// 	}
// 	out_any!(log::trace, "vecmap1:{:?}", std::time::Instant::now() - time);

// 	let mut vec = map::vecmap::VecMap::new();
// 	let time = std::time::Instant::now();
// 	vec.insert(1000000, Style::default());
// 	out_any!(log::trace, "vecmap2: {:?}", std::time::Instant::now() - time);

// 	let mut vec = slab::Slab::new();
// 	let time = std::time::Instant::now();
// 	for i in 1..1000001 {
// 		vec.insert(Style::default());
// 	}
// 	out_any!(log::trace, "slab1:{:?}", std::time::Instant::now() - time);
// }
