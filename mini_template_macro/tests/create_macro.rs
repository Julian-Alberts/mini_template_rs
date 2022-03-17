use mini_template_derive::create_modifier;

#[create_modifier(modifier_name = "asd", defaults::s = 32)]
fn simple_macro(s: String) -> Result<String, String> {
    Ok(s)
}
