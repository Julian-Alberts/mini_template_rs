pub mod ident;
mod storage_method;
mod value_manager;

pub use storage_method::StorageMethod;
pub use value_manager::ValueManager;

use crate::{
    error::Error,
    prelude::{IdentifiableValue, TplPartialEq, TplPartialOrd, ValueAs},
    value::ident::{ResolvedIdent, ResolvedIdentPart},
};
use serde_json::{Map, Value};

impl TplPartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s == o,
            (Self::Number(s), Self::Number(o)) => s == o,
            (Self::Bool(s), Self::Bool(o)) => s == o,
            (Self::Bool(s), o) => *s == ValueAs::as_bool(o),
            (s, Self::Bool(o)) => ValueAs::as_bool(s) == *o,
            (Self::String(s), Self::Number(o)) => s == &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() == o,
            (Self::Null, Self::Null) => true,
            (Self::Null, _) | (_, Self::Null) => false,
            (Self::Object(s), Self::Object(o)) => s == o,
            _ => false,
        }
    }
}

#[allow(clippy::bool_comparison)]
impl TplPartialOrd for Value {
    fn lt(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s < o,
            (Self::Number(s), Self::Number(o)) => s.as_f64().unwrap() < o.as_f64().unwrap(),
            (Self::Bool(s), Self::Bool(o)) => s < o,
            (Self::Bool(s), o) => *s < o.as_bool().unwrap(),
            (s, Self::Bool(o)) => s.as_bool().unwrap() < *o,
            (Self::String(s), Self::Number(o)) => s < &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() < o,
            (Self::Object(_), _) | (_, Self::Object(_)) => false,
            (Self::Null, _) => true,
            (_, Self::Null) => false,
            (Self::Array(_), _) => false,
            (_, Self::Array(_)) => false,
        }
    }

    fn ge(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s >= o,
            (Self::Number(s), Self::Number(o)) => s.as_f64().unwrap() >= o.as_f64().unwrap(),
            (Self::Bool(s), Self::Bool(o)) => s >= o,
            (Self::Bool(s), o) => *s == o.as_bool().unwrap(),
            (s, Self::Bool(o)) => s.as_bool().unwrap() >= *o,
            (Self::String(s), Self::Number(o)) => s >= &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() >= o,
            (Self::Object(_), _) | (_, Self::Object(_)) => false,

            (Self::Null, _) => false,
            (_, Self::Null) => true,
            (Self::Array(_), _) => false,
            (_, Self::Array(_)) => false,
        }
    }

    fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s > o,
            (Self::Number(s), Self::Number(o)) => s.as_f64().unwrap() > o.as_f64().unwrap(),
            (Self::Bool(s), Self::Bool(o)) => s > o,
            (Self::Bool(s), o) => *s > o.as_bool().unwrap(),
            (s, Self::Bool(o)) => s.as_bool().unwrap() > *o,
            (Self::String(s), Self::Number(o)) => s > &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() > o,
            (Self::Object(_), _) | (_, Self::Object(_)) => false,

            (Self::Null, _) => false,
            (_, Self::Null) => true,
            (Self::Array(_), _) => false,
            (_, Self::Array(_)) => false,
        }
    }

    fn le(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(o)) => s <= o,
            (Self::Number(s), Self::Number(o)) => s.as_f64().unwrap() <= o.as_f64().unwrap(),
            (Self::Bool(s), Self::Bool(o)) => s <= o,
            (Self::Bool(s), o) => *s <= ValueAs::as_bool(o),
            (s, Self::Bool(o)) => s.as_bool().unwrap() <= *o,
            (Self::String(s), Self::Number(o)) => s <= &o.to_string(),
            (Self::Number(s), Self::String(o)) => &s.to_string() <= o,
            (Self::Object(_), _) | (_, Self::Object(_)) => false,

            (Self::Null, _) => true,
            (_, Self::Null) => false,
            (Self::Array(_), _) => false,
            (_, Self::Array(_)) => false,
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if TplPartialEq::eq(self, other) {
            Some(std::cmp::Ordering::Equal)
        } else if self.le(other) {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

impl ValueAs for Value {
    fn as_bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Number(n) => n.as_f64() != Some(0.),
            Self::String(s) => !s.is_empty(),
            Self::Null => false,
            Self::Object(_) => true,
            Self::Array(a) => a.len() > 0,
        }
    }

    fn as_string(&self) -> String {
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
            Self::Object(o) => format!("{:#?}", o),
            Self::Array(a) => format!("{:#?}", a),
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

impl IdentifiableValue for Map<String, Value> {
    fn get(&self, ident: &ResolvedIdent) -> crate::error::Result<&Value> {
        let key = get_ident_key(ident)?;
        let value = self.get(&key);
        if let Some(next) = &ident.next {
            if let Some(Value::Object(obj)) = value {
                IdentifiableValue::get(obj, next)
            } else {
                Err(Error::CanNotUseAsObject(ident.clone()))
            }
        } else if let Some(value) = value {
            Ok(value)
        } else {
            return Err(Error::UnknownVariable(ident.clone()));
        }
    }

    fn get_mut(&mut self, ident: &ResolvedIdent) -> crate::error::Result<&mut Value> {
        let key = get_ident_key(ident)?;
        let value = self.get_mut(&key);
        if let Some(next) = &ident.next {
            if let Some(Value::Object(obj)) = value {
                IdentifiableValue::get_mut(obj, next)
            } else {
                Err(Error::CanNotUseAsObject(ident.clone()))
            }
        } else if let Some(value) = value {
            Ok(value)
        } else {
            return Err(Error::UnknownVariable(ident.clone()));
        }
    }

    fn set(&mut self, ident: &ResolvedIdent, value: Value) -> crate::error::Result<()> {
        let key = get_ident_key(ident)?;
        if let Some(next) = &ident.next {
            if let Some(Value::Object(obj)) = self.get_mut(&key) {
                IdentifiableValue::set(obj, next, value)
            } else {
                Err(Error::CanNotUseAsObject(ident.clone()))
            }
        } else {
            self.insert(key, value);
            Ok(())
        }
    }
}

fn get_ident_key(ident: &ResolvedIdent) -> crate::error::Result<String> {
    use crate::prelude::*;
    match &*ident.part {
        ResolvedIdentPart::Static(s) => Ok(s.get_string().to_owned()),
        ResolvedIdentPart::Dynamic(d) => match TplTryInto::try_into(d) {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::UnsupportedIdentifier),
        },
    }
}
