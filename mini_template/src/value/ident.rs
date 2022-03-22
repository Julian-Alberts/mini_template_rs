use super::StorageMethod;
use crate::value::Value;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug, PartialEq)]
pub struct Ident {
    pub next: Option<Box<Ident>>,
    pub part: Box<IdentPart>,
}

impl Ident {
    pub fn new_static(ident: &str) -> Self {
        Self {
            next: None,
            part: Box::new(IdentPart::Static(ident)),
        }
    }
}

#[derive(Debug)]
pub enum IdentPart {
    Static(*const str),
    Dynamic(StorageMethod),
}

impl PartialEq for IdentPart {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IdentPart::Static(s), IdentPart::Static(o)) => unsafe {
                s.as_ref().unwrap() == o.as_ref().unwrap()
            },
            (IdentPart::Dynamic(s), IdentPart::Dynamic(o)) => s == o,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ResolvedIdent {
    pub next: Option<Box<ResolvedIdent>>,
    pub part: Box<ResolvedIdentPart>,
}

impl Display for ResolvedIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self.part {
            ResolvedIdentPart::Dynamic(d) => write!(f, "[{}]", d.to_string())?,
            ResolvedIdentPart::Static(ident) => {
                let ident = unsafe { ident.as_ref().unwrap() };
                f.write_str(ident)?;
                match self.next.as_ref() {
                    Some(ident) => {
                        if let ResolvedIdentPart::Static(_) = *ident.part {
                            f.write_char('.')?
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(ident) = &self.next {
            let ident = ident.as_ref();
            Display::fmt(ident, f)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum ResolvedIdentPart {
    Static(*const str),
    Dynamic(Value),
}

impl PartialEq for ResolvedIdentPart {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ResolvedIdentPart::Dynamic(s), ResolvedIdentPart::Dynamic(o)) => s == o,
            (ResolvedIdentPart::Static(s), ResolvedIdentPart::Static(o)) => unsafe {
                s.as_ref() == o.as_ref()
            },
            _ => false,
        }
    }
}

impl Display for ResolvedIdentPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedIdentPart::Static(s) => unsafe { f.write_str(s.as_ref().unwrap()) },
            ResolvedIdentPart::Dynamic(d) => f.write_str(&d.to_string()),
        }
    }
}
