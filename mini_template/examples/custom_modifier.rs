use mini_template::{MiniTemplate, ValueManager};
use serde_json::json;

#[macro_use]
extern crate mini_template;

const TEMPLATE: &str = include_str!("./custom_modifier.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();

    mini_template.add_modifier("is_even", &modifiers::is_even);
    mini_template.add_modifier("leading_zeros", &modifiers::leading_zeros);
    mini_template.add_modifier("as_usize", &modifiers::parse_as_usize);
    mini_template.add_modifier("nth_upper", &modifiers::nth_upper);
    mini_template.add_modifier("nth_lower", &modifiers::nth_lower);

    mini_template
        .add_template("".to_owned(), TEMPLATE.to_owned())
        .unwrap();
    let render = mini_template.render(
        "0",
        ValueManager::try_from(json!({
            "even": 4_f64,
            "zeros": 4_f64,
        }))
        .unwrap(),
    );

    println!("{}", render.unwrap())
}

mod modifiers {
    use serde_json::json;

    #[mini_template::macros::create_modifier(mini_template_crate = "mini_template")]
    fn is_even(num: usize) -> bool {
        num % 2 == 0
    }

    mini_template::fn_as_modifier!(
        fn leading_zeros(input: usize) -> u32 => usize::leading_zeros
    );

    #[mini_template::macros::create_modifier(
        mini_template_crate = "mini_template",
        returns_result = true
    )]
    fn parse_as_usize(input: String) -> Result<usize, String> {
        match input.parse::<usize>() {
            Ok(o) => Ok(o),
            Err(_) => Err(format!("Can not parse \"{input}\" as usize")),
        }
    }

    #[mini_template::macros::create_modifier(mini_template_crate = "mini_template")]
    fn nth_upper(input: String, n: Option<usize>) -> String {
        let n = n.unwrap_or(2);
        let mut buf = String::new();
        for (i, c) in input.chars().enumerate() {
            if i % n == 0 {
                buf.push(c.to_ascii_uppercase())
            } else {
                buf.push(c)
            }
        }
        buf
    }

    pub fn nth_lower(
        input: &mini_template::Value,
        args: Vec<&mini_template::Value>,
    ) -> mini_template::modifier::Result<mini_template::Value> {
        use mini_template::prelude::*;
        let input: String = match TplTryInto::try_into(input) {
            Ok(inner) => inner,
            Err(e) => {
                return Err(mini_template::modifier::Error::Type {
                    value: input.as_string(),
                    type_error: e,
                })
            }
        };

        let n: usize = match TplTryInto::try_into(*args.get(0).unwrap_or(&&json!(2_f64))) {
            Ok(inner) => inner,
            Err(e) => {
                return Err(mini_template::modifier::Error::Type {
                    value: input.to_string(),
                    type_error: e,
                })
            }
        };

        let mut buf = String::new();
        for (i, c) in input.chars().enumerate() {
            if i % n == 0 {
                buf.push(c.to_ascii_lowercase())
            } else {
                buf.push(c)
            }
        }
        Ok(mini_template::Value::String(buf))
    }
}
