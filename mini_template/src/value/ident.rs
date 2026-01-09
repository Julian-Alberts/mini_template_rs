use super::StorageMethod;
use crate::value::Value;
use crate::ValueManager;
use crate::{template::Span, util::TemplateString};
use std::fmt::{Debug, Display, Formatter, Write};

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
        let mut parts = Vec::new();
        let mut current = self;
        loop {
            let part = match current.part.as_ref() {
                IdentPart::Static(ident) => ResolvedIdentPartType::Static(ident.clone()),
                IdentPart::Dynamic(StorageMethod::Const(v)) => {
                    ResolvedIdentPartType::Dynamic(v.clone())
                }
                IdentPart::Dynamic(StorageMethod::Variable(ident)) => {
                    let value = value_manager.get_value(ident.resolve_ident(value_manager)?)?;
                    ResolvedIdentPartType::Dynamic(value.clone())
                }
            };
            parts.push(ResolvedIdentPart {
                part,
                span: current.span.clone(),
            });
            if let Some(next) = &current.next {
                current = next;
            } else {
                break;
            }
        }

        Ok(ResolvedIdent::new(parts))
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
    pub fn new_static(ident: &'static str) -> Self {
        Self {
            next: None,
            part: Box::new(IdentPart::Static(TemplateString::Ptr(ident))),
            span: Default::default(),
        }
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.part == other.part
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self.part {
            IdentPart::Dynamic(ident) => write!(f, "[{:?}]", ident)?,
            IdentPart::Static(ident) => {
                write!(f, "{}", ident.get_string())?;
                if self.next.is_some() {
                    write!(f, ".")?;
                }
            }
        }
        if let Some(next) = &self.next {
            write!(f, "{:?}", next)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum IdentPart {
    Static(TemplateString),
    Dynamic(StorageMethod),
}

impl PartialEq for IdentPart {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IdentPart::Static(s), IdentPart::Static(o)) => s == o,
            (IdentPart::Dynamic(s), IdentPart::Dynamic(o)) => s == o,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ResolvedIdent {
    parts: Vec<ResolvedIdentPart>,
}

impl ResolvedIdent {
    pub fn new(parts: Vec<ResolvedIdentPart>) -> Self {
        Self { parts }
    }

    pub fn chain(&mut self, next: ResolvedIdent) -> &mut Self {
        self.parts.extend(next.parts);
        self
    }

    pub fn push(&mut self, part: ResolvedIdentPartType, span: Span) {
        self.parts.push(ResolvedIdentPart { part, span });
    }

    pub fn parts(&self) -> &[ResolvedIdentPart] {
        &self.parts
    }
}

impl From<String> for ResolvedIdent {
    fn from(static_path: String) -> Self {
        Self::new(vec![ResolvedIdentPart {
            part: ResolvedIdentPartType::Static(TemplateString::Owned(static_path)),
            span: Span::default(),
        }])
    }
}

impl From<&'static str> for ResolvedIdent {
    fn from(static_path: &'static str) -> Self {
        Self::new(vec![ResolvedIdentPart {
            part: ResolvedIdentPartType::Static(TemplateString::Ptr(static_path)),
            span: Span::default(),
        }])
    }
}

impl From<Value> for ResolvedIdent {
    fn from(dynamic: Value) -> Self {
        Self::new(vec![ResolvedIdentPart {
            part: ResolvedIdentPartType::Dynamic(dynamic),
            span: Span::default(),
        }])
    }
}

impl Display for ResolvedIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            match &part.part {
                ResolvedIdentPartType::Dynamic(d) => write!(f, "[{}]", d.to_string())?,
                ResolvedIdentPartType::Static(ident) => {
                    let ident = ident.get_string();
                    f.write_str(ident)?;
                    if part != self.parts.last().unwrap() {
                        f.write_char('.')?
                    }
                }
            }
        }
        Ok(())
    }
}

impl Debug for ResolvedIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            match &part.part {
                ResolvedIdentPartType::Dynamic(d) => write!(f, "[{}]", d.to_string())?,
                ResolvedIdentPartType::Static(ident) => {
                    let ident = ident.get_string();
                    f.write_str(ident)?;
                    if part != self.parts.last().unwrap() {
                        f.write_char('.')?
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedIdentPart {
    pub part: ResolvedIdentPartType,
    pub(crate) span: Span,
}

impl ResolvedIdentPart {
    pub fn part(&self) -> &ResolvedIdentPartType {
        &self.part
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

impl PartialEq for ResolvedIdentPart {
    fn eq(&self, other: &Self) -> bool {
        self.part == other.part
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedIdentPartType {
    Static(TemplateString),
    Dynamic(Value),
}

impl From<&'static str> for ResolvedIdentPartType {
    fn from(static_path: &'static str) -> Self {
        Self::Static(TemplateString::Ptr(static_path))
    }
}

impl From<String> for ResolvedIdentPartType {
    fn from(path: String) -> Self {
        Self::Static(TemplateString::Owned(path))
    }
}

impl From<Value> for ResolvedIdentPartType {
    fn from(dynamic: Value) -> Self {
        Self::Dynamic(dynamic)
    }
}

impl PartialEq for ResolvedIdentPartType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ResolvedIdentPartType::Dynamic(s), ResolvedIdentPartType::Dynamic(o)) => s == o,
            (ResolvedIdentPartType::Static(s), ResolvedIdentPartType::Static(o)) => s == o,
            _ => false,
        }
    }
}

impl Display for ResolvedIdentPartType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedIdentPartType::Static(s) => f.write_str(s.get_string()),
            ResolvedIdentPartType::Dynamic(d) => f.write_str(&d.to_string()),
        }
    }
}
