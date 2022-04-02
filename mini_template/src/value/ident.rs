use super::StorageMethod;
use crate::template::Span;
use crate::value::Value;
use crate::ValueManager;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug)]
pub struct Ident {
    pub next: Option<Box<Ident>>,
    pub part: Box<IdentPart>,
    pub span: Span,
}

impl Ident {
    pub fn resolve_ident(
        &self,
        value_manager: &ValueManager,
    ) -> crate::error::Result<ResolvedIdent> {
        let part = match &*self.part {
            IdentPart::Static(ident) => ResolvedIdentPart::Static(*ident),
            IdentPart::Dynamic(StorageMethod::Const(v)) => ResolvedIdentPart::Dynamic(v.clone()),
            IdentPart::Dynamic(StorageMethod::Variable(ident)) => {
                let value = value_manager.get_value(ident.resolve_ident(value_manager)?)?;
                ResolvedIdentPart::Dynamic(value.clone())
            }
        };

        let part = Box::new(part);

        let next = if let Some(next) = &self.next {
            Some(Box::new(next.resolve_ident(value_manager)?))
        } else {
            None
        };

        Ok(ResolvedIdent {
            part,
            next,
            span: self.span.clone(),
        })
    }

    pub fn new(part: IdentPart) -> Self {
        Self {
            next: None,
            part: Box::new(part),
            span: Span::default(),
        }
    }

    pub fn new_with_span(part: IdentPart, span: Span) -> Self {
        Self {
            next: None,
            part: Box::new(part),
            span,
        }
    }

    pub fn chain(&mut self, next: Ident) -> &mut Self {
        self.next = Some(Box::new(next));
        self.next.as_mut().unwrap()
    }
}

#[cfg(test)]
impl Ident {
    pub fn new_static(ident: &str) -> Self {
        Self {
            next: None,
            part: Box::new(IdentPart::Static(ident)),
            span: Default::default(),
        }
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.part == other.part
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

#[derive(Debug, Clone)]
pub struct ResolvedIdent {
    pub next: Option<Box<ResolvedIdent>>,
    pub part: Box<ResolvedIdentPart>,
    pub span: Span,
}

impl ResolvedIdent {
    pub fn new(part: ResolvedIdentPart) -> Self {
        Self {
            next: None,
            part: Box::new(part),
            span: Span::default(),
        }
    }

    pub fn chain(&mut self, next: ResolvedIdent) -> &mut Self {
        self.next = Some(Box::new(next));
        self.next.as_mut().unwrap()
    }
}

impl From<*const str> for ResolvedIdent {
    fn from(static_path: *const str) -> Self {
        Self::new(static_path.into())
    }
}

impl<'a> From<&'a str> for ResolvedIdent {
    fn from(static_path: &'a str) -> Self {
        Self::new((static_path as *const str).into())
    }
}

impl From<Value> for ResolvedIdent {
    fn from(dynamic: Value) -> Self {
        Self::new(dynamic.into())
    }
}

impl PartialEq for ResolvedIdent {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.part == other.part
    }
}

impl Display for ResolvedIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self.part {
            ResolvedIdentPart::Dynamic(d) => write!(f, "[{}]", d.to_string())?,
            ResolvedIdentPart::Static(ident) => {
                let ident = unsafe { ident.as_ref().unwrap() };
                f.write_str(ident)?;
                if let Some(ident) = self.next.as_ref() {
                    if let ResolvedIdentPart::Static(_) = *ident.part {
                        f.write_char('.')?
                    }
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

#[derive(Debug, Clone)]
pub enum ResolvedIdentPart {
    Static(*const str),
    Dynamic(Value),
}

impl From<*const str> for ResolvedIdentPart {
    fn from(static_path: *const str) -> Self {
        Self::Static(static_path)
    }
}

impl From<Value> for ResolvedIdentPart {
    fn from(dynamic: Value) -> Self {
        Self::Dynamic(dynamic)
    }
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
