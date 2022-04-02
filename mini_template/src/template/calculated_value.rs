use std::borrow::Cow;

use crate::template::modifier::Modifier;
use crate::{
    renderer::RenderContext,
    value::{StorageMethod, Value},
    TemplateKey,
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

    pub fn calc<TK>(&self, context: &RenderContext<TK>) -> crate::error::Result<Value>
    where
        TK: TemplateKey,
    {
        let mut var = match &self.value {
            StorageMethod::Const(var) => Cow::Borrowed(var),
            StorageMethod::Variable(ident) => {
                let var = context
                    .variables
                    .get_value(ident.resolve_ident(&context.variables)?)?;
                Cow::Borrowed(var)
            }
        };

        for modifier in &self.modifiers {
            var = Cow::Owned(modifier.eval(&var, context)?)
        }

        Ok(var.into_owned())
    }
}
