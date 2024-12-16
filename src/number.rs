/// 数字类型
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum Number {
    /// 用f32定义
    Defined(f32),
    /// 未定义
    Undefined,
}

pub trait OrElse<T> {
    /// 如果为Undefined，则返回other
    fn or_else(self, other: T) -> T;
}

impl Default for Number {
    fn default() -> Number {
        Number::Undefined
    }
}

impl OrElse<f32> for Number {
    fn or_else(self, other: f32) -> f32 {
        if let Number::Defined(val) = self {
            val
        } else {
            other
        }
    }
}

impl OrElse<Number> for Number {
    fn or_else(self, other: Number) -> Number {
        if let Number::Defined(_) = self {
            self
        } else {
            other
        }
    }
}

impl Number {
    /// 判断是否定义
    pub fn is_defined(self) -> bool {
        self != Number::Undefined
    }
}
