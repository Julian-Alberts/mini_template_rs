use std::collections::HashMap;

use crate::template::Statement;

use super::error::Result;

use super::{modifier::Modifier, value::Value};

pub struct RenderContext<'a> {
    pub modifier: &'a HashMap<&'static str, &'a Modifier>,
    pub variables: &'a HashMap<String, Value>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        modifier: &'a HashMap<&'static str, &'a Modifier>,
        variables: &'a HashMap<String, Value>,
    ) -> Self {
        Self {
            modifier,
            variables,
        }
    }
}

pub fn render<'a, 't>(tpl: &'t [Statement], context: &RenderContext<'a>) -> Result<'t, String> {
    let mut tpl_string = String::new();

    for statement in tpl {
        match statement {
            Statement::Literal(literal) => unsafe {
                // literal points to tpl.tpl_str and should never be null
                tpl_string.push_str(literal.as_ref().unwrap())
            },
            Statement::Calculated(cv) => {
                let var = cv.calc(context)?;
                tpl_string.push_str(&var.to_string()[..])
            }
            Statement::Condition(_) => todo!(),
        }
    }

    Ok(tpl_string)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        create_modifier, modifier::Modifier, parser::parse, renderer::RenderContext, value::Value,
    };

    use super::render;

    create_modifier!(
        fn upper_case_modifier(data: String) -> String {
            data.to_uppercase()
        }
    );

    create_modifier!(
        fn args_modifier(data: String, other: String, num: i32) -> String {
            format!("{}={}={}", data, other, num)
        }
    );

    #[test]
    fn literal() {
        let tpl = String::from("Simple template string");
        let tpl = parse(tpl).unwrap();
        let rendered = render(&tpl.tpl, &RenderContext::new(&HashMap::new(), &HashMap::new())).unwrap();
        assert_eq!(rendered, tpl.tpl_str);
    }

    #[test]
    fn replace_variables() {
        let tpl = String::from("Simple {foo} template string");
        let tpl = parse(tpl).unwrap();
        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));
        let rendered = render(&tpl.tpl, &RenderContext::new(&HashMap::new(), &variables)).unwrap();
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

        let rendered = render(&tpl.tpl, &RenderContext::new(&modifiers, &variables)).unwrap();
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

        let rendered = render(&tpl.tpl, &RenderContext::new(&modifiers, &variables)).unwrap();
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

        let rendered = render(&tpl.tpl, &RenderContext::new(&modifiers, &variables)).unwrap();
        assert_eq!(
            rendered,
            String::from("Simple MY TEST VALUE=bar=42 template string")
        );
    }
}
