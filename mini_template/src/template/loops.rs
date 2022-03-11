use crate::{renderer::RenderContext, variable_container::VariableContainer};

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
    fn render<VC: VariableContainer>(
        &self,
        context: &mut RenderContext<VC>,
        buf: &mut String,
    ) -> crate::error::Result<()> {
        while self.condition.eval(context)? {
            self.template.render(context, buf)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        renderer::RenderContext,
        template::{
            condition::Condition, Assign, CalculatedValue, Render, Statement, StorageMethod,
        },
        value::Value,
    };

    use super::Loop;

    #[test]
    fn loop_single_iteration() {
        let l = Loop::new(
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable("var"),
                vec![],
            )),
            vec![
                Statement::Calculated(CalculatedValue::new(StorageMethod::Variable("var"), vec![])),
                Statement::Assign(Assign::new(
                    "var",
                    CalculatedValue::new(
                        StorageMethod::Variable("var"),
                        vec![("sub", vec![StorageMethod::Const(Value::Number(1.))])],
                    ),
                )),
            ],
        );

        let mut modifiers = HashMap::default();
        let sub: &'static crate::modifier::Modifier = &crate::modifier::sub;
        modifiers.insert("sub", sub);
        let mut ctx = RenderContext::new(
            &modifiers,
            HashMap::from_iter([("var".to_owned(), Value::Number(1.))]),
        );
        let mut buffer = String::new();
        assert!(l.render(&mut ctx, &mut buffer).is_ok());
        assert_eq!(buffer.as_str(), "1")
    }

    #[test]
    fn loop_multiple_iterations() {
        let l = Loop::new(
            Condition::CalculatedValue(CalculatedValue::new(
                StorageMethod::Variable("var"),
                vec![],
            )),
            vec![
                Statement::Calculated(CalculatedValue::new(StorageMethod::Variable("var"), vec![])),
                Statement::Assign(Assign::new(
                    "var",
                    CalculatedValue::new(
                        StorageMethod::Variable("var"),
                        vec![("sub", vec![StorageMethod::Const(Value::Number(1.))])],
                    ),
                )),
            ],
        );

        let mut modifiers = HashMap::default();
        let sub: &'static crate::modifier::Modifier = &crate::modifier::sub;
        modifiers.insert("sub", sub);
        let mut ctx = RenderContext::new(
            &modifiers,
            HashMap::from_iter([("var".to_owned(), Value::Number(5.))]),
        );
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
        let mut ctx = RenderContext::new(
            &modifiers,
            HashMap::from_iter([("var".to_owned(), Value::Number(5.))]),
        );
        let mut buffer = String::new();
        assert!(l.render(&mut ctx, &mut buffer).is_ok());
        assert!(buffer.is_empty())
    }
}
