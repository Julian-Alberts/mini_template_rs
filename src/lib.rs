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
use std::{collections::HashMap, fmt::Display, hash::Hash};
use template::Template;
use value::Value;

#[derive(Default)]
pub struct MiniTemplate<K: Eq + Hash + Display> {
    modifier: HashMap<&'static str, &'static Modifier>,
    template: HashMap<K, Template>,
}

impl<K: Eq + Hash + Display> MiniTemplate<K> {
    #[cfg(feature = "default_modifiers")]
    pub fn new_with_default_modifiers() -> MiniTemplate<K> {
        use modifier::default::*;

        let mut tpl = MiniTemplate {
            modifier: HashMap::new(),
            template: HashMap::new(),
        };

        tpl.add_modifier("slice", &slice_modifier);
        tpl.add_modifier("regex", &match_modifier);
        tpl.add_modifier("match", &match_modifier);
        tpl.add_modifier("replace", &replace_modifier);
        tpl.add_modifier("replace_regex", &replace_regex_modifier);
        tpl.add_modifier("upper", &upper);
        tpl.add_modifier("lower", &lower);
        tpl.add_modifier("repeat", &repeat);

        tpl.add_modifier("add", &add);
        tpl.add_modifier("sub", &sub);
        tpl.add_modifier("mul", &mul);
        tpl.add_modifier("div", &div);
        tpl
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
