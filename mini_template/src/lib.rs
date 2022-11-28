//#![deny(clippy::undocumented_unsafe_blocks)]

pub mod error;
pub mod macros;
pub mod modifier;
pub mod parser;
mod renderer;
mod template;
pub mod template_provider;
mod util;
pub mod value;

#[macro_use]
extern crate pest_derive;

use crate::value::{TypeError, Value};
use modifier::{InsertModifier, Modifier, ModifierCallback, ModifierContainer, ModifierGroup};
use parser::ParseContextBuilder;
pub use parser::{ParseError, UnsupportedFeature};
pub use renderer::RenderContext;
use std::{collections::HashMap, sync::Arc};
pub use template::Template;
pub use template::{CustomBlock, CustomBlockParser, Render};
use template_provider::{DefaultTemplateProvider, TemplateProvider};
pub use value::ValueManager;

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
    modifier: ModifierContainer,
    custom_blocks: HashMap<String, Box<dyn CustomBlockParser>>,
    template_provider: Box<dyn TemplateProvider>,
}

impl MiniTemplate {
    /// Register a new Template for a give key
    pub fn add_template(&mut self, key: String, tpl: String) -> Result<(), ParseError> {
        let context = ParseContextBuilder::default()
            .custom_blocks(&self.custom_blocks)
            .build();
        let tpl = parser::parse(tpl, &context)?;
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
            Ok(Some(t)) => t,
            Ok(None) | Err(_) => return Err(crate::error::Error::UnknownTemplate),
        };
        let mut context = RenderContext::new(&self.modifier, data, self.template_provider.as_ref());
        let mut buf = String::new();
        tpl.render(&mut context, &mut buf)?;
        Ok(buf)
    }
}

pub struct MiniTemplateBuilder {
    modifier: ModifierContainer,
    custom_blocks: HashMap<String, Box<dyn CustomBlockParser>>,
    template_provider: Box<dyn TemplateProvider>,
}

impl MiniTemplateBuilder {
    pub fn build(self) -> MiniTemplate {
        MiniTemplate {
            modifier: self.modifier,
            custom_blocks: self.custom_blocks,
            template_provider: self.template_provider,
        }
    }

    pub fn with_custom_block(mut self, custom_block: Box<dyn CustomBlockParser>) -> Self {
        self.custom_blocks
            .insert(custom_block.name().to_owned(), custom_block);
        self
    }

    pub fn build_with_template_provider<T, F>(
        mut self,
        template_provider_builder: F,
    ) -> MiniTemplate
    where
        T: TemplateProvider + 'static,
        F: FnOnce() -> T,
    {
        self.template_provider = Box::new(template_provider_builder());
        self.build()
    }

    /// Adds the following modifiers:
    ///
    /// slice, regex, match, replace, replace_regex, upper, lower, repeat, add, sub, mul, div
    pub fn with_default_modifiers(self) -> Self {
        use modifier::{math::*, regex::*, string::*, *};
        let s = self
            .with_modifier_group(&StringModifierGroup)
            .with_modifier_group(&MathModifierGroup)
            .with_modifier("len", &len_modifier);

        #[cfg(feature = "regex")]
        {
            let cache = Default::default();
            s.with_modifier_group(&RegexModifierGroup::new(Arc::clone(&cache)))
                .with_boxed_modifier(Box::new(MatchModifier::new(cache)))
        }
        #[cfg(not(feature = "regex"))]
        s
    }

    /// Register a new modifier
    ///
    /// You can implement modifiers by hand. But that will result quite complex setup code.
    /// Preferably you should take a look at the [`mini_template::modifier::create_modifier`] macro.
    pub fn with_modifier(mut self, key: &'static str, modifier: &'static ModifierCallback) -> Self {
        self.modifier.insert(key, modifier);
        self
    }

    pub fn with_boxed_modifier(mut self, modifier: Box<dyn Modifier>) -> Self {
        self.modifier.insert(modifier.name().to_owned(), modifier);
        self
    }

    /// Register a new Group of modifiers
    pub fn with_modifier_group(mut self, group: &dyn ModifierGroup) -> Self {
        group
            .get_modifiers()
            .into_iter()
            .for_each(|m| self.modifier.insert(m.name().to_owned(), m));
        self
    }
}

impl Default for MiniTemplateBuilder {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            custom_blocks: Default::default(),
            template_provider: Box::new(DefaultTemplateProvider::default()),
        }
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
        MiniTemplateBuilder, ValueManager,
    };

    #[test]
    fn try_rendering_unknown_template() {
        let engine = MiniTemplateBuilder::default().build();
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
        let mut mini_template = MiniTemplateBuilder::default().build();
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
        let mut mini_template = MiniTemplateBuilder::default().build();
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
        let mut mini_template = MiniTemplateBuilder::default().build();
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
