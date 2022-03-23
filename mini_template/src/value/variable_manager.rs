use crate::error::Error;
use crate::value::StorageMethod;
use std::collections::HashMap;

use super::{ident::*, Value};

pub trait VariableManager {
    fn get(&self, k: &Ident) -> crate::error::Result<&Value>;
    fn get_mut(&mut self, k: &Ident) -> crate::error::Result<&mut Value>;
    fn set(&mut self, k: &Ident, v: Value) -> crate::error::Result<()>;
    fn resolve_ident(&self, k: &Ident) -> crate::error::Result<ResolvedIdent> {
        let part = match &*k.part {
            IdentPart::Static(ident) => ResolvedIdentPart::Static(*ident),
            IdentPart::Dynamic(StorageMethod::Const(v)) => ResolvedIdentPart::Dynamic(v.clone()),
            IdentPart::Dynamic(StorageMethod::Variable(ident)) => {
                let value = self.get(&ident)?;
                ResolvedIdentPart::Dynamic(value.clone())
            }
        };

        let part = Box::new(part);

        let next = if let Some(next) = &k.next {
            Some(Box::new(self.resolve_ident(next)?))
        } else {
            None
        };

        Ok(ResolvedIdent {
            part,
            next,
            span: k.span.clone(),
        })
    }
}

impl VariableManager for HashMap<String, Value> {
    fn get(&self, k: &Ident) -> crate::error::Result<&Value> {
        let ident = self.resolve_ident(k)?;

        let k = match &*ident.part {
            ResolvedIdentPart::Static(s) => unsafe { s.as_ref().unwrap() },
            ResolvedIdentPart::Dynamic(d) => match d.try_into() {
                Ok(s) => s,
                Err(_) => return Err(Error::UnsupportedIdentifier),
            },
        };

        match self.get(k) {
            Some(v) => Ok(v),
            None => Err(Error::UnknownVariable(ident)),
        }
    }

    fn get_mut(&mut self, k: &Ident) -> crate::error::Result<&mut Value> {
        let ident = self.resolve_ident(k)?;

        let k = match &*ident.part {
            ResolvedIdentPart::Static(s) => unsafe { s.as_ref().unwrap() },
            ResolvedIdentPart::Dynamic(d) => match d.try_into() {
                Ok(s) => s,
                Err(_) => return Err(Error::UnsupportedIdentifier),
            },
        };

        match self.get_mut(k) {
            Some(v) => Ok(v),
            None => Err(Error::UnknownVariable(ident)),
        }
    }

    fn set(&mut self, k: &Ident, v: Value) -> crate::error::Result<()> {
        let ident = self.resolve_ident(k)?;

        let k = match &*ident.part {
            ResolvedIdentPart::Static(s) => unsafe { s.as_ref().unwrap() },
            ResolvedIdentPart::Dynamic(d) => match d.try_into() {
                Ok(s) => s,
                Err(_) => return Err(Error::UnsupportedIdentifier),
            },
        }
        .to_owned();

        self.insert(k, v);

        Ok(())
    }
}
