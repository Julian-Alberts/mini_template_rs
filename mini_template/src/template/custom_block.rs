use super::Render;

/// Custom blocks are currently unstable and subject to change.
pub trait CustomBlockParser {
    fn name(&self) -> &str;

    fn parse(
        &self,
        args: &str,
        input: &str,
    ) -> Result<Box<dyn CustomBlock>, crate::parser::ParseError>;
}

pub trait CustomBlock: std::fmt::Debug + Render {}
