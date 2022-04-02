#[derive(Debug, Clone)]
pub struct Span {
    pub input: *const str,
    pub start: usize,
    pub end: usize,
}

impl<'a> From<pest::Span<'a>> for Span {
    fn from(span: pest::Span) -> Self {
        Self {
            input: span.as_str(),
            start: span.start(),
            end: span.end(),
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            input: "" as *const _,
            start: 0,
            end: 0,
        }
    }
}
