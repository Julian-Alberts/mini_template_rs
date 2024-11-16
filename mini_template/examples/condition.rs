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

    let mut buffer = Vec::new();
    mini_template.render(&0, variables, &mut buffer).unwrap();
    println!("{}", String::from_utf8(buffer).unwrap());

    let mut variables = HashMap::default();
    variables.insert(
        "var1".to_owned(),
        Value::String(String::from("HELLO world")),
    );
    variables.insert("var2".to_owned(), Value::Number(10.));

    buffer = Vec::new();
    mini_template.render(&0, variables, &mut buffer).unwrap();
    println!("{}", String::from_utf8(buffer).unwrap());

    let mut variables = HashMap::default();
    variables.insert(
        "var1".to_owned(),
        Value::String(String::from("HELLO world")),
    );
    variables.insert("var2".to_owned(), Value::Number(20.));

    buffer = Vec::new();
    mini_template.render(&0, variables, &mut buffer).unwrap();
    println!("{}", String::from_utf8(buffer).unwrap());
}
