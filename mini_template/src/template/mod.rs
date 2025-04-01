mod parser;

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
mod storage_method;

#[cfg(feature = "assign")]
pub use assign::Assign;
pub use calculated_value::CalculatedValue;
#[cfg(feature = "conditional")]
pub use conditional::*;
#[cfg(feature = "loop")]
pub use loops::Loop;
pub use statement::Statement;
pub use storage_method::StorageMethod;

use crate::{error::Result, renderer::RenderContext, variable_container::VariableContainer};

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement>,
}

impl Render for Template {
    fn render<VC: VariableContainer, W: std::io::Write>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut W,
    ) -> Result<()> {
        self.tpl.render(context, buf)
    }
}

pub trait Render {
    fn render<VC: VariableContainer, W: std::io::Write>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut W,
    ) -> Result<()>;
}

impl Render for Vec<Statement> {
    fn render<VC: VariableContainer, W: std::io::Write>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut W,
    ) -> Result<()> {
        for statement in self {
            match statement {
                Statement::Literal(literal) =>
                // Safety: literal points to tpl.tpl_str and should never be null
                unsafe {
                    buf.write_all(literal.as_ref().unwrap().as_bytes())?;
                },
                Statement::Calculated(cv) => {
                    let var = cv.calc(context)?;
                    buf.write_all(&var.to_string()[..].as_bytes())?;
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{parser::parse, template::Render, value::Value};

    #[test]
    fn move_template() {
        let template = parse("template {{test}} template".to_string()).unwrap();
        let old_addr = &template as *const _ as usize;
        let template = Box::new(template);
        let new_addr = template.as_ref() as *const _ as usize;
        assert_ne!(new_addr, old_addr);
        let mut buf = Vec::new();
        template
            .render(
                &mut crate::renderer::RenderContext {
                    modifier: &Default::default(),
                    variables: HashMap::from_iter([("test".to_string(), Value::Number(12.0))]),
                },
                &mut buf,
            )
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "template 12 template");
    }
}
