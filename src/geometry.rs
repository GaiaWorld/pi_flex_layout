use core::ops::Add;
use std::ops::Sub;
use crate::number::Number;


/// 四边的间隙， 采用left right top bottom定义四边的间隙
#[derive(Debug, Copy, Default, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub struct SideGap<T: Default> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}
impl<T: Copy + Default + Add<Output = T>> SideGap<T> {

    pub fn gap_size(&self) -> Size<T> {
        Size {
            width: self.right + self.left,
            height: self.bottom + self.top,
        }
    }
}
/// 矩形， 采用left right top bottom定义矩形
#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, Hash)]
pub struct Rect<T: Default> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}
impl<T: Copy + Default + Add<Output = T> + Sub<Output = T>> Rect<T> {
    pub fn new(left: T, top: T, width: T, height: T) -> Rect<T> {
        Rect {
            left,
            right: left + width,
            top,
            bottom: top + height,
        }
    }
    pub fn size(&self) -> Size<T> {
        Size {
            width: self.right - self.left,
            height: self.bottom - self.top,
        }
    }
    pub fn pos(&self) -> Point<T> {
        Point {
            x: self.left,
            y: self.top,
        }
    }
}

/// 尺寸
#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, Hash)]
pub struct Size<T: Default> {
    pub width: T,
    pub height: T,
}

impl Size<()> {
    pub fn undefined() -> Size<Number> {
        Size {
            width: Number::Undefined,
            height: Number::Undefined,
        }
    }
}

impl<T: Default> Size<T> {
    pub fn new(width: T, height: T) -> Size<T> {
        Size { width, height }
    }
}
impl Add for Size<f32> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}
impl Sub for Size<f32> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Size {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

/// 点
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}
impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}
