use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::prelude::*;

use super::ident::ResolvedIdent;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ValueManager {
    values: Map<String, Value>,
}

impl ValueManager {
    pub fn get_value(&self, ident: ResolvedIdent) -> crate::error::Result<&Value> {
        IdentifiableValue::get(&self.values, &ident)
    }

    pub fn get_value_mut<'a>(
        &'a mut self,
        ident: ResolvedIdent,
    ) -> crate::error::Result<&'a mut Value> {
        IdentifiableValue::get_mut(&mut self.values, &ident)
    }

    pub fn set_value(&mut self, ident: ResolvedIdent, value: Value) -> crate::error::Result<()> {
        self.values.set(&ident, value)
    }

    pub fn from_serde<T>(value: T) -> Result<Self, ()>
        where T: Serialize
    {
        let value = json!(value);
        Self::try_from(value)
    }

}

impl TryFrom<Value> for ValueManager {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Object(values) = value {
            Ok(ValueManager { values })
        } else {
            Err(())
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Value;
    use crate::value::ident::ResolvedIdent;
    use crate::ValueManager;

    #[test]
    fn simple_static_access() {
        let vm: ValueManager = json!({
            "yay": true
        })
        .try_into()
        .unwrap();
        assert_eq!(vm.get_value("yay".into()), Ok(&Value::Bool(true)))
    }

    #[test]
    fn simple_dynamic_access() {
        let vm: ValueManager = json!({
            "yay": true
        })
        .try_into()
        .unwrap();
        assert_eq!(
            vm.get_value(Value::String("yay".to_string()).into()),
            Ok(&Value::Bool(true))
        )
    }

    #[test]
    fn static_object_access() {
        let vm: ValueManager = json!({
            "obj": {
                "val": true
            }
        })
        .try_into()
        .unwrap();

        let mut ident: ResolvedIdent = "obj".into();
        ident.chain("val".into());

        assert_eq!(vm.get_value(ident), Ok(&Value::Bool(true)))
    }

    #[test]
    fn dynamic_object_access() {
        let vm: ValueManager = json!({
            "obj": {
                "val": true
            }
        })
        .try_into()
        .unwrap();

        let mut ident: ResolvedIdent = "obj".into();
        ident.chain("val".into());

        assert_eq!(vm.get_value(ident), Ok(&Value::Bool(true)))
    }

    #[test]
    fn access_trough_ident() {
        let vm: ValueManager = json!({
            "obj": {
                "hi": true,
                "foo": 33_f64
            }
        })
        .try_into()
        .unwrap();

        let mut ident_42: ResolvedIdent = "obj".into();
        let mut ident_32 = ident_42.clone();
        ident_42.chain("hi".into());
        ident_32.chain("foo".into());

        assert_eq!(vm.get_value(ident_32), Ok(&json! {33_f64}));
        assert_eq!(vm.get_value(ident_42), Ok(&Value::Bool(true)))
    }
}
