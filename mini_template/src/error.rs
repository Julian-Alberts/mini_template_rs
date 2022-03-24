use crate::value::ident::ResolvedIdent;
use std::fmt::Display;
use crate::template::UnknownModifierError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Modifier(super::modifier::error::Error),
    UnknownVariable(ResolvedIdent),
    UnknownModifier(UnknownModifierError),
    UnknownTemplate,
    UnsupportedIdentifier,
}

impl std::error::Error for Error {}

impl<'t> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modifier(e) => e.fmt(f),
            Self::UnknownVariable(ident) => crate::util::mark_area_in_string(
                unsafe { ident.span.input.as_ref().unwrap() },
                ident.span.start,
                ident.span.end,
                f,
            ),
            Self::UnknownModifier(modifier) => crate::util::mark_area_in_string(
                unsafe { modifier.span.input.as_ref().unwrap() },
                modifier.span.start,
                modifier.span.end,
                f,
            ),
            Self::UnknownTemplate => write!(f, "unknown template"),
            Self::UnsupportedIdentifier => f.write_str("Tried to access unsupported Identifier"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::template::Span;
    use crate::value::ident::{ResolvedIdent, ResolvedIdentPart};

    #[test]
    fn format_string() {
        let error = super::Error::UnknownVariable(ResolvedIdent {
            span: Span {
                end: 7,
                start: 5,
                input: "0123456789",
            },
            part: Box::new(ResolvedIdentPart::Static("wasd")),
            next: None,
        });
        assert_eq!(&format!("{}", error), "1> 0123456789\n        ^^^\n")
    }

    #[test]
    fn format_string_multiple_lines() {
        let error = super::Error::UnknownVariable(ResolvedIdent {
            span: Span {
                end: 19,
                start: 14,
                input: "0123456789\nABCDEFGHIJ\nKLMNOPQRST",
            },
            part: Box::new(ResolvedIdentPart::Static("wasd")),
            next: None,
        });
        assert_eq!(&format!("{}", error), "2> ABCDEFGHIJ\n      ^^\n")
    }
}
