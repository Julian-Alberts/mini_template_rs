use crate::value::Value;

#[derive(Debug)]
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