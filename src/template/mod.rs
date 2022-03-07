mod calcualted_value;
mod conditional;
mod statement;
mod storage_method;

pub use calcualted_value::CalcualtedValue;
pub use conditional::*;
pub use statement::Statement;
pub use storage_method::StorageMethod;

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement>,
}
