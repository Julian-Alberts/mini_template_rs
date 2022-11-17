use crate::template::UnknownModifierError;
use crate::value::ident::ResolvedIdent;
#[cfg(feature = "include")]
use crate::value::TypeError;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Modifier(super::modifier::error::Error),
    UnknownVariable(ResolvedIdent),
    UnknownModifier(UnknownModifierError),
    UnknownTemplate,
    UnsupportedIdentifier,
    UnknownProperty(ResolvedIdent),
    #[cfg(feature = "include")]
    Include(TypeError),
}

impl std::error::Error for Error {}

impl Display for Error {
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
            #[cfg(feature = "include")]
            Self::Include(e) => write!(
                f,
                "Can not use value of type \"{}\" as template name",
                e.storage_type
            ),
            Self::UnknownProperty(ident) => crate::util::mark_area_in_string(
                unsafe { ident.span.input.as_ref().unwrap() },
                ident.span.start,
                ident.span.end,
                f,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::template::Span;
    use crate::util::TemplateString;
    use crate::value::ident::{ResolvedIdent, ResolvedIdentPart};

    #[test]
    fn format_string() {
        let error = super::Error::UnknownVariable(ResolvedIdent {
            span: Span {
                end: 7,
                start: 5,
                input: "0123456789",
            },
            part: Box::new(ResolvedIdentPart::Static(TemplateString::Ptr("wasd"))),
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
            part: Box::new(ResolvedIdentPart::Static(TemplateString::Ptr("wasd"))),
            next: None,
        });
        assert_eq!(&format!("{}", error), "2> ABCDEFGHIJ\n      ^^\n")
    }

    #[test]
    fn print_modifier_error() {
        let e = super::Error::Modifier(crate::modifier::Error::Modifier("Test".to_owned()));
        assert_eq!(format!("{e}").as_str(), "Test")
    }

    #[test]
    fn print_unknown_template() {
        let e = super::Error::UnknownTemplate;
        assert_eq!(format!("{e}").as_str(), "unknown template")
    }
    
    #[test]
    fn print_unsupported_ident() {
        let e = super::Error::UnsupportedIdentifier;
        assert_eq!(format!("{e}").as_str(), "Test")
    }
}
