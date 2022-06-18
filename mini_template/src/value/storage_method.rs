use std::fmt::Debug;

use crate::value::ident::Ident;
use serde_json::Value;

pub enum StorageMethod {
    Const(Value),
    Variable(Ident),
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
