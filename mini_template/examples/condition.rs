use mini_template::value::ident::Ident;
use mini_template::{MiniTemplate, Value, ValueManager};
use serde_json::json;

const TEMPLATE: &str = include_str!("./condition.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_default_modifiers();
    mini_template
        .add_template("0".to_owned(), TEMPLATE.to_owned())
        .unwrap();

    let mut variables = ValueManager::default();
    variables
        .set_value(
            Ident::try_from("var1")
                .unwrap()
                .resolve_ident(&variables)
                .unwrap(),
            Value::String(String::from("HELLO world")),
        )
        .unwrap();
    variables
        .set_value(
            Ident::try_from("var2")
                .unwrap()
                .resolve_ident(&variables)
                .unwrap(),
            json!(9_f64),
        )
        .unwrap();
    let render = mini_template.render("0", variables);
    println!("{}", render.unwrap());

    let mut variables = ValueManager::default();
    variables
        .set_value(
            Ident::try_from("var1")
                .unwrap()
                .resolve_ident(&variables)
                .unwrap(),
            Value::String(String::from("HELLO world")),
        )
        .unwrap();
    variables
        .set_value(
            Ident::try_from("var2")
                .unwrap()
                .resolve_ident(&variables)
                .unwrap(),
            json!(10_f64),
        )
        .unwrap();
    let render = mini_template.render("0", variables);
    println!("{}", render.unwrap());

    let mut variables = ValueManager::default();
    variables
        .set_value(
            Ident::try_from("var1")
                .unwrap()
                .resolve_ident(&variables)
                .unwrap(),
            Value::String(String::from("HELLO world")),
        )
        .unwrap();
    variables
        .set_value(
            Ident::try_from("var2")
                .unwrap()
                .resolve_ident(&variables)
                .unwrap(),
            json!(20_f64),
        )
        .unwrap();
    let render = mini_template.render("0", variables);
    println!("{}", render.unwrap());
}
