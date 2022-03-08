use std::collections::HashMap;

use mini_template::{value::Value, MiniTemplate};

const TEMPLATE: &str = include_str!("./condition.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_default_modifiers();
    mini_template.add_template(0, TEMPLATE.to_owned()).unwrap();

    let mut variables = HashMap::default();
    variables.insert(
        "var1".to_owned(),
        Value::String(String::from("HELLO world")),
    );
    variables.insert("var2".to_owned(), Value::Number(9.));
    let render = mini_template.render(&0, &variables);
    println!("{}", render.unwrap());

    variables.insert("var2".to_owned(), Value::Number(10.));
    let render = mini_template.render(&0, &variables);
    println!("{}", render.unwrap());

    variables.insert("var2".to_owned(), Value::Number(20.));
    let render = mini_template.render(&0, &variables);
    println!("{}", render.unwrap());
}
