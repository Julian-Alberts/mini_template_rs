pub mod ident;
mod storage_method;
mod variable_manager;

pub use storage_method::StorageMethod;
pub use variable_manager::VariableManager;

use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

/// Values are used as variables inside a template.
#[derive(Debug, Clone)]
pub enum Value {
    /// Stores a string
    String(String),
    /// Stores a number as f64
    Number(f64),
    /// Stores a boolean
    Bool(bool),
    /// Null value
    Null,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::String(s) => s.hash(state),
            Value::Number(n) => n.to_bits().hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Null => {}
        }
    }
}

impl Value {
    /// Convert any given value into a boolean
    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Number(n) => *n != 0.,
            Self::String(s) => !s.is_empty(),
            Self::Null => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s == o,
            (Self::Number(s), Self::Number(o)) => s == o,
            (Self::Bool(s), Self::Bool(o)) => s == o,
            (Self::Bool(s), o) => *s == o.as_bool(),
            (s, Self::Bool(o)) => s.as_bool() == *o,
            (Self::String(s), Self::Number(o)) => s == &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() == o,
            (Self::Null, Self::Null) => true,
            (Self::Null, _) | (_, Self::Null) => false,
        }
    }
}

#[allow(clippy::bool_comparison)]
impl PartialOrd for Value {
    fn lt(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s < o,
            (Self::Number(s), Self::Number(o)) => s < o,
            (Self::Bool(s), Self::Bool(o)) => s < o,
            (Self::Bool(s), o) => *s < o.as_bool(),
            (s, Self::Bool(o)) => s.as_bool() < *o,
            (Self::String(s), Self::Number(o)) => s < &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() < o,

            (Self::Null, _) => true,
            (_, Self::Null) => false,
        }
    }

    fn ge(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s >= o,
            (Self::Number(s), Self::Number(o)) => s >= o,
            (Self::Bool(s), Self::Bool(o)) => s >= o,
            (Self::Bool(s), o) => *s == o.as_bool(),
            (s, Self::Bool(o)) => s.as_bool() >= *o,
            (Self::String(s), Self::Number(o)) => s >= &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() >= o,

            (Self::Null, _) => false,
            (_, Self::Null) => true,
        }
    }

    fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s > o,
            (Self::Number(s), Self::Number(o)) => s > o,
            (Self::Bool(s), Self::Bool(o)) => s > o,
            (Self::Bool(s), o) => *s > o.as_bool(),
            (s, Self::Bool(o)) => s.as_bool() > *o,
            (Self::String(s), Self::Number(o)) => s > &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() > o,

            (Self::Null, _) => false,
            (_, Self::Null) => true,
        }
    }

    fn le(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s <= o,
            (Self::Number(s), Self::Number(o)) => s <= o,
            (Self::Bool(s), Self::Bool(o)) => s <= o,
            (Self::Bool(s), o) => *s <= o.as_bool(),
            (s, Self::Bool(o)) => s.as_bool() <= *o,
            (Self::String(s), Self::Number(o)) => s <= &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() <= o,

            (Self::Null, _) => true,
            (_, Self::Null) => false,
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(std::cmp::Ordering::Equal)
        } else if self < other {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

/// Error type for mismatched types.
///
/// This error type is used if the expected value type and the given value type do not match.
#[derive(Debug, PartialEq)]
pub struct TypeError {
    pub storage_type: &'static str,
    pub expected_type: &'static str,
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Self::String(s) => s.to_owned(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => {
                if *b {
                    String::from("true")
                } else {
                    String::from("false")
                }
            }
            Self::Null => String::from("null"),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
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

macro_rules! value_impl {
    ($name: ident => $main_type: ty as [$($type: ty),+]) => {
        value_impl!($name => $main_type);
        $(
            impl TryFrom<&Value> for $type {
                type Error = TypeError;
                fn try_from(value: &Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::$name(s) => Ok(*s as $type),
                        _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
                    }
                }
            }

            impl From<$type> for Value {
                fn from(s: $type) -> Self {
                    Self::$name(s as $main_type)
                }
            }

        )+
    };
    ($name: ident => $type: ty) => {
        value_impl!(try_from_type $name => $type);
        value_impl!(from_value $name => $type);
    };
    (try_from_type $name: ident => $type: ty) => {
        impl TryFrom<&Value> for $type {
            type Error = TypeError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok(s.clone()),
                    _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
                }
            }
        }
    };
    (from_value $name: ident => $type: ty) => {
        impl From<$type> for Value {
            fn from(s: $type) -> Self {
                Self::$name(s)
            }
        }
    }

}

value_impl!(String => String);
value_impl!(Bool => bool);
value_impl!(Number => f64 as [isize, i32, usize, u32]);
