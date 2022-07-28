use mini_template::macros::ValueContainer;
use mini_template::value;
use mini_template::MiniTemplate;

const TEMPLATE: &str = include_str!("./condition.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_default_modifiers();
    mini_template
        .add_template("0".to_owned(), TEMPLATE.to_owned())
        .unwrap();

    #[derive(ValueContainer, Clone)]
    struct Variables {
        var1: String,
        var2: usize,
    }
    let mut variables = Variables {
        var1: "HELLO world".to_owned(),
        var2: 9,
    };
    let render = mini_template.render("0", variables.clone().into());
    println!("{}", render.unwrap());

    variables.var2 = 10;
    let render = mini_template.render("0", variables.clone().into());
    println!("{}", render.unwrap());

    variables.var2 = 20;
    let render = mini_template.render("0", variables.into());
    println!("{}", render.unwrap());
}
