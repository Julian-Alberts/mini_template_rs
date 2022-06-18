use crate::{
    prelude::{TplPartialEq, TplPartialOrd},
    renderer::RenderContext,
};

use super::CalculatedValue;

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
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        use crate::prelude::ValueAs;
        match self {
            Self::Or(c) => c.eval(context),
            Self::And(c) => c.eval(context),
            Self::Compare(c) => c.eval(context),
            Self::CalculatedValue(c) => Ok(ValueAs::as_bool(&c.calc(context)?)),
        }
    }
}

pub trait ConditionEval {
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool>;
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
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
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
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
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
    fn eval(&self, context: &RenderContext) -> crate::error::Result<bool> {
        let left = self.left.calc(context)?;
        let right = self.right.calc(context)?;
        let r = match self.operator {
            CompareOperator::EQ => TplPartialEq::eq(&left, &right),
            CompareOperator::NE => TplPartialEq::ne(&left, &right),
            CompareOperator::LT => TplPartialOrd::lt(&left, &right),
            CompareOperator::LE => TplPartialOrd::le(&left, &right),
            CompareOperator::GT => TplPartialOrd::gt(&left, &right),
            CompareOperator::GE => TplPartialOrd::ge(&left, &right),
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
