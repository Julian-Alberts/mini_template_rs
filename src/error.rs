use std::fmt::Display;

pub type Result<'t, T> = std::result::Result<T, Error<'t>>;

#[derive(Debug, PartialEq)]
pub enum Error<'t> {
    Modifier(super::modifier::error::Error),
    UnknownVariable(&'t str),
    UnknownModifier(&'t str),
    UnknownTemplate,
}

impl<'t> std::error::Error for Error<'t> {}

impl<'t> Display for Error<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modifier(e) => e.fmt(f),
            Self::UnknownVariable(var_name) => write!(f, "unknown variable {}", var_name),
            Self::UnknownModifier(modifier_name) => write!(f, "unknown modifier {}", modifier_name),
            Self::UnknownTemplate => write!(f, "unknown template"),
        }
    }
}
