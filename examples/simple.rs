use std::collections::HashMap;

use mini_template::MiniTemplate;

const TEMPLATE: &str = include_str!("./simple.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_template(0, TEMPLATE.to_owned()).unwrap();
    let render = mini_template.render(&0, &HashMap::default());
    println!("{}", render.unwrap())
}
