use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(s: String) -> String {
    s
}

fn main() {
    let r = modifier(&mini_template::Value::String(String::from("FOO")), vec![]);
    assert_eq!(r, Ok(mini_template::Value::String(String::from("FOO"))))
}