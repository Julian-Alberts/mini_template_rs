use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(s: String, u: usize) -> String {
    format!("{} {}", s, u)
}

fn main() {
    let r = modifier(&mini_template::value::Value::String(String::from("FOO")), vec![
        &mini_template::value::Value::Number(42.)
    ]);
    assert_eq!(r, Ok(mini_template::value::Value::String(String::from("FOO 42"))))
}