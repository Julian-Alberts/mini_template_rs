use crate::template::Span;
use crate::value::{StorageMethod, Value};
use crate::RenderContext;

#[derive(Debug)]
pub struct Modifier {
    pub name: &'static str,
    pub args: Vec<StorageMethod>,
    pub span: Span,
}

impl Modifier {
    pub fn eval(&self, value: &Value, context: &RenderContext) -> crate::error::Result<Value> {
        let modifier = context.modifier.get(self.name).ok_or_else(|| {
            crate::error::Error::UnknownModifier(UnknownModifierError {
                name: self.name.to_string(),
                span: self.span.clone(),
            })
        })?;

        let args = storage_methods_to_values(&self.args, context)?;
        match modifier.call(value, args) {
            Ok(v) => Ok(v),
            Err(e) => Err(crate::error::Error::Modifier(e)),
        }
    }
}

impl PartialEq for Modifier {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.args == other.args
    }
}

#[derive(Debug)]
pub struct UnknownModifierError {
    pub name: String,
    pub span: Span,
}

impl PartialEq for UnknownModifierError {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn storage_methods_to_values<'a>(
    args: &'a [StorageMethod],
    context: &'a RenderContext,
) -> crate::error::Result<Vec<&'a Value>> {
    let mut real_args = Vec::with_capacity(args.len());

    for arg in args {
        let arg = arg.get_value(context)?;
        real_args.push(arg);
    }
    Ok(real_args)
}
