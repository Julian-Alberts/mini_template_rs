#[cfg(not(feature = "disable_assign"))]
mod assign;
mod calculated_value;
mod conditional;
mod statement;
mod storage_method;

#[cfg(not(feature = "disable_assign"))]
pub use assign::Assign;
pub use calculated_value::CalculatedValue;
pub use conditional::*;
pub use statement::Statement;
pub use storage_method::StorageMethod;

use crate::{error::Result, renderer::RenderContext, variable_container::VariableContainer};

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement>,
}

impl Render for Template {
    fn render<VC: VariableContainer>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut String,
    ) -> Result<()> {
        self.tpl.render(context, buf)
    }
}

pub trait Render {
    fn render<VC: VariableContainer>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut String,
    ) -> Result<()>;
}

impl Render for Vec<Statement> {
    fn render<VC: VariableContainer>(
        &self,
        context: &mut RenderContext<VC>,
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
                Statement::Condition(c) => c.render(context, buf)?,
                #[cfg(not(feature = "disable_assign"))]
                Statement::Assign(a) => a.assign(context)?,
            }
        }

        Ok(())
    }
}
