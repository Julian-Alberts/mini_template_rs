use mini_template_macro::create_modifier;
use serde_json::json;

#[create_modifier]
fn modifier(s: String, u: usize) -> String {
    format!("{} {}", s, u)
}

fn main() {
    let r = modifier(&mini_template::Value::String(String::from("FOO")), vec![
        &json!(42)
    ]);
    assert_eq!(r, Ok(mini_template::Value::String(String::from("FOO 42"))))
}