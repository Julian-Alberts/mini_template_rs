use nom::{IResult, Parser};
use nom_span::Spanned;

pub fn kw<'a>(input: nom_span::Spanned<&'a str>) -> IResult<Spanned<&'a str>, Kw> {
    nom::branch::alt((
        r#if::enum_wrapped,
        r#else::enum_wrapped,
        r#while::enum_wrapped,
        r#true::enum_wrapped,
        r#end::enum_wrapped,
        r#false::enum_wrapped,
        r#include::enum_wrapped,
        r#with::enum_wrapped,
        r#null::enum_wrapped,
    ))
    .parse(input)
}

macro_rules! new_kw {
    (mod $mod_name: ident { $fn_name:ident($kw:literal) => $enum_variant:ident($struct_name:ident) }) => {
        pub use $mod_name::$fn_name;
        mod $mod_name {
            use super::{$struct_name, Kw};
            use nom::IResult;

            pub fn $fn_name<'a>(
                input: nom_span::Spanned<&'a str>,
            ) -> IResult<nom_span::Spanned<&'a str>, $struct_name> {
                nom::bytes::complete::tag($kw)(input).map(|(next_input, _)| {
                    (
                        next_input,
                        $struct_name {
                            span: crate::template::Span::default(),
                        },
                    )
                })
            }

            pub fn enum_wrapped<'a>(
                input: nom_span::Spanned<&'a str>,
            ) -> IResult<nom_span::Spanned<&'a str>, Kw> {
                $fn_name(input).map(|(next_input, kw)| (next_input, Kw::$enum_variant(kw)))
            }
        }

        pub struct $struct_name {
            span: crate::template::Span,
        }

        impl From<$struct_name> for Kw {
            fn from(value: $struct_name) -> Self {
                Kw::$enum_variant(value)
            }
        }
    };
}

new_kw!(mod r#if { kw_if("if") => If(KwIf) });
new_kw!(mod r#else { kw_else("else") => Else(KwElse) });
new_kw!(mod r#while { kw_while("while") => While(KwWhile) });
new_kw!(mod r#true { kw_true("true") => True(KwTrue) });
new_kw!(mod r#end { kw_end("end") => End(KwEnd) });
new_kw!(mod r#false { kw_false("false") => False(KwFalse) });
new_kw!(mod r#include { kw_include("include") => Include(KwInclude) });
new_kw!(mod r#with { kw_with("with") => With(KwWith) });
new_kw!(mod r#null { kw_null("null") => Null(KwNull) });

pub enum Kw {
    If(KwIf),
    Else(KwElse),
    While(KwWhile),
    True(KwTrue),
    End(KwEnd),
    False(KwFalse),
    Include(KwInclude),
    With(KwWith),
    Null(KwNull),
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom_span::Spanned;

    #[test]
    fn test_kw_if() {
        let input: Spanned<&str> = nom_span::Spanned::new("if condition", true);
        let result = kw(input);
        assert!(result.is_ok());
        let (_, parsed_kw) = result.unwrap();
        match parsed_kw {
            Kw::If(_) => (),
            _ => panic!("Expected Kw::If"),
        }
    }

    #[test]
    fn test_kw_else() {
        let input: Spanned<&str> = nom_span::Spanned::new("else more content", true);
        let result = kw(input);
        assert!(result.is_ok());
        let (_, parsed_kw) = result.unwrap();
        match parsed_kw {
            Kw::Else(_) => (),
            _ => panic!("Expected Kw::Else"),
        }
    }

    #[test]
    fn test_kw_true() {
        let input: Spanned<&str> = nom_span::Spanned::new("true some text", true);
        let result = kw(input);
        assert!(result.is_ok());
        let (_, parsed_kw) = result.unwrap();
        match parsed_kw {
            Kw::True(_) => (),
            _ => panic!("Expected Kw::True"),
        }
    }

    #[test]
    fn test_kw_null() {
        let input: Spanned<&str> = nom_span::Spanned::new("null value here", true);
        let result = kw(input);
        assert!(result.is_ok());
        let (_, parsed_kw) = result.unwrap();
        match parsed_kw {
            Kw::Null(_) => (),
            _ => panic!("Expected Kw::Null"),
        }
    }

    #[test]
    fn test_kw_not_found() {
        let input: Spanned<&str> = nom_span::Spanned::new("unknown_keyword", true);
        let result = kw(input);
        assert!(result.is_err());
    }
}
