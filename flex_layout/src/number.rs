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

impl Number {
    /// 判断是否定义
    pub fn is_defined(self) -> bool {
        self != Number::Undefined
    }

}

// pub trait MinMax<In, Out> {
//     fn maybe_min(self, rhs: In) -> Out;
//     fn maybe_max(self, rhs: In) -> Out;
// }

// impl MinMax<Number, Number> for Number {
//     fn maybe_min(self, rhs: Number) -> Number {
//         match self {
//             Number::Defined(val) => match rhs {
//                 Number::Defined(other) => Number::Defined(val.min(other)),
//                 Number::Undefined => self,
//             },
//             Number::Undefined => Number::Undefined,
//         }
//     }

//     fn maybe_max(self, rhs: Number) -> Number {
//         match self {
//             Number::Defined(val) => match rhs {
//                 Number::Defined(other) => Number::Defined(val.max(other)),
//                 Number::Undefined => self,
//             },
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl MinMax<f32, Number> for Number {
//     fn maybe_min(self, rhs: f32) -> Number {
//         match self {
//             Number::Defined(val) => Number::Defined(val.min(rhs)),
//             Number::Undefined => Number::Undefined,
//         }
//     }

//     fn maybe_max(self, rhs: f32) -> Number {
//         match self {
//             Number::Defined(val) => Number::Defined(val.max(rhs)),
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl MinMax<Number, f32> for f32 {
//     fn maybe_min(self, rhs: Number) -> f32 {
//         match rhs {
//             Number::Defined(val) => self.min(val),
//             Number::Undefined => self,
//         }
//     }

//     fn maybe_max(self, rhs: Number) -> f32 {
//         match rhs {
//             Number::Defined(val) => self.max(val),
//             Number::Undefined => self,
//         }
//     }
// }

// impl ToNumber for f32 {
//     fn to_number(self) -> Number {
//         Number::Defined(self)
//     }
// }

// impl ops::Add<f32> for Number {
//     type Output = Number;

//     fn add(self, rhs: f32) -> Number {
//         match self {
//             Number::Defined(val) => Number::Defined(val + rhs),
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Add<Number> for Number {
//     type Output = Number;

//     fn add(self, rhs: Number) -> Number {
//         match self {
//             Number::Defined(val) => match rhs {
//                 Number::Defined(other) => Number::Defined(val + other),
//                 Number::Undefined => self,
//             },
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Sub<f32> for Number {
//     type Output = Number;

//     fn sub(self, rhs: f32) -> Number {
//         match self {
//             Number::Defined(val) => Number::Defined(val - rhs),
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Sub<Number> for Number {
//     type Output = Number;

//     fn sub(self, rhs: Number) -> Number {
//         match self {
//             Number::Defined(val) => match rhs {
//                 Number::Defined(other) => Number::Defined(val - other),
//                 Number::Undefined => self,
//             },
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Mul<f32> for Number {
//     type Output = Number;

//     fn mul(self, rhs: f32) -> Number {
//         match self {
//             Number::Defined(val) => Number::Defined(val * rhs),
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Mul<Number> for Number {
//     type Output = Number;

//     fn mul(self, rhs: Number) -> Number {
//         match self {
//             Number::Defined(val) => match rhs {
//                 Number::Defined(other) => Number::Defined(val * other),
//                 Number::Undefined => self,
//             },
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Div<f32> for Number {
//     type Output = Number;

//     fn div(self, rhs: f32) -> Number {
//         match self {
//             Number::Defined(val) => Number::Defined(val / rhs),
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }

// impl ops::Div<Number> for Number {
//     type Output = Number;

//     fn div(self, rhs: Number) -> Number {
//         match self {
//             Number::Defined(val) => match rhs {
//                 Number::Defined(other) => Number::Defined(val / other),
//                 Number::Undefined => self,
//             },
//             Number::Undefined => Number::Undefined,
//         }
//     }
// }
