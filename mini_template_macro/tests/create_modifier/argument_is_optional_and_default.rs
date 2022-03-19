use mini_template_macro::create_modifier;

#[create_modifier(defaults::_o = false)]
fn modifier(s: String, _o: Option<bool>) -> String {
    s
}

fn main() {}