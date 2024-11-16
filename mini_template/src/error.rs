use std::fmt::{Debug, Display};

pub type Result<'t, T> = std::result::Result<T, Error<'t>>;

#[derive(Debug)]
pub enum Error<'t> {
    Modifier(super::modifier::error::Error),
    UnknownVariable(&'t str),
    UnknownModifier(&'t str),
    UnknownTemplate,
    IoError(std::io::Error),
}

impl<'t> std::error::Error for Error<'t> {}

impl<'t> Display for Error<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modifier(e) => Display::fmt(e, f),
            Self::UnknownVariable(var_name) => write!(f, "unknown variable {}", var_name),
            Self::UnknownModifier(modifier_name) => write!(f, "unknown modifier {}", modifier_name),
            Self::UnknownTemplate => write!(f, "unknown template"),
            Self::IoError(e) => Display::fmt(e, f),
        }
    }
}

impl<'t> PartialEq for Error<'t> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UnknownTemplate, Self::UnknownTemplate) => true,
            (Self::UnknownModifier(m1), Self::UnknownModifier(m2)) => m1 == m2,
            (Self::UnknownVariable(v1), Self::UnknownVariable(v2)) => v1 == v2,
            (Self::Modifier(e1), Self::Modifier(e2)) => e1 == e2,
            (Self::IoError(e1), Self::IoError(e2)) => e1.kind() == e2.kind(),
            _ => false,
        }
    }
}

impl<'t> From<std::io::Error> for Error<'t> {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
