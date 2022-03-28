use crate::{
    renderer::RenderContext,
    value::{ident::Ident, VariableManager},
    TemplateKey,
};

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

    pub fn assign<VM: VariableManager, TK>(
        &self,
        context: &mut RenderContext<VM, TK>,
    ) -> crate::error::Result<()>
    where
        TK: TemplateKey,
    {
        let v = self.calc.calc(context)?;
        context.variables.set(&self.identifier, v)
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
    use crate::value::ident::Ident;
    use crate::{
        renderer::RenderContext,
        template::CalculatedValue,
        value::{StorageMethod, Value},
        VariableManager,
    };

    use super::Assign;

    #[test]
    fn simple_assign() {
        let mut vars = HashMap::default();
        vars.set(&Ident::new_static("input"), Value::Number(42.))
            .unwrap();
        let modifiers = HashMap::default();
        let templates = HashMap::new();
        let mut rc = RenderContext::<_, String>::new(&modifiers, vars, &templates);

        let assign = Assign::new(
            Ident::new_static("output"),
            CalculatedValue::new(StorageMethod::Variable(Ident::new_static("input")), vec![]),
        );
        assert!(assign.assign(&mut rc).is_ok());
        assert_eq!(rc.variables.get("output"), Some(&Value::Number(42.)))
    }

    #[test]
    fn assign_calculated() {
        let mut vars = HashMap::default();
        vars.set(&Ident::new_static("input"), Value::Number(42.))
            .unwrap();
        let mut modifiers = HashMap::new();
        let add_modifier: &crate::modifier::Modifier = &crate::modifier::add;
        modifiers.insert("add", add_modifier);
        let templates = HashMap::new();
        let mut rc = RenderContext::<_, String>::new(&modifiers, vars, &templates);

        let assign = Assign::new(
            Ident::new_static("output"),
            CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("input")),
                vec![Modifier {
                    name: "add",
                    args: vec![StorageMethod::Const(Value::Number(2.))],
                    span: Default::default(),
                }],
            ),
        );
        assert!(assign.assign(&mut rc).is_ok());
        assert_eq!(rc.variables.get("output"), Some(&Value::Number(44.)))
    }
}
