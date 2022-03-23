use std::borrow::Cow;

use crate::template::modifier::Modifier;
use crate::{
    renderer::RenderContext,
    value::{StorageMethod, Value, VariableManager},
};

#[derive(Debug, PartialEq)]
pub struct CalculatedValue {
    value: StorageMethod,
    modifiers: Vec<Modifier>,
}

impl CalculatedValue {
    pub fn new(value: StorageMethod, modifiers: Vec<Modifier>) -> Self {
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

        for modifier in &self.modifiers {
            var = Cow::Owned(modifier.eval(&var, context)?)
        }

        Ok(var.into_owned())
    }
}
