#[cfg(feature = "assign")]
mod assign;
mod calculated_value;
#[cfg(feature = "condition")]
pub mod condition;
#[cfg(feature = "conditional")]
mod conditional;
mod custom_block;
#[cfg(feature = "include")]
mod include;
#[cfg(feature = "loop")]
mod loops;
mod modifier;
mod span;
mod statement;

#[cfg(feature = "assign")]
pub use assign::Assign;
pub use calculated_value::CalculatedValue;
#[cfg(feature = "conditional")]
pub use conditional::*;
pub use custom_block::*;
#[cfg(feature = "include")]
pub use include::Include;
#[cfg(feature = "loop")]
pub use loops::Loop;
pub use modifier::{Modifier, UnknownModifierError};
pub use span::Span;
pub use statement::Statement;

use crate::{error::Result, renderer::RenderContext, prelude::ValueAs};

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement>,
}

impl Render for Template {
    fn render<'a>(&self, context: &mut RenderContext, buf: &mut String) -> Result<()> {
        self.tpl.render(context, buf)
    }
}

pub trait Render {
    fn render(&self, context: &mut RenderContext, buf: &mut String) -> Result<()>;
}

impl Render for Vec<Statement> {
    fn render(&self, context: &mut RenderContext, buf: &mut String) -> Result<()> {
        for statement in self {
            match statement {
                Statement::Literal(literal) =>
                // Safety: literal points to tpl.tpl_str and should never be null
                unsafe { buf.push_str(literal.as_ref().unwrap()) },
                Statement::Calculated(cv) => {
                    let var = cv.calc(context)?;
                    buf.push_str(&ValueAs::as_string(&var)[..])
                }
                #[cfg(feature = "conditional")]
                Statement::Conditional(c) => c.render(context, buf)?,
                #[cfg(feature = "assign")]
                Statement::Assign(a) => a.assign(context)?,
                #[cfg(feature = "loop")]
                Statement::Loop(l) => l.render(context, buf)?,
                #[cfg(feature = "include")]
                Statement::Include(i) => i.render(context, buf)?,
                Statement::CustomBlock(cb) => cb.render(context, buf)?,
            }
        }

        Ok(())
    }
}
