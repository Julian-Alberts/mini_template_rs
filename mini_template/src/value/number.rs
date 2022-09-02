use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone, Debug)]
pub enum Number {
    ISize(isize),
    USize(usize),
    F32(f32),
    F64(f64),
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        use self::Number::*;
        match (self, other) {
            (ISize(s), ISize(o)) => s == o,
            (ISize(s), USize(o)) => *s as usize == *o,
            (ISize(s), F32(o)) => *s as f32 == *o,
            (ISize(s), F64(o)) => *s as f64 == *o,

            (USize(s), ISize(o)) => *s as isize == *o,
            (USize(s), USize(o)) => s == o,
            (USize(s), F32(o)) => *s as f32 == *o,
            (USize(s), F64(o)) => *s as f64 == *o,

            (F32(s), ISize(o)) => *s == *o as f32,
            (F32(s), USize(o)) => *s == *o as f32,
            (F32(s), F32(o)) => s == o,
            (F32(s), F64(o)) => *s as f64 == *o,

            (F64(s), ISize(o)) => *s == *o as f64,
            (F64(s), USize(o)) => *s == *o as f64,
            (F64(s), F32(o)) => *s == *o as f64,
            (F64(s), F64(o)) => s == o,
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use self::Number::*;
        let result = match (self, other) {
            (ISize(s), ISize(o)) => s.cmp(o),
            (ISize(s), USize(o)) => (*s as usize).cmp(o),
            (ISize(s), F32(o)) => return (*s as f32).partial_cmp(o),
            (ISize(s), F64(o)) => return (*s as f64).partial_cmp(o),

            (USize(s), ISize(o)) => (*s as isize).cmp(o),
            (USize(s), USize(o)) => s.cmp(o),
            (USize(s), F32(o)) => return (*s as f32).partial_cmp(o),
            (USize(s), F64(o)) => return (*s as f64).partial_cmp(o),

            (F32(s), ISize(o)) => return (*o as f32).partial_cmp(s),
            (F32(s), USize(o)) => return (*o as f32).partial_cmp(s),
            (F32(s), F32(o)) => return s.partial_cmp(o),
            (F32(s), F64(o)) => return (*s as f64).partial_cmp(o),

            (F64(s), ISize(o)) => return (*o as f64).partial_cmp(s),
            (F64(s), USize(o)) => return (*o as f64).partial_cmp(s),
            (F64(s), F32(o)) => return (*o as f64).partial_cmp(s),
            (F64(s), F64(o)) => return (*o).partial_cmp(s),
        };
        Some(result)
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        use self::Number::*;
        match self {
            ISize(o) => o.to_string(),
            USize(o) => o.to_string(),
            F32(o) => o.to_string(),
            F64(o) => o.to_string(),
        }
    }
}

impl From<f32> for Number {
    fn from(n: f32) -> Self {
        Number::F32(n)
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Number::F64(n)
    }
}

impl From<isize> for Number {
    fn from(n: isize) -> Self {
        Number::ISize(n)
    }
}

impl From<usize> for Number {
    fn from(n: usize) -> Self {
        Number::USize(n)
    }
}

impl TryFrom<&str> for Number {
    type Error = ParseNumberError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const IS_FLOAT: bool = true;
        const IS_INTEGER: bool = false;
        const IS_NEGATIVE: bool = true;
        const IS_POSITIVE: bool = false;
        match (value.starts_with('-'), value.contains('.')) {
            (IS_POSITIVE, IS_INTEGER) => {
                let num = match value.parse() {
                    Ok(n) => n,
                    Err(e) => return Err(ParseNumberError::Integer(e)),
                };
                Ok(Number::USize(num))
            }
            (IS_NEGATIVE, IS_INTEGER) => {
                let num = match value.parse() {
                    Ok(n) => n,
                    Err(e) => return Err(ParseNumberError::Integer(e)),
                };
                Ok(Number::ISize(num))
            }
            (_, IS_FLOAT) => {
                let num = match value.parse() {
                    Ok(n) => n,
                    Err(e) => return Err(ParseNumberError::Float(e)),
                };
                Ok(Number::F64(num))
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseNumberError {
    Float(ParseFloatError),
    Integer(ParseIntError),
}
