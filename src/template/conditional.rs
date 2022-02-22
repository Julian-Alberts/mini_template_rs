use crate::renderer::RenderContext;

use super::{CalcualtedValue, Statement};

#[derive(Debug, PartialEq)]
pub struct Conditional {
    pub(crate) condition: Condition,
    pub(crate) then_case: Vec<Statement>,
    pub(crate) else_case: Option<Vec<Statement>>,
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Compare(CompareCondition),
    Simple(CalcualtedValue),
}

impl Condition {
    pub fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        let b = match self {
            Self::Simple(s) => s.calc(context)?.as_bool(),
            Self::Compare(c) => unimplemented!(),
        };

        Ok(b)
    }
}

#[derive(Debug, PartialEq)]
pub struct CompareCondition {
    pub(crate) left: CalcualtedValue,
    pub(crate) operator: CompareOperator,
    pub(crate) right: CalcualtedValue,
}

#[derive(Debug, PartialEq)]
pub enum CompareOperator {
    EQ,
    NE,
    LT,
    LE,
    GT,
    GE,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        renderer::RenderContext,
        template::{CalcualtedValue, StorageMethod},
        value::Value,
    };

    use super::Condition;

    #[test]
    fn eval_simple_bool_true() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Bool(true));
        let condition = Condition::Simple(CalcualtedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), &vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_false() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Bool(false));
        let condition = Condition::Simple(CalcualtedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), &vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_false() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(0.));
        let condition = Condition::Simple(CalcualtedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), &vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_1_0() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(1.));
        let condition = Condition::Simple(CalcualtedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), &vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_10() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(10.));
        let condition = Condition::Simple(CalcualtedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), &vars))
            .unwrap());
    }
}
