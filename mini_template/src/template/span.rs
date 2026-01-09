#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub input: &'static str,
    pub start: usize,
    pub end: usize,
}

impl From<pest::Span<'static>> for Span {
    fn from(span: pest::Span<'static>) -> Self {
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
            input: "",
            start: 0,
            end: 0,
        }
    }
}
