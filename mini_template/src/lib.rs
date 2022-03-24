//#![deny(clippy::undocumented_unsafe_blocks)]

mod error;
pub mod macros;
pub mod modifier;
mod parser;
mod renderer;
mod template;
mod util;
pub mod value;
mod value_container;

#[macro_use]
extern crate pest_derive;

use modifier::Modifier;
use parser::{parse, ParseError};
use renderer::RenderContext;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::{collections::HashMap, hash::Hash};
use template::{Render, Template};
use value::VariableManager;

/// A Storage for Templates
///
/// A MiniTemplate instance is used to parse, save and render templates.
#[derive(Default)]
pub struct MiniTemplate {
    modifier: HashMap<&'static str, &'static Modifier>,
    template: HashMap<u64, Template>,
}

impl MiniTemplate {
    /// Creates a new instance.
    /// Use [`MiniTemplate::default`] instead.
    #[deprecated]
    pub fn new() -> Self {
        MiniTemplate {
            modifier: HashMap::new(),
            template: HashMap::new(),
        }
    }

    /// Adds the following modifiers:
    ///
    /// slice, regex, match, replace, replace_regex, upper, lower, repeat, add, sub, mul, div
    pub fn add_default_modifiers(&mut self) {
        use modifier::*;
        self.add_modifier("slice", &slice_modifier);
        #[cfg(feature = "regex")]
        {
            self.add_modifier("regex", &match_modifier);
            self.add_modifier("match", &match_modifier);
            self.add_modifier("replace_regex", &replace_regex_modifier);
        }
        self.add_modifier("replace", &replace_modifier);
        self.add_modifier("upper", &upper);
        self.add_modifier("lower", &lower);
        self.add_modifier("repeat", &repeat);

        self.add_modifier("add", &add);
        self.add_modifier("sub", &sub);
        self.add_modifier("mul", &mul);
        self.add_modifier("div", &div);
    }

    /// Register a new modifier
    ///
    /// You can implement modifiers by hand. But that will result quite complex setup code.
    /// Preferably you should take a look at the [`mini_template::modifier::create_modifier`] macro.
    pub fn add_modifier(&mut self, key: &'static str, modifier: &'static Modifier) {
        self.modifier.insert(key, modifier);
    }

    /// Register a new Template for a give key
    pub fn add_template<K: Eq + Hash>(
        &mut self,
        key: K,
        tpl: String,
    ) -> Result<Option<Template>, ParseError> {
        let tpl = parse(tpl)?;
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let key = hasher.finish();
        Ok(self.template.insert(key, tpl))
    }

    /// Render the template for a given key.
    /// # Error
    /// This function will return the following errors:
    /// * Modifier: An unhandled error occurred inside a modifier
    /// * UnknownTemplate: There is no template with the given key registered
    /// * UnknownModifier: The template contains a unknown modifier
    /// * UnknownVariable: The template contains a unknown variable
    pub fn render<VM: VariableManager, K: Eq + Hash>(
        &self,
        key: &K,
        data: VM,
    ) -> crate::error::Result<String> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let key = hasher.finish();
        let tpl = match self.template.get(&key) {
            Some(t) => t,
            None => return Err(crate::error::Error::UnknownTemplate),
        };
        let mut context = RenderContext::new(&self.modifier, data, &self.template);
        let mut buf = String::new();
        tpl.render(&mut context, &mut buf)?;
        Ok(buf)
    }
}
