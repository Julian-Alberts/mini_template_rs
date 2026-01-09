use super::StorageMethod;
use crate::value::Value;
use crate::ValueManager;
use crate::{template::Span, util::TemplateString};
use std::fmt::{Debug, Display, Formatter, Write};

pub struct IdentOld {
    pub next: Option<Box<Ident>>,
    pub part: Box<IdentPartType>,
    pub span: Span,
}

#[derive(Clone, PartialEq)]
pub struct Ident {
    parts: Vec<IdentPart>,
}

impl Ident {
    pub fn resolve_ident(
        &self,
        value_manager: &ValueManager,
    ) -> crate::error::Result<ResolvedIdent> {
        let parts = self
            .parts
            .iter()
            .map(|part| {
                let part_type = match &part.part {
                    IdentPartType::Static(ident) => ResolvedIdentPartType::Static(ident.clone()),
                    IdentPartType::Dynamic(StorageMethod::Const(v)) => {
                        ResolvedIdentPartType::Dynamic(v.clone())
                    }
                    IdentPartType::Dynamic(StorageMethod::Variable(ident)) => {
                        let value = value_manager.get_value(ident.resolve_ident(value_manager)?)?;
                        ResolvedIdentPartType::Dynamic(value.clone())
                    }
                };
                crate::error::Result::Ok(ResolvedIdentPart {
                    part: part_type,
                    span: part.span.clone(),
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(ResolvedIdent::new(parts))
    }

    pub fn new(part: IdentPartType) -> Self {
        Self {
            parts: vec![IdentPart {
                part,
                span: Span::default(),
            }],
        }
    }

    pub fn new_with_span(part: IdentPartType, span: Span) -> Self {
        Self {
            parts: vec![IdentPart { part, span }],
        }
    }

    pub fn new_with_parts(parts: Vec<IdentPart>) -> Self {
        Self { parts }
    }

    pub fn chain(&mut self, next: Ident) -> &mut Self {
        self.parts.extend(next.parts);
        self
    }
}

#[cfg(test)]
impl Ident {
    pub fn new_static(ident: &'static str) -> Self {
        Self {
            parts: vec![IdentPart {
                part: IdentPartType::Static(TemplateString::Ref(ident)),
                span: Span::default(),
            }],
        }
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            match &part.part {
                IdentPartType::Dynamic(ident) => write!(f, "[{:?}]", ident)?,
                IdentPartType::Static(ident) => {
                    write!(f, "{}", ident.get_string())?;
                    if part != self.parts.last().unwrap() {
                        write!(f, ".")?;
                    }
                }
            }
        }
        if f.alternate() {
            writeln!(f)?;
        } else {
            write!(f, " => ")?;
        }
        let mut debug = f.debug_struct("Ident");
        debug.field("parts", &self.parts).finish()?;
        debug.finish()
    }
}

#[derive(Debug, Clone)]
pub struct IdentPart {
    pub part: IdentPartType,
    pub(crate) span: Span,
}

impl IdentPart {
    pub(crate) fn dynamic(part: StorageMethod, span: Span) -> Self {
        Self {
            part: IdentPartType::Dynamic(part),
            span,
        }
    }

    pub(crate) fn static_part(part: TemplateString, span: Span) -> Self {
        Self {
            part: IdentPartType::Static(part),
            span,
        }
    }
}

impl PartialEq for IdentPart {
    fn eq(&self, other: &Self) -> bool {
        self.part == other.part
    }
}

#[derive(Debug, Clone)]
pub enum IdentPartType {
    Static(TemplateString),
    Dynamic(StorageMethod),
}

impl PartialEq for IdentPartType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IdentPartType::Static(s), IdentPartType::Static(o)) => s == o,
            (IdentPartType::Dynamic(s), IdentPartType::Dynamic(o)) => s == o,
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
            part: ResolvedIdentPartType::Static(TemplateString::Ref(static_path)),
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
        Self::Static(TemplateString::Ref(static_path))
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
