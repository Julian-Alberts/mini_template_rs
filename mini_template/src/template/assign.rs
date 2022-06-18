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

    use serde_json::json;

    use crate::template::Modifier;
    use crate::value::ident::Ident;
    use crate::{renderer::RenderContext, template::CalculatedValue, value::StorageMethod};

    use super::Assign;

    #[test]
    fn simple_assign() {
        let vars = json!({
            "input": 42_f64
        })
        .try_into()
        .unwrap();
        let modifiers = HashMap::default();
        let templates = HashMap::<String, _>::new();
        let mut rc = RenderContext::new(&modifiers, vars, &templates);

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
            Ok(&json!(42_f64))
        )
    }

    #[test]
    fn assign_calculated() {
        let vars = json!({
            "input": 42_f64
        })
        .try_into()
        .unwrap();

        let mut modifiers = HashMap::new();
        let add_modifier: &crate::modifier::Modifier = &crate::modifier::add;
        modifiers.insert("add", add_modifier);
        let templates = HashMap::<String, _>::new();
        let mut rc = RenderContext::new(&modifiers, vars, &templates);

        let assign = Assign::new(
            Ident::new_static("output"),
            CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("input")),
                vec![Modifier {
                    name: "add",
                    args: vec![StorageMethod::Const(json!(2_f64))],
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
            Ok(&json!(44_f64))
        )
    }
}
