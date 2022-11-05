use crate::renderer::RenderContext;

use super::CalculatedValue;

#[derive(Debug, PartialEq)]
pub enum Condition {
    Not(Box<Condition>),
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
        match self {
            Self::Or(c) => c.eval(context),
            Self::And(c) => c.eval(context),
            Self::Compare(c) => c.eval(context),
            Self::CalculatedValue(c) => Ok(c.calc(context)?.as_bool()),
            Self::Not(c) => c.eval(context).map(|b| !b),
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
    use crate::modifier::ModifierContainer;
    use crate::template_provider::DefaultTemplateProvider;
    use crate::value::ident::Ident;
    use crate::{
        renderer::RenderContext,
        template::{
            condition::{AndCondition, Condition, ConditionEval, OrCondition},
            CalculatedValue,
        },
        value::{StorageMethod, Value},
        value_iter, ValueManager,
    };

    #[test]
    fn eval_condition() {
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Const(Value::Bool(true)),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                ValueManager::default(),
                &DefaultTemplateProvider::default()
            ))
            .unwrap())
    }

    #[test]
    fn eval_condition_and() {
        let condition = AndCondition::new(vec![
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("a")),
                vec![],
            )),
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("b")),
                vec![],
            )),
        ]);
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(true),
            "b": Value::Bool(true)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(true),
            "b": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(false),
            "b": Value::Bool(true)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(false),
            "b": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_condition_or() {
        let condition = OrCondition::new(vec![
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("a")),
                vec![],
            )),
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("b")),
                vec![],
            )),
        ]);
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(true),
            "b": Value::Bool(true)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(true),
            "b": Value::Bool(false)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(false),
            "b": Value::Bool(true)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
        let vars = ValueManager::try_from_iter(value_iter!(
            "a": Value::Bool(false),
            "b": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_true() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "my_var": Value::Bool(true)
        ))
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_false() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "my_var": Value::Bool(false)
        ))
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_not() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "my_var": Value::Bool(false)
        ))
        .unwrap();
        let condition = Condition::Not(Box::new(Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ))));
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_false() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "my_var": Value::Number(0usize.into())
        ))
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_1_0() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "my_var": Value::Number(1usize.into())
        ))
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_10() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "my_var": Value::Number(10usize.into())
        ))
        .unwrap();

        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(
                &ModifierContainer::default(),
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }

    #[test]
    fn eval_complex_rule() {
        //(var1 || var2) && var3
        let condition = Condition::and(vec![
            Condition::or(vec![
                Condition::CalculatedValue(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("var1")),
                    vec![],
                )),
                Condition::CalculatedValue(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("var2")),
                    vec![],
                )),
            ]),
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("var3")),
                vec![],
            )),
        ]);
        let mods = ModifierContainer::default();
        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(false),
            "var2": Value::Bool(false),
            "var3": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true),
            "var2": Value::Bool(false),
            "var3": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(false),
            "var2": Value::Bool(true),
            "var3": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true),
            "var2": Value::Bool(true),
            "var3": Value::Bool(false)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(false),
            "var2": Value::Bool(false),
            "var3": Value::Bool(true)
        ))
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true),
            "var2": Value::Bool(false),
            "var3": Value::Bool(true)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(false),
            "var2": Value::Bool(true),
            "var3": Value::Bool(true)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());

        let vars = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true),
            "var2": Value::Bool(true),
            "var3": Value::Bool(true)
        ))
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(
                &mods,
                vars,
                &DefaultTemplateProvider::default()
            ))
            .unwrap());
    }
}
