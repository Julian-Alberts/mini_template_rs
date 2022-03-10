use std::fmt::Debug;

use crate::{renderer::RenderContext, variable_container::VariableContainer};

use super::{CalculatedValue, Render, Statement};

#[derive(Debug, PartialEq)]
pub struct Conditional {
    pub(crate) condition: Condition,
    pub(crate) then_case: Vec<Statement>,
    pub(crate) else_case: Option<Vec<Statement>>,
}

impl Render for Conditional {
    fn render<VC: VariableContainer>(&self, context: &RenderContext<VC>, buf: &mut String) -> crate::error::Result<()> {
        if self.condition.eval(context)? {
            self.then_case.render(context, buf)
        } else {
            if let Some(e) = &self.else_case {
                e.render(context, buf)
            } else {
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Or(OrCondition),
    And(AndCondition),
    CalculatedValue(CalculatedValue),
    Compare(CompareCondition),
}

#[cfg(test)]
impl Condition {
    pub fn and(and: Vec<Condition>) -> Condition {
        Condition::And(AndCondition::new(and))
    }

    pub fn or(and: Vec<Condition>) -> Condition {
        Condition::Or(OrCondition::new(and))
    }
}

impl ConditionEval for Condition {
    fn eval<VC: VariableContainer>(&self, context: &RenderContext<VC>) -> crate::error::Result<bool> {
        match self {
            Self::Or(c) => c.eval(context),
            Self::And(c) => c.eval(context),
            Self::Compare(c) => c.eval(context),
            Self::CalculatedValue(c) => Ok(c.calc(context)?.as_bool()),
        }
    }
}

pub trait ConditionEval {
    fn eval<VC: VariableContainer>(&self, context: &RenderContext<VC>) -> crate::error::Result<bool>;
}

#[derive(Debug, PartialEq)]
pub struct OrCondition {
    conditions: Vec<Condition>,
}

impl OrCondition {
    pub fn new(conditions: Vec<Condition>) -> Self {
        Self { conditions }
    }
}

impl ConditionEval for OrCondition {
    fn eval<VC: VariableContainer>(&self, context: &RenderContext<VC>) -> crate::error::Result<bool> {
        for condition in &self.conditions {
            if condition.eval(context)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Debug, PartialEq)]
pub struct AndCondition {
    conditions: Vec<Condition>,
}

impl AndCondition {
    pub fn new(conditions: Vec<Condition>) -> Self {
        Self { conditions }
    }
}

impl ConditionEval for AndCondition {
    fn eval<VC: VariableContainer>(&self, context: &RenderContext<VC>) -> crate::error::Result<bool> {
        for condition in &self.conditions {
            if !condition.eval(context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Debug, PartialEq)]
pub struct CompareCondition {
    pub(crate) left: CalculatedValue,
    pub(crate) operator: CompareOperator,
    pub(crate) right: CalculatedValue,
}

impl ConditionEval for CompareCondition {
    fn eval<VC: VariableContainer>(&self, context: &RenderContext<VC>) -> crate::error::Result<bool> {
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
        template::{AndCondition, CalculatedValue, Condition, OrCondition, StorageMethod},
        value::Value,
    };

    use super::ConditionEval;

    #[test]
    fn eval_condition() {
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Const(Value::Bool(true)),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), HashMap::new()))
            .unwrap())
    }

    #[test]
    fn eval_condition_and() {
        let condition = AndCondition::new(vec![
            Condition::CalculatedValue(CalculatedValue::new(StorageMethod::Variable("a"), vec![])),
            Condition::CalculatedValue(CalculatedValue::new(StorageMethod::Variable("b"), vec![])),
        ]);
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_condition_or() {
        let condition = OrCondition::new(vec![
            Condition::CalculatedValue(CalculatedValue::new(StorageMethod::Variable("a"), vec![])),
            Condition::CalculatedValue(CalculatedValue::new(StorageMethod::Variable("b"), vec![])),
        ]);
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(true));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(true));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
        let mut vars = HashMap::new();
        vars.insert("a".to_owned(), Value::Bool(false));
        vars.insert("b".to_owned(), Value::Bool(false));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_true() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Bool(true));
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_false() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Bool(false));
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_false() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(0.));
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_1_0() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(1.));
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_10() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_owned(), Value::Number(10.));
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable("my_var"),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars))
            .unwrap());
    }

    #[test]
    fn eval_complex_rule() {
        //(var1 || var2) && var3
        let condition = Condition::and(vec![
            Condition::or(vec![
                Condition::CalculatedValue(CalculatedValue::new(
                    StorageMethod::Variable("var1"),
                    vec![],
                )),
                Condition::CalculatedValue(CalculatedValue::new(
                    StorageMethod::Variable("var2"),
                    vec![],
                )),
            ]),
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable("var3"),
                vec![],
            )),
        ]);
        let mods = HashMap::default();
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(false));
        vars.insert("var2".to_owned(), Value::Bool(false));
        vars.insert("var3".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(true));
        vars.insert("var2".to_owned(), Value::Bool(false));
        vars.insert("var3".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(false));
        vars.insert("var2".to_owned(), Value::Bool(true));
        vars.insert("var3".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(true));
        vars.insert("var2".to_owned(), Value::Bool(true));
        vars.insert("var3".to_owned(), Value::Bool(false));
        assert!(!condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(false));
        vars.insert("var2".to_owned(), Value::Bool(false));
        vars.insert("var3".to_owned(), Value::Bool(true));
        assert!(!condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(true));
        vars.insert("var2".to_owned(), Value::Bool(false));
        vars.insert("var3".to_owned(), Value::Bool(true));
        assert!(condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(false));
        vars.insert("var2".to_owned(), Value::Bool(true));
        vars.insert("var3".to_owned(), Value::Bool(true));
        assert!(condition.eval(&RenderContext::new(&mods, vars)).unwrap());
        let mut vars = HashMap::new();
        vars.insert("var1".to_owned(), Value::Bool(true));
        vars.insert("var2".to_owned(), Value::Bool(true));
        vars.insert("var3".to_owned(), Value::Bool(true));
        assert!(condition.eval(&RenderContext::new(&mods, vars)).unwrap());
    }
}
