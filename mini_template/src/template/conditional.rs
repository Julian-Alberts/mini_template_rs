use std::fmt::Debug;

use crate::renderer::RenderContext;

use super::{
    condition::{Condition, ConditionEval},
    Render, Statement,
};

#[derive(Debug, PartialEq)]
pub struct Conditional {
    pub(crate) condition: Condition,
    pub(crate) then_case: Vec<Statement>,
    pub(crate) else_case: Option<Vec<Statement>>,
}

impl Render for Conditional {
    fn render(&self, context: &mut RenderContext, buf: &mut String) -> crate::error::Result<()> {
        if self.condition.eval(context)? {
            self.then_case.render(context, buf)
        } else if let Some(e) = &self.else_case {
            e.render(context, buf)
        } else {
            Ok(())
        }
    }
}
