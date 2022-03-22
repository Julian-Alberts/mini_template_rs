use crate::value::ident::ResolvedIdent;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Modifier(super::modifier::error::Error),
    UnknownVariable(ResolvedIdent),
    UnknownModifier(String),
    UnknownTemplate,
    UnsupportedIdentifier,
}

impl std::error::Error for Error {}

impl<'t> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modifier(e) => e.fmt(f),
            Self::UnknownVariable(var_name) => write!(f, "unknown variable {}", var_name),
            Self::UnknownModifier(modifier_name) => write!(f, "unknown modifier {}", modifier_name),
            Self::UnknownTemplate => write!(f, "unknown template"),
            Self::UnsupportedIdentifier => f.write_str("Tried to access unsupported Identifier"),
        }
    }
}
