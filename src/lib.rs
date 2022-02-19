#![deny(clippy::undocumented_unsafe_blocks)]

mod error;
mod modifier;
mod parser;
mod prelude;
mod renderer;
mod template;
pub mod value;

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate log;

use modifier::Modifier;
use parser::{parse, ParseError};
use renderer::render;
use template::Template;
use std::{collections::HashMap, fmt::Display, hash::Hash};
use value::Value;

#[derive(Default)]
pub struct MiniTemplate<K: Eq + Hash + Display> {
    modifier: HashMap<&'static str, &'static Modifier>,
    template: HashMap<K, Template>,
}

impl<K: Eq + Hash + Display> MiniTemplate<K> {
    #[deprecated]
    pub fn new() -> Self {
        MiniTemplate {
            modifier: HashMap::new(),
            template: HashMap::new(),
        }
    }

    pub fn add_default_modifiers(&mut self) {
        use modifier::*;
        self.add_modifier("slice", &slice_modifier);
        self.add_modifier("regex", &match_modifier);
        self.add_modifier("match", &match_modifier);
        self.add_modifier("replace", &replace_modifier);
        self.add_modifier("replace_regex", &replace_regex_modifier);
        self.add_modifier("upper", &upper);
        self.add_modifier("lower", &lower);
        self.add_modifier("repeat", &repeat);

        self.add_modifier("add", &add);
        self.add_modifier("sub", &sub);
        self.add_modifier("mul", &mul);
        self.add_modifier("div", &div);
    }

    pub fn add_modifier(&mut self, key: &'static str, modifier: &'static Modifier) {
        self.modifier.insert(key, modifier);
    }

    pub fn add_template(&mut self, key: K, tpl: String) -> Result<Option<Template>, ParseError> {
        let tpl = parse(tpl)?;
        Ok(self.template.insert(key, tpl))
    }

    pub fn render(&self, key: &K, data: &HashMap<String, Value>) -> error::Result<String> {
        let tpl = match self.template.get(key) {
            Some(t) => t,
            None => return Err(error::Error::UnknownTemplate),
        };
        render(tpl, &self.modifier, data)
    }
}
