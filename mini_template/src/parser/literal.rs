use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag},
    character::complete::{char, none_of},
    combinator::{map, opt},
    sequence::delimited,
    Parser as _,
};

use crate::parser::kw::{kw_false, kw_null, kw_true};

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Bool(bool),
    String(String),
    Number(NumberLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberLiteral {
    F64(f64),
}

pub fn value<'a>(
    input: nom_span::Spanned<&'a str>,
) -> nom::IResult<nom_span::Spanned<&'a str>, Literal> {
    alt((
        map(kw_null, |_| Literal::Null),
        map(kw_true, |_| Literal::Bool(true)),
        map(kw_false, |_| Literal::Bool(false)),
        string_value,
        number_value,
    ))
    .parse(input)
}

fn string_value<'a>(
    input: nom_span::Spanned<&'a str>,
) -> nom::IResult<nom_span::Spanned<&'a str>, Literal> {
    let (input, str) = delimited(
        char('"'),
        opt(escaped_transform(
            none_of("\\\""),
            '\\',
            alt((
                nom::combinator::value("\\", tag("\\")),
                nom::combinator::value("\"", tag("\"")),
                nom::combinator::value("\n", tag("n")),
            )),
        )),
        char('"'),
    )
    .parse(input)?;
    Ok((input, Literal::String(str.unwrap_or_default())))
}

fn number_value<'a>(
    input: nom_span::Spanned<&'a str>,
) -> nom::IResult<nom_span::Spanned<&'a str>, Literal> {
    let (input, number_str) = nom::combinator::recognize((
        nom::combinator::opt(nom::character::complete::char('-')),
        nom::character::complete::digit1,
        nom::combinator::opt((
            nom::character::complete::char('.'),
            nom::character::complete::digit1,
        )),
    ))
    .parse(input)?;

    let number: f64 = number_str.data().parse().unwrap();

    Ok((input, Literal::Number(NumberLiteral::F64(number))))
}

#[cfg(test)]
mod tests {
    use nom_span::Spanned;

    use super::{Literal, NumberLiteral};

    #[test]
    fn parse_as_correct_type() {
        let inputs = vec![
            ("null", Literal::Null),
            ("true", Literal::Bool(true)),
            ("false", Literal::Bool(false)),
            ("\"hello\"", Literal::String("hello".to_string())),
            ("42", Literal::Number(NumberLiteral::F64(42.0))),
            ("-3.14", Literal::Number(NumberLiteral::F64(-3.14))),
        ];

        for (input_str, expected_value) in inputs {
            let input: Spanned<&str> = nom_span::Spanned::new(input_str, true);
            let result = super::value(input);
            assert!(result.is_ok());
            let (remaining, value) = result.unwrap();
            assert_eq!(value, expected_value);
            assert_eq!(*remaining.data(), "");
        }
    }

    #[test]
    fn parse_null() {
        let input: Spanned<&str> = nom_span::Spanned::new("null remaining", true);
        let result = super::value(input);
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(*remaining.data(), " remaining");
        assert_eq!(value, Literal::Null);
    }

    #[test]
    fn parse_true() {
        let input: Spanned<&str> = nom_span::Spanned::new("true remaining", true);
        let result = super::value(input);
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(*remaining.data(), " remaining");
        assert_eq!(value, Literal::Bool(true));
    }

    #[test]
    fn parse_false() {
        let input: Spanned<&str> = nom_span::Spanned::new("false remaining", true);
        let result = super::value(input);
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(*remaining.data(), " remaining");
        assert_eq!(value, Literal::Bool(false));
    }

    #[test]
    fn parse_string() {
        let input: Spanned<&str> = nom_span::Spanned::new("\"hello\" remaining", true);
        let result = super::value(input);
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(*remaining.data(), " remaining");
        assert_eq!(value, Literal::String("hello".to_string()));
    }

    #[test]
    fn parse_empty_string() {
        let input: Spanned<&str> = nom_span::Spanned::new("\"\" remaining", true);
        let result = super::string_value(input);
        let (remaining, value) = result.unwrap();
        assert_eq!(value, Literal::String("".to_string()));
        assert_eq!(*remaining.data(), " remaining");
    }

    #[test]
    fn parse_string_with_quote() {
        let input: Spanned<&str> =
            nom_span::Spanned::new("\"he said: \\\"hello\\\"\" remaining", true);
        let result = super::value(input);
        let (remaining, value) = result.unwrap();
        assert_eq!(value, Literal::String("he said: \"hello\"".to_string()));
        assert_eq!(*remaining.data(), " remaining");
    }

    #[test]
    fn parse_string_with_backslash() {
        let input: Spanned<&str> = nom_span::Spanned::new(r#""path: C:\\folder" remaining"#, true);
        let result = super::value(input);
        let (remaining, value) = result.unwrap();
        assert_eq!(value, Literal::String(r#"path: C:\folder"#.to_string()));
        assert_eq!(*remaining.data(), " remaining");
    }

    #[test]
    fn parse_string_with_newline() {
        let input: Spanned<&str> = nom_span::Spanned::new("\"line1\\nline2\" remaining", true);
        let result = super::value(input);
        let (remaining, value) = result.unwrap();
        assert_eq!(value, Literal::String("line1\nline2".to_string()));
        assert_eq!(*remaining.data(), " remaining");
    }

    #[test]
    fn parse_number() {
        let input: Spanned<&str> = nom_span::Spanned::new("42 remaining", true);
        let result = super::value(input);
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(*remaining.data(), " remaining");
        assert_eq!(value, Literal::Number(NumberLiteral::F64(42.0)));
    }

    #[test]
    fn parse_negative_number() {
        let input: Spanned<&str> = nom_span::Spanned::new("-3.14 remaining", true);
        let result = super::value(input);
        let (remaining, value) = result.unwrap();
        assert_eq!(*remaining.data(), " remaining");
        assert_eq!(value, Literal::Number(NumberLiteral::F64(-3.14)));
    }
}
