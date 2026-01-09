use crate::error::Error;
use std::collections::{btree_map::Entry, BTreeMap};

use super::{ident::*, Value};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ValueManager {
    values: BTreeMap<String, Value>,
}

impl ValueManager {
    pub fn get_value(&self, ident: ResolvedIdent) -> crate::error::Result<&Value> {
        self.get_value_recursive(&ident)
    }

    fn get_value_recursive(&self, ident: &ResolvedIdent) -> crate::error::Result<&Value> {
        let mut ident_parts = ident.parts();
        let mut vm = self;
        if ident_parts.is_empty() {
            return Err(Error::UnknownVariable(ident.clone()));
        }
        loop {
            let part = &ident_parts[0];
            ident_parts = &ident_parts[1..];

            let k = get_ident_key(part)?;
            let value = match vm.values.get(&k) {
                Some(v) => v,
                None => return Err(Error::UnknownVariable(ident.clone())),
            };

            if ident_parts.is_empty() {
                return Ok(value);
            }

            match value {
                Value::Object(next_vm) => {
                    vm = next_vm;
                }
                _ => return Err(Error::UnknownVariable(ident.clone())),
            }
        }
    }

    pub fn get_value_mut(&mut self, ident: ResolvedIdent) -> crate::error::Result<&mut Value> {
        self.get_value_mut_recursive(&ident, &ident)
    }

    fn get_value_mut_recursive(
        &mut self,
        ident: &ResolvedIdent,
        _full_ident: &ResolvedIdent,
    ) -> crate::error::Result<&mut Value> {
        self.get_value_mut_inner(ident)
    }

    fn get_value_mut_inner(&mut self, ident: &ResolvedIdent) -> crate::error::Result<&mut Value> {
        let mut ident_parts = ident.parts();
        let mut vm = self;
        if ident_parts.is_empty() {
            return Err(Error::UnknownVariable(ident.clone()));
        }
        loop {
            let part = &ident_parts[0];
            let k = &get_ident_key(part)?;
            let value = match vm.values.get_mut(k) {
                Some(v) => v,
                None => return Err(Error::UnknownVariable(ident.clone())),
            };

            ident_parts = &ident_parts[1..];

            if ident_parts.is_empty() {
                return Ok(value);
            }

            match value {
                Value::Object(next_vm) => {
                    vm = next_vm;
                }
                _ => return Err(Error::UnknownVariable(ident.clone())),
            }
        }
    }

    pub fn set_value(&mut self, ident: ResolvedIdent, value: Value) -> crate::error::Result<()> {
        self.set_value_inner(ident.parts(), &ident, value)
    }

    fn set_value_inner(
        &mut self,
        ident_parts: &[ResolvedIdentPart],
        ident: &ResolvedIdent,
        value: Value,
    ) -> crate::error::Result<()> {
        if ident_parts.is_empty() {
            return Err(Error::UnknownVariable(ident.clone()));
        }
        let k = get_ident_key(&ident_parts[0])?;
        match ident_parts.len() {
            1 => {
                self.values.insert(k, value);
                Ok(())
            }
            _ => match self.values.entry(k) {
                Entry::Occupied(mut v) => {
                    if let Value::Object(vm) = v.get_mut() {
                        vm.set_value_inner(&ident_parts[1..], ident, value)
                    } else {
                        Err(Error::UnknownProperty(ident.clone()))
                    }
                }
                Entry::Vacant(entry) => {
                    let mut new_vm = ValueManager::default();
                    new_vm.set_value_inner(&ident_parts[1..], ident, value)?;
                    entry.insert(Value::Object(new_vm));
                    Ok(())
                }
            },
        }
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

fn get_ident_key(ident: &ResolvedIdentPart) -> crate::error::Result<String> {
    match &ident.part {
        ResolvedIdentPartType::Static(s) => Ok(s.get_string().to_owned()),
        ResolvedIdentPartType::Dynamic(Value::Number(super::Number::ISize(n))) => {
            Ok((*n as usize).to_string())
        }
        ResolvedIdentPartType::Dynamic(Value::Number(super::Number::USize(n))) => {
            Ok((n).to_string())
        }
        ResolvedIdentPartType::Dynamic(Value::Number(super::Number::F32(n))) => {
            Ok((*n as usize).to_string())
        }
        ResolvedIdentPartType::Dynamic(Value::Number(super::Number::F64(n))) => {
            Ok((*n as usize).to_string())
        }
        ResolvedIdentPartType::Dynamic(d) => match d.try_into() {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::UnsupportedIdentifier),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::util::TemplateString;
    use crate::value::ident::{Ident, ResolvedIdent, ResolvedIdentPart, ResolvedIdentPartType};
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
    fn mut_access() {
        let mut vm = ValueManager::try_from_iter(value_iter![
            "yay": Value::Bool(true)
        ])
        .unwrap();
        assert_eq!(
            vm.get_value_mut(Value::String("yay".to_string()).into()),
            Ok(&mut Value::Bool(true))
        )
    }

    #[test]
    fn resolve_static_ident() {
        let ident = Ident::try_from("obj.val")
            .unwrap()
            .resolve_ident(&ValueManager::default());
        assert_eq!(
            ident,
            Ok(ResolvedIdent::new(vec![
                ResolvedIdentPart {
                    part: ResolvedIdentPartType::Static(TemplateString::Ref("obj")),
                    span: Default::default(),
                },
                ResolvedIdentPart {
                    part: ResolvedIdentPartType::Static(TemplateString::Ref("val")),
                    span: Default::default(),
                },
            ]))
        );
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
    fn static_object_access_outer() {
        let vm = ValueManager::try_from_iter(value_iter![
            "obj.val": Value::Bool(true)
        ])
        .unwrap();

        let ident: ResolvedIdent = "obj".into();

        let mut inner = ValueManager::default();
        inner.set_value("val".into(), Value::Bool(true)).unwrap();
        assert_eq!(vm.get_value(ident), Ok(&Value::Object(inner)))
    }

    #[test]
    fn try_set_nested_value_of_bool() {
        let mut vm = ValueManager::try_from_iter(value_iter![
            "foo": Value::Bool(false)
        ])
        .unwrap();
        let mut ident: ResolvedIdent = "foo".into();
        ident.chain("bar".into());
        assert_eq!(
            vm.set_value(ident.clone(), Value::Number(1usize.into())),
            Err(super::Error::UnknownProperty(ident))
        )
    }

    #[test]
    fn access_of_unknown_value() {
        let vm = ValueManager::try_from_iter(value_iter![
            "foo": Value::Bool(false)
        ])
        .unwrap();
        let mut ident: ResolvedIdent = "foo".into();
        ident.chain("bar".into());
        assert_eq!(
            vm.get_value(ident.clone()),
            Err(super::Error::UnknownVariable(ident))
        )
    }

    #[test]
    fn mut_access_of_unknown_value() {
        let mut vm = ValueManager::try_from_iter(value_iter![
            "foo": Value::Bool(false)
        ])
        .unwrap();
        let ident: ResolvedIdent = "bar".into();
        assert_eq!(
            vm.get_value_mut(ident.clone()),
            Err(super::Error::UnknownVariable(ident))
        )
    }

    #[test]
    fn access_of_nested_value() {
        let mut vm = ValueManager::try_from_iter(value_iter![
            "foo.foobar": Value::Bool(false)
        ])
        .unwrap();
        let mut ident: ResolvedIdent = "foo".into();
        ident.chain("foobar".into());
        assert_eq!(vm.get_value_mut(ident.clone()), Ok(&mut Value::Bool(false)))
    }

    #[test]
    fn access_of_unknown_nested_value() {
        let vm = ValueManager::try_from_iter(value_iter![
            "foo": Value::Bool(false)
        ])
        .unwrap();
        let mut ident: ResolvedIdent = "foo".into();
        ident.chain("foobar".into());
        assert_eq!(
            vm.get_value(ident.clone()),
            Err(super::Error::UnknownVariable(ident))
        )
    }

    #[test]
    fn mut_access_of_unknown_nested_value() {
        let mut vm = ValueManager::try_from_iter(value_iter![
            "foo": Value::Bool(false)
        ])
        .unwrap();
        let mut ident: ResolvedIdent = "foo".into();
        ident.chain("foobar".into());
        assert_eq!(
            vm.get_value_mut(ident.clone()),
            Err(super::Error::UnknownVariable(ident))
        )
    }

    #[test]
    fn dynamic_object_access() {
        let vm = ValueManager::try_from_iter(value_iter![
            "obj": Value::Object(Default::default()),
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
            "val": Value::Number(1usize.into()),
            // I don't know why any body should ever do this,
            // but it is supported by the ident parser so why not.
            "obj[val]": Value::Bool(true),
            "obj[\"foo\"]": Value::Number(33usize.into())
        ])
        .unwrap();

        let mut ident_num: ResolvedIdent = "obj".into();
        ident_num.chain("foo".into());
        let ident_bool = Ident::try_from("obj[val]")
            .unwrap()
            .resolve_ident(&vm)
            .unwrap();

        assert_eq!(vm.get_value(ident_num), Ok(&Value::Number(33usize.into())));
        assert_eq!(vm.get_value(ident_bool), Ok(&Value::Bool(true)))
    }

    #[test]
    fn get_length_of_value_manager() {
        let vm = ValueManager::try_from_iter(value_iter![
            "val": Value::String("hi".to_owned()),
            "obj[\"bar\"]": Value::Bool(true),
            "obj[\"foo\"]": Value::Number(33usize.into())
        ])
        .unwrap();
        assert_eq!(vm.len(), 2);
    }
}
