use crate::template::Span;
use crate::value::StorageMethod;
use crate::{RenderContext, ValueManager};
use serde_json::Value;

#[derive(Debug)]
pub struct Modifier {
    pub name: *const str,
    pub args: Vec<StorageMethod>,
    pub span: Span,
}

impl Modifier {
    pub fn eval(&self, value: &Value, context: &RenderContext) -> crate::error::Result<Value> {
        // Safety: modifier_name points to tpl.tpl_str and should never be null
        let modifier_name = unsafe { self.name.as_ref().unwrap() };
        let modifier = *context.modifier.get(modifier_name).ok_or_else(|| {
            crate::error::Error::UnknownModifier(UnknownModifierError {
                name: modifier_name.to_string(),
                span: self.span.clone(),
            })
        })?;

        let args = storage_methods_to_values(&self.args, &context.variables)?;
        match modifier(value, args) {
            Ok(v) => Ok(v),
            Err(e) => Err(crate::error::Error::Modifier(e)),
        }
    }
}

impl PartialEq for Modifier {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.name.as_ref() == other.name.as_ref() && self.args == other.args }
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
    variables: &'a ValueManager,
) -> crate::error::Result<Vec<&'a Value>> {
    let mut real_args = Vec::with_capacity(args.len());

    for arg in args {
        let arg = match arg {
            StorageMethod::Const(value) => value,
            StorageMethod::Variable(ident) => {
                variables.get_value(ident.resolve_ident(variables)?)?
            }
        };
        real_args.push(arg);
    }
    Ok(real_args)
}
