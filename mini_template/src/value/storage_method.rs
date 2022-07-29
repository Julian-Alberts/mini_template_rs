use std::fmt::Debug;

use crate::{value::{ident::Ident, Value}, RenderContext};

pub enum StorageMethod {
    Const(Value),
    Variable(Ident),
}

impl StorageMethod {

    pub fn get_value<'a, 'b>(&'a self, context: &'b RenderContext) -> crate::error::Result<&'a Value> 
        where 'b: 'a
    {
        let var = match &self {
            StorageMethod::Const(var) => var,
            StorageMethod::Variable(ident) => {
                context
                    .variables
                    .get_value(ident.resolve_ident(&context.variables)?)?
            }
        };
        Ok(var)
    }

}

impl PartialEq for StorageMethod {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StorageMethod::Const(s), StorageMethod::Const(o)) => s == o,
            (StorageMethod::Variable(s), StorageMethod::Variable(o)) => s == o,
            _ => false,
        }
    }
}

impl Debug for StorageMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const(c) => write!(f, "Const({:?})", c),
            Self::Variable(v) => write!(f, "Variable({:?})", v),
        }
    }
}
