use mini_template::MiniTemplate;
use mini_template_macro::ValueContainer;
use mini_template::value;

const TEMPLATE: &str = include_str!("./value_container.tpl");

#[derive(ValueContainer)]
struct TemplateData {
    name: String,
    #[name(userId)]
    user_id: u64,
}

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template
        .add_template("0".to_owned(), TEMPLATE.to_owned())
        .unwrap();

    let template_data = TemplateData {
        name: "Julian".to_owned(),
        user_id: 42
    };
    
    let render = mini_template.render("0", template_data.into());
    println!("{}", render.unwrap())
}