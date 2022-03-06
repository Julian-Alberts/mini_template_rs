use std::fmt::Debug;

use crate::renderer::RenderContext;

use super::{CalcualtedValue, Statement};

#[derive(Debug)]
pub struct Conditional {
    pub(crate) condition: Condition,
    pub(crate) then_case: Vec<Statement>,
    pub(crate) else_case: Option<Vec<Statement>>,
}

impl PartialEq for Conditional {

    fn eq(&self, other: &Self) -> bool {
        self.then_case == other.then_case &&
        self.else_case == other.else_case
    }

}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Or(OrCondition),
    And(AndCondition),
    CalculatedValue(CalcualtedValue),
    Compare(CompareCondition)
}

impl Condition {

    pub fn and(and: Vec<Condition>) -> Condition {
        Condition::And(AndCondition::new(and))
    }

    pub fn or(and: Vec<Condition>) -> Condition {
        Condition::Or(OrCondition::new(and))
    }

}

impl ConditionEval for Condition {

    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        match self {
            Self::Or(c) => c.eval(context),
            Self::And(c) => c.eval(context),
            Self::Compare(c) => c.eval(context),
            Self::CalculatedValue(c) => Ok(c.calc(context)?.as_bool())
        }
    }

}

pub trait ConditionEval {
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool>;
}

#[derive(Debug, PartialEq)]
pub struct OrCondition {
    conditions: Vec<Condition>
}

impl OrCondition {

    pub fn new(conditions: Vec<Condition>) -> Self {
        Self {
            conditions
        }
    }

}

impl ConditionEval for OrCondition {

    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        for condition in &self.conditions {
            if condition.eval(context)? {
                return Ok(true)
            }
        }
        Ok(false)
    }

}

#[derive(Debug, PartialEq)]
pub struct AndCondition {
    conditions: Vec<Condition>
}

impl AndCondition {

    pub fn new(conditions: Vec<Condition>) -> Self {
        Self {
            conditions
        }
    }

}

impl ConditionEval for AndCondition {

    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        for condition in &self.conditions {
            if !condition.eval(context)? {
                return Ok(false)
            }
        }
        Ok(true)
    }

}

#[derive(Debug, PartialEq)]
pub enum ConditionKind {
    Compare(CompareCondition),
    Simple(CalcualtedValue),
}

impl ConditionEval for ConditionKind {
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
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

impl ConditionEval for CompareCondition {

    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        let left = self.left.calc(context)?;
        let right = self.right.calc(context)?;
        let r = match self.operator {
            CompareOperator::EQ => left == right,
            CompareOperator::NE => left != right,
            CompareOperator::LT => left < right,
            CompareOperator::LE => left <= right,
            CompareOperator::GT => left > right,
            CompareOperator::GE => left >= right,
        };
        Ok(r)
    }

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
        template::{CalcualtedValue, StorageMethod, ConditionKind, AndCondition, OrCondition, Condition},
        value::Value,
    };

    use super::ConditionEval;

    #[test]
    fn eval_condition() {
        let condition = ConditionKind::Simple(
                CalcualtedValue::new(StorageMethod::Const(Value::Bool(true)),vec![])
        );
        assert!(condition.eval(&RenderContext::new(&HashMap::new(), &HashMap::new())).unwrap())
    }

    #[test]
    fn eval_condition_and() {
        let condition = AndCondition::new(vec![
                Condition::CalculatedValue(
                    CalcualtedValue::new(StorageMethod::Variable("a"),vec![])
                ),
                Condition::CalculatedValue(
                        CalcualtedValue::new(StorageMethod::Variable("b"),vec![])
                )
            ]
        );
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(!condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
    }

    #[test]
    fn eval_condition_or() {
        let condition = OrCondition::new(vec![
            Condition::CalculatedValue(
                CalcualtedValue::new(StorageMethod::Variable("a"),vec![])
            ),
            Condition::CalculatedValue(
                CalcualtedValue::new(StorageMethod::Variable("b"),vec![])
            )
        ]
    );
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&HashMap::new(), &vars)).unwrap());
    }

    #[test]
    fn eval_simple_bool_true() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Bool(true));
        let condition = ConditionKind::Simple(CalcualtedValue::new(
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
        let condition = ConditionKind::Simple(CalcualtedValue::new(
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
        let condition = ConditionKind::Simple(CalcualtedValue::new(
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
        let condition = ConditionKind::Simple(CalcualtedValue::new(
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
        let condition = ConditionKind::Simple(CalcualtedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), &vars))
            .unwrap());
    }
}
