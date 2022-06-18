use crate::renderer::RenderContext;

#[cfg(feature = "condition")]
use super::condition::{Condition, ConditionEval};
use super::{Render, Statement};

#[derive(PartialEq, Debug)]
pub struct Loop {
    condition: Condition,
    template: Vec<Statement>,
}

impl Loop {
    pub fn new(condition: Condition, template: Vec<Statement>) -> Self {
        Self {
            condition,
            template,
        }
    }
}

impl Render for Loop {
    fn render(&self, context: &mut RenderContext, buf: &mut String) -> crate::error::Result<()> {
        while self.condition.eval(context)? {
            self.template.render(context, buf)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::template::Modifier;
    use crate::value::ident::Ident;
    use crate::{
        renderer::RenderContext,
        template::{condition::Condition, Assign, CalculatedValue, Render, Statement},
        value::StorageMethod,
    };
    use serde_json::Value;

    use super::Loop;

    #[test]
    fn loop_single_iteration() {
        let l = Loop::new(
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("var")),
                vec![],
            )),
            vec![
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![],
                )),
                Statement::Assign(Assign::new(
                    Ident::new_static("var"),
                    CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var")),
                        vec![
                            Modifier {
                                name: "sub",
                                args: vec![StorageMethod::Const(json!(1_f64))],
                                span: Default::default(),
                            },
                            Modifier {
                                name: "floor",
                                args: vec![],
                                span: Default::default()
                            }
                        ],
                    ),
                )),
            ],
        );

        let mut modifiers = HashMap::default();
        let sub: &'static crate::modifier::Modifier = &crate::modifier::sub;
        modifiers.insert("sub", sub);
        let floor: &'static crate::modifier::Modifier = &crate::modifier::floor;
        modifiers.insert("floor", floor);
        let templates = HashMap::<String, _>::new();
        let vars = json!({
            "var": 1
        })
        .try_into()
        .unwrap();
        let mut ctx = RenderContext::new(&modifiers, vars, &templates);
        let mut buffer = String::new();
        assert!(l.render(&mut ctx, &mut buffer).is_ok());
        assert_eq!(buffer.as_str(), "1")
    }

    #[test]
    fn loop_multiple_iterations() {
        let l = Loop::new(
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("var")),
                vec![],
            )),
            vec![
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![],
                )),
                Statement::Assign(Assign::new(
                    Ident::new_static("var"),
                    CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var")),
                        vec![Modifier {
                            name: "sub",
                            args: vec![StorageMethod::Const(json!(1))],
                            span: Default::default(),
                        },
                        Modifier {
                            name: "floor",
                            args: vec![],
                            span: Default::default()
                        }],
                    ),
                )),
            ],
        );

        let mut modifiers = HashMap::default();
        let sub: &'static crate::modifier::Modifier = &crate::modifier::sub;
        modifiers.insert("sub", sub);
        let floor: &'static crate::modifier::Modifier = &crate::modifier::floor;
        modifiers.insert("floor", floor);
        let templates = HashMap::<String, _>::new();
        let vars = json!({
            "var": 5
        })
        .try_into()
        .unwrap();
        let mut ctx = RenderContext::new(&modifiers, vars, &templates);
        let mut buffer = String::new();
        assert!(l.render(&mut ctx, &mut buffer).is_ok());
        assert_eq!(buffer.as_str(), "54321")
    }

    #[test]
    fn loop_no_iterations() {
        let l = Loop::new(
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Const(Value::Bool(false)),
                vec![],
            )),
            vec![Statement::Literal("TEST")],
        );

        let modifiers = HashMap::new();
        let templates = HashMap::<String, _>::new();
        let vars = json!({
            "var": 5_f64
        })
        .try_into()
        .unwrap();
        let mut ctx = RenderContext::new(&modifiers, vars, &templates);
        let mut buffer = String::new();
        assert!(l.render(&mut ctx, &mut buffer).is_ok());
        assert!(buffer.is_empty())
    }
}
