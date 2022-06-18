use std::fmt::Debug;

use crate::renderer::RenderContext;

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
    fn render(&self, context: &mut RenderContext, buf: &mut String) -> crate::error::Result<()> {
        if self.condition.eval(context)? {
            self.then_case.render(context, buf)
        } else if let Some(e) = &self.else_case {
            e.render(context, buf)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::value::ident::Ident;
    use crate::{
        renderer::RenderContext,
        template::{
            condition::{AndCondition, Condition, ConditionEval, OrCondition},
            CalculatedValue,
        },
        value::StorageMethod,
        ValueManager,
    };
    use serde_json::Value;

    #[test]
    fn eval_condition() {
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Const(Value::Bool(true)),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(
                &HashMap::new(),
                ValueManager::default(),
                &HashMap::new()
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
        let vars = json!({
            "a": Value::Bool(true),
            "b": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
        let vars = json!({
            "a": Value::Bool(true),
            "b": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
        let vars = json!({
            "a": Value::Bool(false),
            "b": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
        let vars = json!({
            "a": Value::Bool(false),
            "b": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
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
        let vars = json!({
            "a": Value::Bool(true),
            "b": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
        let vars = json!({
            "a": Value::Bool(true),
            "b": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
        let vars = json!({
            "a": Value::Bool(false),
            "b": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
        let vars = json!({
            "a": Value::Bool(false),
            "b": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_true() {
        let vars = json!({
            "my_var": true
        })
        .try_into()
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
    }

    #[test]
    fn eval_simple_bool_false() {
        let vars = json!({
            "my_var": false
        })
        .try_into()
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_false() {
        let vars = json!({
            "my_var": 0_f64
        })
        .try_into()
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(!condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_1_0() {
        let vars = json!({
            "my_var": 1_f64
        })
        .try_into()
        .unwrap();
        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
            .unwrap());
    }

    #[test]
    fn eval_simple_int_true_10() {
        let vars = json!({
            "my_var": 10_f64
        })
        .try_into()
        .unwrap();

        let condition = Condition::CalculatedValue(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static("my_var")),
            vec![],
        ));
        assert!(condition
            .eval(&RenderContext::new(&HashMap::new(), vars, &HashMap::new()))
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
        let mods = HashMap::default();
        let vars = json!({
            "var1": Value::Bool(false),
            "var2": Value::Bool(false),
            "var3": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(true),
            "var2": Value::Bool(false),
            "var3": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(false),
            "var2": Value::Bool(true),
            "var3": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(true),
            "var2": Value::Bool(true),
            "var3": Value::Bool(false)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(false),
            "var2": Value::Bool(false),
            "var3": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(!condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(true),
            "var2": Value::Bool(false),
            "var3": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(false),
            "var2": Value::Bool(true),
            "var3": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());

        let vars = json!({
            "var1": Value::Bool(true),
            "var2": Value::Bool(true),
            "var3": Value::Bool(true)
        })
        .try_into()
        .unwrap();
        assert!(condition
            .eval(&RenderContext::new(&mods, vars, &HashMap::new()))
            .unwrap());
    }
}
