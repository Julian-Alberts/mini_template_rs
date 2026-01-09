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

use crate::{error::Result, renderer::RenderContext};

/// Only for internal use to store the template string
pub(crate) enum TemplateStr<'a> {
    Boxed(*mut str),
    Ref(&'a str),
}

impl TemplateStr<'_> {
    pub(crate) fn new(s: String) -> Self {
        let str_box = s.into_boxed_str();
        let str_box_ref = Box::leak(str_box);
        Self::Boxed(str_box_ref)
    }

    pub(crate) fn new_boxed(s: Box<String>) -> Self {
        let s = s.into_boxed_str();
        let str_box_ref = Box::leak(s);
        Self::Boxed(str_box_ref)
    }

    pub(crate) fn from_static(s: &'static str) -> Self {
        Self::Ref(s)
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            TemplateStr::Boxed(ptr) => unsafe {
                // Safety: ptr was created from a Box<str> in TemplateStringBox::new
                &**ptr
            },
            TemplateStr::Ref(s) => s,
        }
    }
}

impl PartialEq for TemplateStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl std::fmt::Debug for TemplateStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("TemplateStr");
        s.field("value", &self.as_str()).finish()?;
        s.finish()
    }
}

impl From<&'static str> for TemplateStr<'_> {
    fn from(s: &'static str) -> Self {
        TemplateStr::from_static(s)
    }
}

impl From<String> for TemplateStr<'_> {
    fn from(s: String) -> Self {
        TemplateStr::new(s)
    }
}

impl From<Box<String>> for TemplateStr<'_> {
    fn from(s: Box<String>) -> Self {
        TemplateStr::new_boxed(s)
    }
}

impl Drop for TemplateStr<'_> {
    fn drop(&mut self) {
        match self {
            TemplateStr::Boxed(ptr) => {
                // Safety: ptr was created from a Box<str> in TemplateStringBox::new
                unsafe {
                    let _ = Box::from_raw(*ptr);
                }
            }
            TemplateStr::Ref(_) => {}
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: TemplateStr<'static>,
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
                    buf.push_str(&var.to_string()[..])
                }
                #[cfg(feature = "conditional")]
                Statement::Condition(c) => c.render(context, buf)?,
                #[cfg(feature = "assign")]
                Statement::Assign(a) => a.assign(context)?,
                #[cfg(feature = "loop")]
                Statement::Loop(l) => l.render(context, buf)?,
                #[cfg(feature = "include")]
                Statement::Include(i) => i.render(context, buf)?,
                Statement::CustomBlock(cb) => cb.render(context, buf)?,
                Statement::ForceNewLine => buf.push('\n'),
            }
        }

        Ok(())
    }
}
