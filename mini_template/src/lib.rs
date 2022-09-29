//#![deny(clippy::undocumented_unsafe_blocks)]

pub mod error;
pub mod macros;
pub mod modifier;
mod parser;
mod renderer;
mod template;
mod util;
pub mod value;
pub mod template_provider;

#[macro_use]
extern crate pest_derive;

use crate::value::{TypeError, Value};
use modifier::Modifier;
use parser::ParseContextBuilder;
pub use parser::{ParseError, UnsupportedFeature};
pub use renderer::RenderContext;
use template_provider::{TemplateProvider, DefaultTemplateProvider};
use std::collections::HashMap;
use template::Template;
pub use template::{CustomBlock, CustomBlockParser, Render};
pub use value::ValueManager;

#[cfg(parser)]
pub use parser::export as parse;

/// A Storage for Templates
///
/// A MiniTemplate instance is used to parse, save and render templates.
/// ```ignore
/// # // This test does not compile. There seams to be an error inside the ValueContainer macro. The macro tries to use `crate` instead of `mini_template` 
/// use mini_template::MiniTemplate;
/// use mini_template::macros::ValueContainer;
/// use mini_template::value;
/// #[derive(Clone, ValueContainer)]
/// struct TplData {
///    foo: String,
///    bar: usize,
/// }
/// let mut mini = MiniTemplate::default();
/// mini.add_template("foo".to_owned(), "{{foo}}".to_string()).unwrap();
/// mini.add_template("bar".to_owned(), "{% if bar > 10%} {{foo|upper}} {%else%} {{foo|lower}} {%end if%}".to_string()).unwrap();
///
/// # let foo =
/// mini.render("foo", TplData {
///     foo: "Me".to_string(),
///     bar: 10
/// }.into()).unwrap();
/// # let bar1 =
/// mini.render("bar", TplData {
///     foo: "Me".to_string(),
///     bar: 1
/// }.into()).unwrap();
/// # let bar2 =
/// mini.render("bar", TplData {
///     foo: "Me".to_string(),
///     bar: 11
/// }.into()).unwrap();
/// # assert_eq!(foo.as_str(), "Me");
/// # assert_eq!(bar1.as_str(), "ME");
/// # assert_eq!(bar2.as_str(), "me");
/// ```
pub struct MiniTemplate {
    modifier: HashMap<&'static str, &'static Modifier>,
    custom_blocks: HashMap<String, Box<dyn CustomBlockParser>>,
    template_provider: Box<dyn TemplateProvider>
}

impl MiniTemplate {
    /// Creates a new instance.
    /// Use [`MiniTemplate::default`] instead.
    #[deprecated]
    pub fn new() -> Self {
        MiniTemplate {
            modifier: HashMap::new(),
            template_provider: Box::new(DefaultTemplateProvider::default()),
            custom_blocks: HashMap::new(),
        }
    }

    pub fn new_with_template_provider(template_provider: Box<dyn TemplateProvider>) -> Self {
        Self {
            template_provider,
            ..Default::default()
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
        self.add_modifier("len", &len_modifier)
    }

    /// Register a new custom block
    pub fn add_custom_block(&mut self, custom_block: Box<dyn CustomBlockParser>) {
        self.custom_blocks
            .insert(custom_block.name().to_owned(), custom_block);
    }

    /// Register a new modifier
    ///
    /// You can implement modifiers by hand. But that will result quite complex setup code.
    /// Preferably you should take a look at the [`mini_template::modifier::create_modifier`] macro.
    pub fn add_modifier(&mut self, key: &'static str, modifier: &'static Modifier) {
        self.modifier.insert(key, modifier);
    }

    /// Register a new Template for a give key
    pub fn add_template(
        &mut self,
        key: String,
        tpl: String,
    ) -> Result<(), ParseError> {
        let context = ParseContextBuilder::default()
            .custom_blocks(&self.custom_blocks)
            .build();
        let tpl = parser::parse(tpl, context)?;
        self.template_provider.insert_template(key, tpl);
        Ok(())
    }

    /// Render the template for a given key.
    /// # Error
    /// This function will return the following errors:
    /// * Modifier: An unhandled error occurred inside a modifier
    /// * UnknownTemplate: There is no template with the given key registered
    /// * UnknownModifier: The template contains a unknown modifier
    /// * UnknownVariable: The template contains a unknown variable
    pub fn render(&self, key: &str, data: ValueManager) -> crate::error::Result<String> {
        let tpl = match self.template_provider.get_template(key) {
            Some(t) => t,
            None => return Err(crate::error::Error::UnknownTemplate),
        };
        let mut context = RenderContext::new(&self.modifier, data, self.template_provider.as_ref());
        let mut buf = String::new();
        tpl.render(&mut context, &mut buf)?;
        Ok(buf)
    }
}

impl Default for MiniTemplate {

    fn default() -> Self {
        Self { modifier: HashMap::default(), custom_blocks: HashMap::default(), template_provider: Box::new(DefaultTemplateProvider::default()) }
    }

}

#[cfg(test)]
mod tests {
    use mini_template_macro::ValueContainer;

    use crate::{
        value::{
            ident::{Ident, ResolvedIdent},
            Value,
        },
        MiniTemplate, ValueManager,
    };

    #[test]
    fn add_default_modifiers() {
        let mut engine = MiniTemplate::default();
        engine.add_default_modifiers();
        let modifier_names = vec![
            "slice",
            #[cfg(feature = "regex")]
            "regex",
            #[cfg(feature = "regex")]
            "match",
            #[cfg(feature = "regex")]
            "replace_regex",
            "replace",
            "upper",
            "lower",
            "repeat",
            "add",
            "sub",
            "mul",
            "div",
            "len",
        ];

        modifier_names.iter().for_each(|name| {
            assert!(
                engine.modifier.keys().any(|found| found == name),
                "Could not find modifier \"{name}\""
            );
        });
    }

    #[test]
    fn try_rendering_unknown_template() {
        let engine = MiniTemplate::default();
        assert_eq!(
            engine.render("template", ValueManager::default()),
            Err(super::error::Error::UnknownTemplate)
        )
    }

    #[test]
    fn old_style() {
        const TEMPLATE: &str = r##"
{%if user.name == "Jon"%}
    Hi {{user.name}}
{%else%}
    {{greeting}} {{user.name}}
{%endif%}
"##;
        let mut mini_template = MiniTemplate::default();
        mini_template
            .add_template("test".to_owned(), TEMPLATE.to_owned())
            .unwrap();

        let mut data = ValueManager::default();
        data.set_value(
            Ident::try_from("greeting")
                .unwrap()
                .resolve_ident(&data)
                .unwrap(),
            Value::String("Hello".to_owned()),
        )
        .unwrap();
        data.set_value(
            Ident::try_from("user.name")
                .unwrap()
                .resolve_ident(&data)
                .unwrap(),
            Value::String("Jon".to_owned()),
        )
        .unwrap();

        let output = mini_template.render("test", data.clone()).unwrap();
        assert_eq!(output.trim(), "Hi Jon");

        data.set_value(
            Ident::try_from("user.name")
                .unwrap()
                .resolve_ident(&data)
                .unwrap(),
            Value::String("David".to_owned()),
        )
        .unwrap();

        let output = mini_template.render("test", data.clone()).unwrap();
        assert_eq!(output.trim(), "Hello David");
    }

    #[test]
    fn new_style() {
        const TEMPLATE: &str = r##"
{%if user.name == "Jon"%}
    Hi {{user.name}}
{%else%}
    {{greeting}} {{user.name}}
{%endif%}
"##;
        let mut mini_template = MiniTemplate::default();
        mini_template
            .add_template("test".to_owned(), TEMPLATE.to_owned())
            .unwrap();

        #[derive(Clone)]
        struct TplData {
            user: TplUser,
            greeting: String,
        }

        impl From<TplData> for ValueManager {
            fn from(data: TplData) -> ValueManager {
                let mut vm = ValueManager::default();
                vm.set_value(
                    ResolvedIdent::from("user"),
                    ValueManager::from(data.user).into(),
                )
                .unwrap();
                vm.set_value(ResolvedIdent::from("greeting"), data.greeting.into())
                    .unwrap();
                vm
            }
        }

        #[derive(Clone)]
        struct TplUser {
            name: String,
        }
        impl From<TplUser> for ValueManager {
            fn from(data: TplUser) -> ValueManager {
                let mut vm = ValueManager::default();
                vm.set_value(ResolvedIdent::from("name"), data.name.into())
                    .unwrap();
                vm
            }
        }

        let mut data = TplData {
            user: TplUser {
                name: "Jon".to_owned(),
            },
            greeting: "Hello".to_owned(),
        };

        let output = mini_template
            .render("test", ValueManager::from(data.clone()))
            .unwrap();
        assert_eq!(output.trim(), "Hi Jon");

        data.user.name = "David".to_owned();

        let output = mini_template.render("test", data.into()).unwrap();
        assert_eq!(output.trim(), "Hello David");
    }

    #[test]
    fn derive_style() {
        const TEMPLATE: &str = r##"
{%if user.name == "Jon"%}
    Hi {{user.name}}
{%else%}
    {{greeting}} {{user.name}}
{%endif%}
"##;
        let mut mini_template = MiniTemplate::default();
        mini_template
            .add_template("test".to_owned(), TEMPLATE.to_owned())
            .unwrap();

        #[derive(Clone, ValueContainer)]
        struct TplData {
            user: TplUser,
            greeting: String,
        }

        #[derive(Clone, ValueContainer)]
        struct TplUser {
            name: String,
        }

        let mut data = TplData {
            user: TplUser {
                name: "Jon".to_owned(),
            },
            greeting: "Hello".to_owned(),
        };

        let output = mini_template.render("test", data.clone().into()).unwrap();
        assert_eq!(output.trim(), "Hi Jon");

        data.user.name = "David".to_owned();

        let output = mini_template.render("test", data.into()).unwrap();
        assert_eq!(output.trim(), "Hello David");
    }
}
