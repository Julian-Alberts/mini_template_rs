pub fn mark_area_in_string(
    input: &str,
    start: usize,
    end: usize,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    debug_assert!(start < end);
    let start_char_pos = find_char_pos(input, start);
    let end_char_pos = {
        let (line, mut col) = find_char_pos(&input[start..], end - start);
        if line == 0 {
            col += start_char_pos.1
        }
        (line + start_char_pos.0, col)
    };
    mark_between_points(start_char_pos, end_char_pos, input, f)
}

pub fn mark_between_points(
    start_char_pos: (usize, usize),
    end_char_pos: (usize, usize),
    input: &str,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    input
        .lines()
        .enumerate()
        .filter(|(line, _)| *line >= start_char_pos.0 && *line <= end_char_pos.0)
        .try_for_each(|(num, line)| {
            let num_str = (num + 1).to_string();
            let num_len = num_str.len();
            writeln!(f, "{}> {}", num_str, line)?;

            if num < start_char_pos.0 && num > end_char_pos.0 {
                return Ok(());
            }

            write!(f, "{}  ", " ".repeat(num_len))?;

            match (num == start_char_pos.0, num == end_char_pos.0) {
                (false, false) => writeln!(f, "{}", "^".repeat(line.len())),
                (false, true) => writeln!(f, "{}", "^".repeat(end_char_pos.1)),
                (true, false) => writeln!(
                    f,
                    "{}{}",
                    " ".repeat(start_char_pos.1),
                    "^".repeat(line.len() - start_char_pos.1)
                ),
                (true, true) => writeln!(
                    f,
                    "{}{}",
                    " ".repeat(start_char_pos.1),
                    "^".repeat(line.len() - start_char_pos.1 - (end_char_pos.1 - start_char_pos.1))
                ),
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

#[derive(Debug, Clone)]
pub enum TemplateString {
    Ptr(*const str),
    Owned(String),
}

impl PartialEq for TemplateString {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Owned(s), Self::Owned(o)) => s == o,
            (Self::Ptr(s), Self::Ptr(o)) => unsafe { s.as_ref() == o.as_ref() },
            (Self::Ptr(s), Self::Owned(o)) => unsafe { s.as_ref() == Some(o) },
            (Self::Owned(s), Self::Ptr(o)) => unsafe { o.as_ref() == Some(s) },
        }
    }
}

impl TemplateString {
    pub fn get_string(&self) -> &str {
        match self {
            TemplateString::Owned(s) => s,
            TemplateString::Ptr(s) => unsafe {
                s.as_ref()
                    .expect("Values in TemplateString::Prt should point to a valid string")
            },
        }
    }
}

#[cfg(test)]
mod tests {
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
}
