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
    use crate::modifier::{InsertModifier, ModifierContainer};
    use crate::template::Modifier;
    use crate::template_provider::DefaultTemplateProvider;
    use crate::value::ident::Ident;
    use crate::{
        renderer::RenderContext,
        template::{condition::Condition, Assign, CalculatedValue, Render, Statement},
        value::{StorageMethod, Value},
        value_iter, ValueManager,
    };

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
                        vec![Modifier {
                            name: "sub",
                            args: vec![StorageMethod::Const(Value::Number(1usize.into()))],
                            span: Default::default(),
                        }],
                    ),
                )),
            ],
        );

        let mut modifiers = ModifierContainer::default();
        modifiers.insert("sub", &crate::modifier::sub);
        let vars = ValueManager::try_from_iter(value_iter!(
            "var": Value::Number(1usize.into())
        ))
        .unwrap();
        let tpl_provider = DefaultTemplateProvider::default();
        let mut ctx = RenderContext::new(&modifiers, vars, &tpl_provider);
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
                            args: vec![StorageMethod::Const(Value::Number(1usize.into()))],
                            span: Default::default(),
                        }],
                    ),
                )),
            ],
        );

        let mut modifiers = ModifierContainer::default();
        modifiers.insert("sub", &crate::modifier::sub);
        let vars = ValueManager::try_from_iter(value_iter!(
            "var": Value::Number(5usize.into())
        ))
        .unwrap();
        let tpl_provider = DefaultTemplateProvider::default();
        let mut ctx = RenderContext::new(&modifiers, vars, &tpl_provider);
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

        let modifiers = ModifierContainer::default();
        let vars = ValueManager::try_from_iter(value_iter![
            "var": Value::Number(5usize.into())
        ])
        .unwrap();
        let tpl_provider = DefaultTemplateProvider::default();
        let mut ctx = RenderContext::new(&modifiers, vars, &tpl_provider);
        let mut buffer = String::new();
        assert!(l.render(&mut ctx, &mut buffer).is_ok());
        assert!(buffer.is_empty())
    }
}
