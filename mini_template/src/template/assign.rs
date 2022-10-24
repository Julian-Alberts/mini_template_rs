use crate::{renderer::RenderContext, value::ident::Ident};

use super::CalculatedValue;

#[derive(Debug)]
pub struct Assign {
    identifier: Ident,
    calc: CalculatedValue,
}

impl Assign {
    pub fn new(identifier: Ident, calc: CalculatedValue) -> Self {
        Self { identifier, calc }
    }

    pub fn assign(&self, context: &mut RenderContext) -> crate::error::Result<()> {
        let v = self.calc.calc(context)?;
        let ident = self.identifier.resolve_ident(&context.variables)?;
        context.variables.set_value(ident, v)
    }
}

impl PartialEq for Assign {
    fn eq(&self, other: &Self) -> bool {
        let ident_eq = self.identifier == other.identifier;
        ident_eq && self.calc == other.calc
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::template::Modifier;
    use crate::template_provider::DefaultTemplateProvider;
    use crate::value::ident::Ident;
    use crate::{
        renderer::RenderContext,
        template::CalculatedValue,
        value::{StorageMethod, Value},
        value_iter, ValueManager,
    };

    use super::Assign;

    #[test]
    fn simple_assign() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "input": Value::Number(42f64.into())
        ))
        .unwrap();

        let modifiers = HashMap::default();
        let tpl_provider = DefaultTemplateProvider::default();
        let mut rc = RenderContext::new(&modifiers, vars, &tpl_provider);

        let assign = Assign::new(
            Ident::new_static("output"),
            CalculatedValue::new(StorageMethod::Variable(Ident::new_static("input")), vec![]),
        );
        assert!(assign.assign(&mut rc).is_ok());
        assert_eq!(
            rc.variables.get_value(
                Ident::try_from("output")
                    .unwrap()
                    .resolve_ident(&rc.variables)
                    .unwrap()
            ),
            Ok(&Value::Number(42f64.into()))
        )
    }

    #[test]
    fn assign_calculated() {
        let vars = ValueManager::try_from_iter(value_iter!(
            "input": Value::Number(42f64.into())
        ))
        .unwrap();

        let mut modifiers = HashMap::new();
        let add_modifier: &crate::modifier::Modifier = &crate::modifier::add;
        modifiers.insert("add", add_modifier);
        let tpl_provider = DefaultTemplateProvider::default();
        let mut rc = RenderContext::new(&modifiers, vars, &tpl_provider);

        let assign = Assign::new(
            Ident::new_static("output"),
            CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("input")),
                vec![Modifier {
                    name: "add",
                    args: vec![StorageMethod::Const(Value::Number(2f64.into()))],
                    span: Default::default(),
                }],
            ),
        );
        assert!(assign.assign(&mut rc).is_ok());
        assert_eq!(
            rc.variables.get_value(
                Ident::try_from("output")
                    .unwrap()
                    .resolve_ident(&rc.variables)
                    .unwrap()
            ),
            Ok(&Value::Number(44f64.into()))
        )
    }
}
