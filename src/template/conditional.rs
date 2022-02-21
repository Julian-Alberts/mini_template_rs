use std::{str::FromStr, collections::HashMap};

use crate::{value::Value, modifier::Modifier};

use super::{Statement, CalcualtedValue};

#[derive(Debug, PartialEq)]
pub struct Conditional {
    pub(crate) condition: Condition,
    pub(crate) then_case: Vec<Statement>,
    pub(crate) else_case: Option<Vec<Statement>>
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Compare(CompareCondition),
    Simple(CalcualtedValue),
}

impl Condition {

    pub fn eval(&self, modifier: &HashMap<&'static str, &Modifier>, variables: &HashMap<String, Value>) -> bool {
        match self {
            Self::Simple(s) => unimplemented!(),
            Self::Compare(c) => unimplemented!()
        }
    }

}

#[derive(Debug, PartialEq)]
pub struct CompareCondition {
    pub(crate) left: CalcualtedValue,
    pub(crate) operator: CompareOperator,
    pub(crate) right: CalcualtedValue
}

#[derive(Debug, PartialEq)]
pub enum CompareOperator {
    EQ,
    NE,
    LT,
    LE,
    GT,
    GE
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{template::{CalcualtedValue, StorageMethod}, value::Value};

    use super::Condition;


    #[test]
    fn eval_simple_int_false() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(0.));
        let condition = Condition::Simple(CalcualtedValue {
            value: StorageMethod::Variable(
                "my_var"
            ),
            modifiers: vec![]
        });
        assert!(!condition.eval(&HashMap::new(), &vars));


    }

}
