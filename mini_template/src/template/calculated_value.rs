use std::borrow::Cow;

use crate::{
    renderer::RenderContext,
    value::{StorageMethod, Value, VariableManager},
};

#[derive(Debug)]
pub struct CalculatedValue {
    value: StorageMethod,
    modifiers: Vec<(*const str, Vec<StorageMethod>)>,
}

impl CalculatedValue {
    pub fn new(value: StorageMethod, modifiers: Vec<(*const str, Vec<StorageMethod>)>) -> Self {
        Self { value, modifiers }
    }

    pub fn calc<VM: VariableManager>(
        &self,
        context: &RenderContext<VM>,
    ) -> crate::error::Result<Value> {
        let mut var = match &self.value {
            StorageMethod::Const(var) => Cow::Borrowed(var),
            StorageMethod::Variable(ident) => {
                let var = context.variables.get(ident)?;
                Cow::Borrowed(var)
            }
        };

        for (modifier_name, args) in &self.modifiers {
            // Safety: modifier_name points to tpl.tpl_str and should never be null
            let modifier_name = unsafe { modifier_name.as_ref().unwrap() };
            let modifier =
                context
                    .modifier
                    .get(modifier_name)
                    .ok_or(crate::error::Error::UnknownModifier(
                        modifier_name.to_string(),
                    ))?;

            let args = storage_methods_to_values(args, &context.variables)?;

            var = match modifier(&var, args) {
                Ok(v) => Cow::Owned(v),
                Err(e) => {
                    let error = e.to_string();
                    error!("{}", error);
                    return Err(crate::error::Error::Modifier(e));
                }
            };
        }

        Ok(var.into_owned())
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

impl PartialEq for CalculatedValue {
    fn eq(&self, other: &Self) -> bool {
        if self.value != other.value {
            return false;
        }

        self.modifiers.iter().zip(&other.modifiers).all(|(s, o)|
            // Safety: Both modifier names point to positions in the original template string.
            unsafe {
                s.0.as_ref() == o.0.as_ref() && s.1 == o.1
            })
    }
}
