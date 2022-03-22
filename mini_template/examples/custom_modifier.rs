use std::collections::HashMap;

use mini_template::{value::Value, MiniTemplate};

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

    mini_template.add_template(0, TEMPLATE.to_owned()).unwrap();
    let render = mini_template.render(
        &0,
        HashMap::from_iter([
            (String::from("even"), Value::Number(4.)),
            (String::from("zeros"), Value::Number(4.)),
        ]),
    );

    println!("{}", render.unwrap())
}

mod modifiers {

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
        input: &mini_template::value::Value,
        args: Vec<&mini_template::value::Value>,
    ) -> mini_template::modifier::Result<mini_template::value::Value> {
        let input: String = match input.try_into() {
            Ok(inner) => inner,
            Err(e) => {
                return Err(mini_template::modifier::Error::Type {
                    value: input.to_string(),
                    type_error: e,
                })
            }
        };

        let n: usize = match (*args
            .get(0)
            .unwrap_or(&&mini_template::value::Value::Number(2.)))
        .try_into()
        {
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
        Ok(mini_template::value::Value::String(buf))
    }
}
