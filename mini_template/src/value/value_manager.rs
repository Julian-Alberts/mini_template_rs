use crate::error::Error;
use std::collections::BTreeMap;

use super::{ident::*, Value};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ValueManager {
    values: BTreeMap<String, Value>,
}

impl ValueManager {
    pub fn get_value(&self, ident: ResolvedIdent) -> crate::error::Result<&Value> {
        self.get_value_recursive(&ident, &ident)
    }

    fn get_value_recursive(
        &self,
        ident: &ResolvedIdent,
        full_ident: &ResolvedIdent,
    ) -> crate::error::Result<&Value> {
        let k = &get_ident_key(ident)?;

        let value = match self.values.get(k) {
            Some(v) => v,
            None => return Err(Error::UnknownVariable(full_ident.clone())),
        };

        if let Some(ident) = &ident.next {
            match value {
                Value::Object(vm) => vm.get_value_recursive(ident, full_ident),
                _ => Err(Error::UnknownVariable(full_ident.clone())),
            }
        } else {
            Ok(value)
        }
    }

    pub fn get_value_mut(&mut self, ident: ResolvedIdent) -> crate::error::Result<&mut Value> {
        self.get_value_mut_recursive(&ident, &ident)
    }

    fn get_value_mut_recursive(
        &mut self,
        ident: &ResolvedIdent,
        full_ident: &ResolvedIdent,
    ) -> crate::error::Result<&mut Value> {
        let k = &get_ident_key(ident)?;

        let value = match self.values.get_mut(k) {
            Some(v) => v,
            None => return Err(Error::UnknownVariable(full_ident.clone())),
        };

        if let Some(ident) = &ident.next {
            match value {
                Value::Object(vm) => vm.get_value_mut_recursive(ident, full_ident),
                _ => Err(Error::UnknownVariable(full_ident.clone())),
            }
        } else {
            Ok(value)
        }
    }

    pub fn set_value(&mut self, ident: ResolvedIdent, value: Value) -> crate::error::Result<()> {
        self.set_value_recursive(&ident, value, &ident)
    }

    fn set_value_recursive(
        &mut self,
        ident: &ResolvedIdent,
        value: Value,
        full_ident: &ResolvedIdent,
    ) -> crate::error::Result<()> {
        let k = &get_ident_key(ident)?;

        use None as EndOfPath;
        use None as EmptyValue;
        use Some as NextPath;
        use Some as FoundValue;

        match (self.values.get_mut(k), &ident.next) {
            (_, EndOfPath) => {
                self.values.insert(k.to_owned(), value);
            }
            (FoundValue(Value::Object(vm)), NextPath(next)) => {
                vm.set_value_recursive(next, value, full_ident)?;
            }
            (FoundValue(_), NextPath(_)) => {
                return Err(crate::error::Error::UnknownProperty(full_ident.clone()))
            }
            (EmptyValue, NextPath(next)) => {
                let mut vm = Self::default();
                vm.set_value_recursive(next, value, full_ident)?;
                self.values.insert(k.to_owned(), Value::Object(vm));
            }
        };

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl ValueManager {
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, Value)>>(
        iter: T,
    ) -> crate::error::Result<Self> {
        let mut me = ValueManager::default();
        for (i, v) in iter {
            me.set_value(i.resolve_ident(&me)?, v)?
        }
        Ok(me)
    }
}

fn get_ident_key(ident: &ResolvedIdent) -> crate::error::Result<String> {
    match &*ident.part {
        ResolvedIdentPart::Static(s) => Ok(s.get_string().to_owned()),
        ResolvedIdentPart::Dynamic(Value::Number(n)) => Ok((*n as usize).to_string()),
        ResolvedIdentPart::Dynamic(d) => match d.try_into() {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::UnsupportedIdentifier),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::value::ident::ResolvedIdent;
    use crate::{value_iter, Value, ValueManager};

    #[test]
    fn simple_static_access() {
        let vm = ValueManager::try_from_iter(value_iter![
            "yay": Value::Bool(true)
        ])
        .unwrap();
        assert_eq!(vm.get_value("yay".into()), Ok(&Value::Bool(true)))
    }

    #[test]
    fn simple_dynamic_access() {
        let vm = ValueManager::try_from_iter(value_iter![
            "yay": Value::Bool(true)
        ])
        .unwrap();
        assert_eq!(
            vm.get_value(Value::String("yay".to_string()).into()),
            Ok(&Value::Bool(true))
        )
    }

    #[test]
    fn static_object_access() {
        let vm = ValueManager::try_from_iter(value_iter![
            "obj.val": Value::Bool(true)
        ])
        .unwrap();

        let mut ident: ResolvedIdent = "obj".into();
        ident.chain("val".into());

        assert_eq!(vm.get_value(ident), Ok(&Value::Bool(true)))
    }

    #[test]
    fn dynamic_object_access() {
        let vm = ValueManager::try_from_iter(value_iter![
            "obj.val": Value::Bool(true)
        ])
        .unwrap();

        let mut ident: ResolvedIdent = "obj".into();
        ident.chain("val".into());

        assert_eq!(vm.get_value(ident), Ok(&Value::Bool(true)))
    }

    #[test]
    fn access_trough_ident() {
        let vm = ValueManager::try_from_iter(value_iter![
            "val": Value::String("hi".to_owned()),
            // I don't know why any body should ever do this,
            // but it is supported by the ident parser so why not.
            "obj[val]": Value::Bool(true),
            "obj[\"foo\"]": Value::Number(33.)
        ])
        .unwrap();

        let mut ident_42: ResolvedIdent = "obj".into();
        let mut ident_32 = ident_42.clone();
        ident_42.chain("hi".into());
        ident_32.chain("foo".into());

        assert_eq!(vm.get_value(ident_32), Ok(&Value::Number(33.)));
        assert_eq!(vm.get_value(ident_42), Ok(&Value::Bool(true)))
    }
}
