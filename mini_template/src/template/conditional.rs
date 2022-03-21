use std::fmt::Debug;

use crate::{renderer::RenderContext, value_container::ValueContainer};

use super::{
    condition::{Condition, ConditionEval},
    Render, Statement,
};

#[derive(Debug, PartialEq)]
pub struct Conditional {
    pub(crate) condition: Condition,
    pub(crate) then_case: Vec<Statement>,
    pub(crate) else_case: Option<Vec<Statement>>,
}

impl Render for Conditional {
    fn render<VC: ValueContainer>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut String,
    ) -> crate::error::Result<()> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        renderer::RenderContext,
        template::{
            condition::{AndCondition, Condition, ConditionEval, OrCondition},
            CalculatedValue,
        },
        value::{Value, StorageMethod},
    };

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
