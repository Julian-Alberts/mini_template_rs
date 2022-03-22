use crate::value::Value;

#[deprecated(since = "0.1.0", note = "Use mini_template::value::VariableManager")]
pub trait ValueContainer {
    fn get(&self, k: &str) -> Option<&Value>;
    fn get_mut(&mut self, k: &str) -> Option<&mut Value>;
    fn set(&mut self, k: String, v: Value);
}
