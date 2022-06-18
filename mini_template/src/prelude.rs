use std::cmp::Ordering;

use serde_json::Number;

use crate::{value::ident::ResolvedIdent, TypeError, Value};

pub trait TplInto<T>: Sized {
    fn into(self) -> T;
}
pub trait TplFrom<T>: Sized {
    fn from(_: T) -> Self;
}

pub trait TplTryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

pub trait TplTryInto<T>: Sized {
    type Error;
    fn try_into(self) -> Result<T, Self::Error>;
}

impl<T, U> TplInto<U> for T
where
    U: TplFrom<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}

impl<T, U> TplTryInto<U> for T
where
    U: TplTryFrom<T>,
{
    type Error = U::Error;
    fn try_into(self) -> Result<U, U::Error> {
        U::try_from(self)
    }
}

pub trait ValueAs {
    fn as_bool(&self) -> bool;
    fn as_string(&self) -> String;
}

pub trait TplPartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

pub trait TplPartialOrd<Rhs: ?Sized = Self>: PartialEq<Rhs> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;
    fn lt(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }
    fn le(&self, other: &Rhs) -> bool {
        !matches!(self.partial_cmp(other), None | Some(Ordering::Greater))
    }
    fn gt(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }
    fn ge(&self, other: &Rhs) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
}

impl<'a> TplTryFrom<&'a Value> for &'a str {
    type Error = TypeError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => {
                let s = &s[..];
                Ok(s)
            }
            _ => Err(TypeError {
                expected_type: stringify!(&str),
                storage_type: stringify!(String),
            }),
        }
    }
}

pub trait IdentifiableValue {
    fn get(&self, ident: &ResolvedIdent) -> crate::error::Result<&Value>;
    fn get_mut(&mut self, ident: &ResolvedIdent) -> crate::error::Result<&mut Value>;
    fn set(&mut self, ident: &ResolvedIdent, value: Value) -> crate::error::Result<()>;
}

macro_rules! value_impl {
    ($name: ident => $type: ty: $fn: ident) => {
        impl TplTryFrom<&Value> for $type {
            type Error = TypeError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok($fn(s) as $type),
                    _ => Err(TypeError {
                        expected_type: stringify!($type),
                        storage_type: stringify!($name),
                    }),
                }
            }
        }

        impl TplTryFrom<Value> for $type {
            type Error = TypeError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok($fn(&s) as $type),
                    _ => Err(TypeError {
                        expected_type: stringify!($type),
                        storage_type: stringify!($name),
                    }),
                }
            }
        }

        impl TplFrom<$type> for Value {
            fn from(s: $type) -> Self {
                serde_json::json!(s)
            }
        }
    };
    ($name: ident => $type: ty) => {
        impl TplTryFrom<&Value> for $type {
            type Error = TypeError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok(s.clone()),
                    _ => Err(TypeError {
                        expected_type: stringify!($type),
                        storage_type: stringify!($name),
                    }),
                }
            }
        }

        impl TplTryFrom<Value> for $type {
            type Error = TypeError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok(s),
                    _ => Err(TypeError {
                        expected_type: stringify!($type),
                        storage_type: stringify!($name),
                    }),
                }
            }
        }

        impl TplFrom<$type> for Value {
            fn from(s: $type) -> Self {
                Self::$name(s)
            }
        }
    };
}

#[inline]
fn number_to_f64(num: &Number) -> f64 {
    if num.is_f64() {
        num.as_f64().unwrap()
    } else if num.is_i64() {
        num.as_i64().unwrap() as f64
    } else {
        num.as_u64().unwrap() as f64
    }
}

#[inline]
fn number_to_u64(num: &Number) -> u64 {
    if num.is_f64() {
        num.as_f64().unwrap() as u64
    } else if num.is_i64() {
        num.as_i64().unwrap() as u64
    } else {
        num.as_u64().unwrap()
    }
}

#[inline]
fn number_to_i64(num: &Number) -> i64 {
    if num.is_f64() {
        num.as_f64().unwrap() as i64
    } else if num.is_i64() {
        num.as_i64().unwrap()
    } else {
        num.as_u64().unwrap() as i64
    }
}

value_impl!(String => String);
value_impl!(Bool => bool);
value_impl!(Number => f64: number_to_f64);
value_impl!(Number => f32: number_to_f64);
value_impl!(Number => isize: number_to_i64);
value_impl!(Number => i8: number_to_i64);
value_impl!(Number => i16: number_to_i64);
value_impl!(Number => i32: number_to_i64);
value_impl!(Number => i64: number_to_i64);
value_impl!(Number => usize: number_to_u64);
value_impl!(Number => u8: number_to_u64);
value_impl!(Number => u16: number_to_u64);
value_impl!(Number => u32: number_to_u64);
value_impl!(Number => u64: number_to_u64);
