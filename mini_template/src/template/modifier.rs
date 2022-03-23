use crate::template::Span;
use crate::value::{StorageMethod, Value};
use crate::{RenderContext, VariableManager};

#[derive(Debug)]
pub struct Modifier {
    pub name: *const str,
    pub args: Vec<StorageMethod>,
    pub span: Span,
}

impl Modifier {
    pub fn eval<VM: VariableManager>(
        &self,
        value: &Value,
        context: &RenderContext<VM>,
    ) -> crate::error::Result<Value> {
        // Safety: modifier_name points to tpl.tpl_str and should never be null
        let modifier_name = unsafe { self.name.as_ref().unwrap() };
        let modifier =
            *context
                .modifier
                .get(modifier_name)
                .ok_or(crate::error::Error::UnknownModifier(
                    UnknownModifierError{
                        name: modifier_name.to_string(),
                        span: self.span.clone()
                    }
                ))?;

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
    pub span: Span
}

impl PartialEq for UnknownModifierError {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }

}

fn storage_methods_to_values<'a>(
    args: &'a [StorageMethod],
    variables: &'a dyn VariableManager,
) -> crate::error::Result<Vec<&'a Value>> {
    let mut real_args = Vec::with_capacity(args.len());

    for arg in args {
        let arg = match arg {
            StorageMethod::Const(value) => value,
            StorageMethod::Variable(ident) => variables.get(ident)?,
        };
        real_args.push(arg);
    }
    Ok(real_args)
}
