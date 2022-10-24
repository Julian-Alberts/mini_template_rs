use crate::template_provider::TemplateProvider;
use std::collections::HashMap;

use crate::value::ValueManager;

use super::modifier::Modifier;

pub struct RenderContext<'a> {
    pub modifier: &'a HashMap<&'static str, &'a Modifier>,
    pub variables: ValueManager,
    pub template_provider: &'a dyn TemplateProvider,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        modifier: &'a HashMap<&'static str, &'a Modifier>,
        variables: ValueManager,
        template_provider: &'a dyn TemplateProvider,
    ) -> Self {
        Self {
            modifier,
            variables,
            template_provider,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use crate::template_provider::DefaultTemplateProvider;
    use crate::{
        modifier::Modifier, parser::parse, renderer::RenderContext, template::Render, value::Value,
        value_iter, ValueManager,
    };
    use mini_template_macro::create_modifier;
    use std::collections::HashMap;

    #[create_modifier]
    fn upper_case_modifier(data: String) -> String {
        data.to_uppercase()
    }

    #[create_modifier]
    fn args_modifier(data: String, other: String, num: isize) -> String {
        format!("{}={}={}", data, other, num)
    }

    #[test]
    fn literal() {
        let tpl = String::from("Simple template string");
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();
        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(
                &HashMap::new(),
                ValueManager::default(),
                &DefaultTemplateProvider::default(),
            ),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, tpl.tpl_str);
    }

    #[test]
    fn replace_variables() {
        let tpl = String::from("Simple {{foo}} template string");
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();
        let variables = ValueManager::try_from_iter(value_iter!(
            "foo": Value::String("my test value".to_owned())
        ))
        .unwrap();
        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(
                &HashMap::new(),
                variables,
                &DefaultTemplateProvider::default(),
            ),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(
            rendered,
            String::from("Simple my test value template string")
        );
    }

    #[test]
    #[cfg(feature = "dynamic_global_access")]
    fn dynamic_global_access() {
        let tpl = String::from("Simple {{[foo]}} template string");
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();
        let variables = ValueManager::try_from_iter(value_iter!(
            "foo": Value::String("my_var".to_owned()),
            "my_var": Value::String("BAR".to_owned())
        ))
        .unwrap();

        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(&HashMap::new(), variables, &HashMap::new()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("Simple BAR template string"));
    }

    #[test]
    #[cfg(not(feature = "dynamic_global_access"))]
    fn dynamic_global_access_disabled() {
        let tpl = String::from("Simple {{[foo]}} template string");
        let tpl = parse(tpl, &ParseContextBuilder::default().build());
        assert_eq!(
            tpl,
            Err(crate::parser::ParseError::DisabledFeature(
                UnsupportedFeature::DynamicGlobalAccess
            ))
        );
    }

    #[test]
    fn modifier() {
        let tpl = String::from("Simple {{foo|upper}} template string");
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();

        let variables = ValueManager::try_from_iter(value_iter!(
            "foo": Value::String("my test value".to_owned())
        ))
        .unwrap();

        let mut modifiers: HashMap<&'static str, &Modifier> = HashMap::new();
        modifiers.insert("upper", &upper_case_modifier);
        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(
            rendered,
            String::from("Simple MY TEST VALUE template string")
        );
    }

    #[test]
    fn modifier_values() {
        let tpl = String::from(r#"Simple {{foo|args:"BAR":42}} template string"#);
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();

        let variables = ValueManager::try_from_iter(value_iter!(
            "foo": Value::String("my test value".to_owned())
        ))
        .unwrap();

        let mut modifiers: HashMap<&'static str, &Modifier> = HashMap::new();
        modifiers.insert("args", &args_modifier);
        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(
            rendered,
            String::from("Simple my test value=BAR=42 template string")
        );
    }

    #[test]
    fn modifier_list() {
        let tpl = String::from(r#"Simple {{foo|upper|args:"bar":42}} template string"#);
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();

        let variables = ValueManager::try_from_iter(value_iter!(
            "foo": Value::String("my test value".to_owned())
        ))
        .unwrap();

        let mut modifiers: HashMap<&str, &Modifier> = HashMap::new();
        modifiers.insert("args", &args_modifier);
        modifiers.insert("upper", &upper_case_modifier);

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(
            rendered,
            String::from("Simple MY TEST VALUE=bar=42 template string")
        );
    }

    #[test]
    fn condition1() {
        let tpl = String::from(
            r#"Foo
{%if var1%}Bar {%endif%}
Baz"#,
        );
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();

        let variables = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true)
        ))
        .unwrap();

        let modifiers: HashMap<&str, &Modifier> = HashMap::new();

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("Foo\nBar Baz"));
    }

    #[test]
    fn condition2() {
        let tpl = String::from("Foo\n{%if var1%}\nBar\n{%endif%}\nBaz");
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();

        let variables = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true)
        ))
        .unwrap();

        let modifiers: HashMap<&str, &Modifier> = HashMap::new();

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("Foo\nBar\nBaz"));
    }

    #[test]
    fn condition3() {
        let tpl = String::from("Foo{%if var1%}Bar{%else%}Fizz{%endif%}Baz");
        let tpl = parse(tpl, &ParseContextBuilder::default().build()).unwrap();

        let variables = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(true)
        ))
        .unwrap();

        let modifiers: HashMap<&str, &Modifier> = HashMap::new();

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("FooBarBaz"));

        let variables = ValueManager::try_from_iter(value_iter!(
            "var1": Value::Bool(false)
        ))
        .unwrap();
        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables, &DefaultTemplateProvider::default()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("FooFizzBaz"));
    }
}
