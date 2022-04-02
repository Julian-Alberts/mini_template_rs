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
        let k = get_ident_key(ident)?;

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
        let k = get_ident_key(ident)?;

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
        let k = get_ident_key(ident)?;

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

fn get_ident_key(ident: &ResolvedIdent) -> crate::error::Result<&str> {
    match &*ident.part {
        ResolvedIdentPart::Static(s) => unsafe { Ok(s.as_ref().unwrap()) },
        ResolvedIdentPart::Dynamic(d) => match d.try_into() {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::UnsupportedIdentifier),
        },
    }
}
