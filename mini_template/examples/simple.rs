use mini_template::{MiniTemplate, ValueManager};

const TEMPLATE: &str = include_str!("./simple.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template
        .add_template("0".to_owned(), TEMPLATE.to_owned())
        .unwrap();
    let render = mini_template.render("0", ValueManager::default());
    println!("{}", render.unwrap())
}
