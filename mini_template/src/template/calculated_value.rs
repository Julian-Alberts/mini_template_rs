use std::borrow::Cow;

use crate::template::modifier::Modifier;
use crate::{
    renderer::RenderContext,
    value::{StorageMethod, Value},
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

    pub fn calc(&self, context: &RenderContext) -> crate::error::Result<Value> {
        let mut var = Cow::Borrowed(self.value.get_value(context)?);

        for modifier in &self.modifiers {
            var = Cow::Owned(modifier.eval(&var, context)?)
        }

        Ok(var.into_owned())
    }
}
