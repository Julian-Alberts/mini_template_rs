#[cfg(feature = "assign")]
mod assign;
mod calculated_value;
#[cfg(feature = "condition")]
pub mod condition;
#[cfg(feature = "conditional")]
mod conditional;
#[cfg(feature = "loop")]
mod loops;
mod statement;

#[cfg(feature = "assign")]
pub use assign::Assign;
pub use calculated_value::CalculatedValue;
#[cfg(feature = "conditional")]
pub use conditional::*;
#[cfg(feature = "loop")]
pub use loops::Loop;
pub use statement::Statement;

use crate::{error::Result, renderer::RenderContext, value::VariableManager};

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement>,
}

impl Render for Template {
    fn render<VM: VariableManager>(
        &self,
        context: &mut RenderContext<VM>,
        buf: &mut String,
    ) -> Result<()> {
        self.tpl.render(context, buf)
    }
}

pub trait Render {
    fn render<VM: VariableManager>(
        &self,
        context: &mut RenderContext<VM>,
        buf: &mut String,
    ) -> Result<()>;
}

impl Render for Vec<Statement> {
    fn render<VM: VariableManager>(
        &self,
        context: &mut RenderContext<VM>,
        buf: &mut String,
    ) -> Result<()> {
        for statement in self {
            match statement {
                Statement::Literal(literal) =>
                // Safety: literal points to tpl.tpl_str and should never be null
                unsafe { buf.push_str(literal.as_ref().unwrap()) },
                Statement::Calculated(cv) => {
                    let var = cv.calc(context)?;
                    buf.push_str(&var.to_string()[..])
                }
                #[cfg(feature = "conditional")]
                Statement::Condition(c) => c.render(context, buf)?,
                #[cfg(feature = "assign")]
                Statement::Assign(a) => a.assign(context)?,
                #[cfg(feature = "loop")]
                Statement::Loop(l) => l.render(context, buf)?,
            }
        }

        Ok(())
    }
}
