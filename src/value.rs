use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}

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
value_impl!(Number => f64 as [isize, i32, usize]);
