

pub trait OrElse<T> {
    /// 如果为Undefined，则返回other
    fn or_else(self, other: T) -> T;
}

impl OrElse<f32> for Number {
    fn or_else(self, other: f32) -> f32 {
        match self {
            Number::Defined(val) => val,
            Number::Undefined => other,
        }
    }
}

impl OrElse<Number> for Number {
    fn or_else(self, other: Number) -> Number {
        match self {
            Number::Defined(_) => self,
            Number::Undefined => other,
        }
    }
}

pub trait DimensionValue {
	fn resolve_value(self, parent: f32) -> f32;
	fn is_defined(self) -> bool;
	fn is_undefined(self) -> bool;
	fn is_points(self) -> bool;
}

impl DimensionValue for Dimension {
    fn resolve_value(self, parent: f32) -> f32 {
        match self {
            Dimension::Points(points) => points,
            Dimension::Percent(percent) => parent * percent,
            _ => 0.0,
        }
    }

    fn is_defined(self) -> bool {
        match self {
            Dimension::Points(_) => true,
            Dimension::Percent(_) => true,
            _ => false,
        }
    }
    fn is_undefined(self) -> bool {
        match self {
            Dimension::Points(_) => false,
            Dimension::Percent(_) => false,
            _ => true,
        }
    }
    fn is_points(self) -> bool {
        match self {
            Dimension::Points(_) => true,
            _ => false,
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

    // fn overflow(&self) -> Overflow;
    fn min_width(&self) -> Dimension;
    fn min_height(&self) -> Dimension;
    fn max_width(&self) -> Dimension;
    fn max_height(&self) -> Dimension;
    fn aspect_ratio(&self) -> Number;
}