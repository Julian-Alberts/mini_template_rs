use std::borrow::Cow;

use crate::{renderer::RenderContext, value::Value, value_container::ValueContainer};

use super::StorageMethod;

#[derive(Debug)]
pub struct CalculatedValue {
    value: StorageMethod,
    modifiers: Vec<(*const str, Vec<StorageMethod>)>,
}

impl CalculatedValue {
    pub fn new(value: StorageMethod, modifiers: Vec<(*const str, Vec<StorageMethod>)>) -> Self {
        Self { value, modifiers }
    }

    pub fn calc<VC: ValueContainer>(
        &self,
        context: &RenderContext<VC>,
    ) -> crate::error::Result<Value> {
        let mut var = match &self.value {
            StorageMethod::Const(var) => Cow::Borrowed(var),
            StorageMethod::Variable(var_name) => {
                // Safety: var_name points to tpl.tpl_str and should never be null
                let var_name = unsafe { var_name.as_ref().unwrap() };
                let var = context.variables.get(var_name);
                Cow::Borrowed(var.ok_or(crate::error::Error::UnknownVariable(var_name))?)
            }
        };

        for (modifier_name, args) in &self.modifiers {
            // Safety: modifier_name points to tpl.tpl_str and should never be null
            let modifier_name = unsafe { modifier_name.as_ref().unwrap() };
            let modifier = context
                .modifier
                .get(modifier_name)
                .ok_or(crate::error::Error::UnknownModifier(modifier_name))?;

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

fn storage_methods_to_values<'a, 't>(
    args: &'a [StorageMethod],
    variables: &'a dyn ValueContainer,
) -> crate::error::Result<'t, Vec<&'a Value>> {
    let mut real_args = Vec::with_capacity(args.len());

    for arg in args {
        let arg = match arg {
            StorageMethod::Const(value) => value,
            StorageMethod::Variable(var) =>
            //Safety: var points to tpl.tpl_str and should never be null
            unsafe {
                let var = var.as_ref().unwrap();
                variables
                    .get(var)
                    .ok_or(crate::error::Error::UnknownVariable(var))?
            },
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
