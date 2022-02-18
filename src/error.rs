pub type Result<'t, T> = std::result::Result<T, ErrorKind<'t>>;

#[derive(Debug, PartialEq)]
pub enum ErrorKind<'t> {
    ModifierError(super::modifier::error::ErrorKind),
    UnknownVariable(&'t str),
    UnknownModifier(&'t str),
    UnknownTemplate,
}

impl<'t> ToString for ErrorKind<'t> {
    fn to_string(&self) -> String {
        match self {
            Self::ModifierError(e) => e.to_string(),
            Self::UnknownVariable(var_name) => format!("unknown variable {}", var_name),
            Self::UnknownModifier(modifier_name) => format!("unknown modifier {}", modifier_name),
            Self::UnknownTemplate => String::from("unknown template"),
        }
    }
}
