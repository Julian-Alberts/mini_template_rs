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
            Self::UnknownVariable(ident) => mark_area_in_string(unsafe{ident.span.input.as_ref().unwrap()}, ident.span.start, ident.span.end, f),
            Self::UnknownModifier(modifier_name) => write!(f, "unknown modifier {}", modifier_name),
            Self::UnknownTemplate => write!(f, "unknown template"),
            Self::UnsupportedIdentifier => f.write_str("Tried to access unsupported Identifier"),
        }
    }
}

fn mark_area_in_string(input: &str, start: usize, end: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    debug_assert!(start < end);
    let start_char_pos = find_char_pos(input, start);
    let end_char_pos = {
        let (line, mut col) = find_char_pos(&input[start..], end - start);
        if line == 0 {
            col += start_char_pos.1
        }
        (line + start_char_pos.0, col)
    };

    input
        .lines()
        .enumerate()
        .filter(|(line, _)| *line >= start_char_pos.0 && *line <= end_char_pos.0)
        .try_for_each(|(num, line)| {
            let num_str = (num + 1).to_string();
            let num_len = num_str.len();
            writeln!(f, "{}> {}", num_str, line)?;

            if num < start_char_pos.0 && num > end_char_pos.0 {
                return Ok(())
            }

            write!(f, "{}  ", " ".repeat(num_len))?;

            match (num == start_char_pos.0, num == end_char_pos.0) {
                (false, false) => writeln!(f, "{}", "^".repeat(line.len())),
                (false, true) => writeln!(f, "{}", "^".repeat(end_char_pos.1)),
                (true, false) => writeln!(f, "{}{}", " ".repeat(start_char_pos.1), "^".repeat(line.len() - start_char_pos.1)),
                (true, true) => writeln!(f, "{}{}", " ".repeat(start_char_pos.1), "^".repeat(line.len() - start_char_pos.1 - (end_char_pos.1 - start_char_pos.1))),
            }
        })
}

fn find_char_pos(input: &str, index: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in input.bytes().enumerate() {
        if i == index {
            break;
        }
        col += 1;
        if c == b'\n' {
            line += 1;
            col = 0;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use crate::template::Span;
    use crate::value::ident::{ResolvedIdent, ResolvedIdentPart};

    #[test]
    fn at_begin_of_string() {
        assert_eq!(super::find_char_pos("line1\nline2", 0), (0, 0))
    }

    #[test]
    fn in_first_line() {
        assert_eq!(super::find_char_pos("line1\nline2", 3), (0, 3))
    }

    #[test]
    fn in_second_line() {
        assert_eq!(super::find_char_pos("line1\nline2", 7), (1, 1))
    }

    #[test]
    fn with_empty_line() {
        assert_eq!(super::find_char_pos("line1\n\nline2", 9), (2, 2))
    }

    #[test]
    fn format_string() {
        let error = super::Error::UnknownVariable(ResolvedIdent{
            span: Span {
                end: 7,
                start: 5,
                input: "0123456789"
            },
            part: Box::new(ResolvedIdentPart::Static("wasd")),
            next: None
        });
        assert_eq!(&format!("{}", error), "1> 0123456789\n        ^^^\n")
    }

    #[test]
    fn format_string_multiple_lines() {
        let error = super::Error::UnknownVariable(ResolvedIdent{
            span: Span {
                end: 19,
                start: 14,
                input: "0123456789\nABCDEFGHIJ\nKLMNOPQRST"
            },
            part: Box::new(ResolvedIdentPart::Static("wasd")),
            next: None
        });
        assert_eq!(&format!("{}", error), "2> ABCDEFGHIJ\n      ^^\n")
    }
}
