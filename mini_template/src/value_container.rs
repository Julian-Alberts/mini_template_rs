use std::collections::HashMap;

use crate::value::Value;

pub trait ValueContainer {
    fn get(&self, k: &str) -> Option<&Value>;
    fn get_mut(&mut self, k: &str) -> Option<&mut Value>;
    fn set(&mut self, k: String, v: Value);
}

impl ValueContainer for HashMap<String, Value> {
    fn get(&self, k: &str) -> Option<&Value> {
        self.get(k)
    }

    fn get_mut(&mut self, k: &str) -> Option<&mut Value> {
        self.get_mut(k)
    }

    fn set(&mut self, k: String, v: Value) {
        self.insert(k, v);
    }
}
