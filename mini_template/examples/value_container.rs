use mini_template::macros::ValueContainer;
use mini_template::value;
use mini_template::MiniTemplateBuilder;

const TEMPLATE: &str = include_str!("./value_container.tpl");

#[derive(ValueContainer)]
struct TemplateData {
    name: String,
    #[name(userId)]
    user_id: u64,
    cart: Vec<TemplateItem>,
}

#[derive(ValueContainer)]
struct TemplateItem {
    id: u64,
    name: String,
}

fn main() {
    let mut mini_template = MiniTemplateBuilder::default()
        .with_default_modifiers()
        .build();
    mini_template
        .add_template("0".to_owned(), TEMPLATE.to_owned())
        .unwrap();

    let template_data = TemplateData {
        name: "Julian".to_owned(),
        user_id: 42,
        cart: vec![
            TemplateItem {
                id: 123,
                name: "Prod1".to_string(),
            },
            TemplateItem {
                id: 1234,
                name: "Prod2".to_string(),
            },
        ],
    };

    let render = mini_template.render("0", template_data.into());
    match render {
        Ok(r) => println!("{r}"),
        Err(e) => println!("{e}"),
    }
}
