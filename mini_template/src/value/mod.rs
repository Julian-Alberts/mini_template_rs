pub mod ident;
mod storage_method;
mod traits;
mod variable_manager;

pub use storage_method::StorageMethod;
pub use variable_manager::VariableManager;

use std::convert::TryFrom;

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
            //(Self::Object(s), Self::Object(o)) => s == o
            _ => false,
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
