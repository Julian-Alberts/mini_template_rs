mod calcualted_value;
mod conditional;
mod statement;
mod storage_method;

pub use calcualted_value::CalcualtedValue;
pub use conditional::*;
pub use statement::Statement;
pub use storage_method::StorageMethod;

use crate::{error::Result, renderer::RenderContext};

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement>,
}

impl Render for Template {
    fn render(&self, context: &RenderContext, buf: &mut String) -> Result<()> {
        self.tpl.render(context, buf)
    }
}

pub trait Render {
    fn render(&self, context: &RenderContext, buf: &mut String) -> Result<()>;
}

impl Render for Vec<Statement> {
    fn render(&self, context: &RenderContext, buf: &mut String) -> Result<()> {
        for statement in self {
            match statement {
                Statement::Literal(literal) =>
                // Safety: literal points to tpl.tpl_str and should never be null
                unsafe { buf.push_str(literal.as_ref().unwrap()) },
                Statement::Calculated(cv) => {
                    let var = cv.calc(context)?;
                    buf.push_str(&var.to_string()[..])
                }
                Statement::Condition(c) => c.render(context, buf)?,
            }
        }

        Ok(())
    }
}
