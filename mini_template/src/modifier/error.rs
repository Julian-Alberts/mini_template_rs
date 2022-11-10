use std::fmt::Display;

use crate::value::{TypeError, Value};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, PartialEq)]
pub enum Error {
    MissingArgument {
        argument_name: &'static str,
    },
    Type {
        value: String,
        type_error: TypeError,
    },
    Modifier(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingArgument { argument_name } => {
                write!(f, "Missing argument \"{}\"", argument_name)
            }
            Self::Type { value, type_error } => write!(
                f,
                "Can not convert {} to type {} value of type {} found",
                value, type_error.expected_type, type_error.storage_type
            ),
            Self::Modifier(e) => write!(f, "{e}"),
        }
    }
}

pub trait IntoModifierResult<T> {
    fn into_modifier_result(self) -> Result<T>;
}

impl<T> IntoModifierResult<T> for std::result::Result<T, String>
where
    T: Into<Value>,
{
    fn into_modifier_result(self) -> Result<T> {
        self.or_else(|e| Err(Error::Modifier(e)))
    }
}

impl<T> IntoModifierResult<T> for T
where
    T: Into<Value>,
{
    fn into_modifier_result(self) -> Result<T> {
        Ok(self)
    }
}
