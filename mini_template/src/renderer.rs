use std::collections::HashMap;

use crate::value::VariableManager;

use super::modifier::Modifier;

pub struct RenderContext<'a, VM: VariableManager> {
    pub modifier: &'a HashMap<&'static str, &'a Modifier>,
    pub variables: VM,
}

impl<'a, VM: VariableManager> RenderContext<'a, VM> {
    pub fn new(modifier: &'a HashMap<&'static str, &'a Modifier>, variables: VM) -> Self {
        Self {
            modifier,
            variables,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        modifier::Modifier, parser::parse, renderer::RenderContext, template::Render, value::Value,
    };
    use mini_template_macro::create_modifier;
    use std::collections::HashMap;

    #[create_modifier]
    fn upper_case_modifier(data: String) -> String {
        data.to_uppercase()
    }

    #[create_modifier]
    fn args_modifier(data: String, other: String, num: i32) -> String {
        format!("{}={}={}", data, other, num)
    }

    #[test]
    fn literal() {
        let tpl = String::from("Simple template string");
        let tpl = parse(tpl).unwrap();
        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&HashMap::new(), HashMap::new()),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, tpl.tpl_str);
    }

    #[test]
    fn replace_variables() {
        let tpl = String::from("Simple {foo} template string");
        let tpl = parse(tpl).unwrap();
        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));
        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(&HashMap::new(), variables),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(
            rendered,
            String::from("Simple my test value template string")
        );
    }

    #[test]
    fn modifier() {
        let tpl = String::from("Simple {foo|upper} template string");
        let tpl = parse(tpl).unwrap();

        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));

        let mut modifiers: HashMap<&'static str, &Modifier> = HashMap::new();
        modifiers.insert("upper", &upper_case_modifier);
        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
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
        let tpl = String::from(r#"Simple {foo|args:"BAR":42} template string"#);
        let tpl = parse(tpl).unwrap();

        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));

        let mut modifiers: HashMap<&'static str, &Modifier> = HashMap::new();
        modifiers.insert("args", &args_modifier);
        let mut rendered = String::new();

        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
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
        let tpl = String::from(r#"Simple {foo|upper|args:"bar":42} template string"#);
        let tpl = parse(tpl).unwrap();

        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));

        let mut modifiers: HashMap<&str, &Modifier> = HashMap::new();
        modifiers.insert("args", &args_modifier);
        modifiers.insert("upper", &upper_case_modifier);

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
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
{if var1}Bar {endif}
Baz"#,
        );
        let tpl = parse(tpl).unwrap();

        let mut variables = HashMap::new();
        variables.insert("var1".to_owned(), Value::Bool(true));

        let modifiers: HashMap<&str, &Modifier> = HashMap::new();

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("Foo\nBar Baz"));
    }

    #[test]
    fn condition2() {
        let tpl = String::from("Foo\n{if var1}\nBar\n{endif}\nBaz");
        let tpl = parse(tpl).unwrap();

        let mut variables = HashMap::new();
        variables.insert("var1".to_owned(), Value::Bool(true));

        let modifiers: HashMap<&str, &Modifier> = HashMap::new();

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("Foo\nBar\nBaz"));
    }

    #[test]
    fn condition3() {
        let tpl = String::from("Foo{if var1}Bar{else}Fizz{endif}Baz");
        let tpl = parse(tpl).unwrap();

        let mut variables = HashMap::new();
        variables.insert("var1".to_owned(), Value::Bool(true));

        let modifiers: HashMap<&str, &Modifier> = HashMap::new();

        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("FooBarBaz"));

        let mut variables = HashMap::new();
        variables.insert("var1".to_owned(), Value::Bool(false));
        let mut rendered = String::new();
        tpl.render(
            &mut RenderContext::new(&modifiers, variables),
            &mut rendered,
        )
        .unwrap();
        assert_eq!(rendered, String::from("FooFizzBaz"));
    }
}
