use std::fmt::Debug;

use crate::value::Value;

pub enum StorageMethod {
    Const(Value),
    Variable(*const str),
}

impl PartialEq for StorageMethod {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StorageMethod::Const(s), StorageMethod::Const(o)) => s == o,
            (StorageMethod::Variable(s), StorageMethod::Variable(o)) =>
            // Safety: Both variable names point to positions in the original template string.
            unsafe {
                s.as_ref() == o.as_ref()
            },
            _ => false,
        }
    }
}

impl Debug for StorageMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const(c) => write!(f, "Const({c:#?})"),
            Self::Variable(v) => unsafe {
                write!(f, "Variable({:#?} \"{}\")", v, v.as_ref().unwrap())
            },
        }
    }
}
