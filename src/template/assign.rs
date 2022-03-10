use crate::{renderer::RenderContext, variable_container::VariableContainer};

use super::CalculatedValue;

#[derive(Debug)]
pub struct Assign {
    identifier: *const str,
    calc: CalculatedValue,
}

impl Assign {
    pub fn new(identifier: *const str, calc: CalculatedValue) -> Self {
        Self { identifier, calc }
    }

    pub fn assign<VC: VariableContainer>(
        &self,
        context: &mut RenderContext<VC>,
    ) -> crate::error::Result<()> {
        let v = self.calc.calc(context)?;
        // Safety: identifier points to the original template string
        let k = unsafe { self.identifier.as_ref().unwrap() };
        context.variables.set(k.to_owned(), v);
        Ok(())
    }
}

impl PartialEq for Assign {

    fn eq(&self, other: &Self) -> bool {
        // Safety: identifier points to the original template string
        let ident_eq = unsafe { self.identifier.as_ref() == other.identifier.as_ref() };
        ident_eq && self.calc == other.calc
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        renderer::RenderContext,
        template::{CalculatedValue, StorageMethod},
        value::Value,
        variable_container::VariableContainer,
    };

    use super::Assign;

    #[test]
    fn simple_assign() {
        let mut vars = HashMap::default();
        vars.set(String::from("input"), Value::Number(42.));
        let modifiers = HashMap::default();
        let mut rc = RenderContext::new(&modifiers, vars);

        let assign = Assign::new(
            "output",
            CalculatedValue::new(StorageMethod::Variable("input"), vec![]),
        );
        assert!(assign.assign(&mut rc).is_ok());
        assert_eq!(rc.variables.get("output"), Some(&Value::Number(42.)))
    }

    #[test]
    fn assign_calculated() {
        let mut vars = HashMap::default();
        vars.set(String::from("input"), Value::Number(42.));
        let mut modifiers = HashMap::new();
        let add_modifier: &crate::modifier::Modifier = &crate::modifier::add;
        modifiers.insert("add", add_modifier);
        let mut rc = RenderContext::new(&modifiers, vars);

        let assign = Assign::new(
            "output",
            CalculatedValue::new(StorageMethod::Variable("input"), vec![
                ("add", vec![
                    StorageMethod::Const(Value::Number(2.))
                ])
            ]),
        );
        assert!(assign.assign(&mut rc).is_ok());
        assert_eq!(rc.variables.get("output"), Some(&Value::Number(44.)))
    }
}
