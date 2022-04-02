use mini_template::value::ident::Ident;
use mini_template::{value::Value, MiniTemplate, ValueManager};

const TEMPLATE: &str = include_str!("./condition.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_default_modifiers();
    mini_template
        .add_template(0_usize, TEMPLATE.to_owned())
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
            Value::Number(9.),
        )
        .unwrap();
    let render = mini_template.render(&0, variables);
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
            Value::Number(10.),
        )
        .unwrap();
    let render = mini_template.render(&0, variables);
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
            Value::Number(20.),
        )
        .unwrap();
    let render = mini_template.render(&0, variables);
    println!("{}", render.unwrap());
}
