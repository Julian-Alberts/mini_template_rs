pub mod error;
pub mod math;
#[cfg(feature = "regex")]
pub mod regex;
pub mod string;

use std::collections::HashMap;

use super::value::Value;
use crate::ValueManager;
pub use error::*;

pub type ModifierCallback = dyn Fn(&Value, Vec<&Value>) -> Result<Value>;

pub trait Modifier {
    fn name(&self) -> &str;
    fn call(&self, subject: &Value, args: Vec<&Value>) -> Result<Value>;
}

pub(crate) struct NamedModifier<M>
where
    M: Modifier,
{
    name: String,
    inner: M,
}

impl<M> Modifier for NamedModifier<M>
where
    M: Modifier,
{
    fn name(&self) -> &str {
        &self.name
    }
    fn call(&self, subject: &Value, args: Vec<&Value>) -> Result<Value> {
        self.inner.call(subject, args)
    }
}

pub struct FunctionStyleModifier {
    name: &'static str,
    function: &'static ModifierCallback,
}

impl Modifier for FunctionStyleModifier {
    fn name(&self) -> &str {
        self.name
    }

    fn call(&self, subject: &Value, args: Vec<&Value>) -> Result<Value> {
        (self.function)(subject, args)
    }
}

pub struct ModifierContainer {
    modifiers: HashMap<String, Box<dyn Modifier>>,
}

impl Default for ModifierContainer {
    fn default() -> Self {
        Self {
            modifiers: HashMap::default(),
        }
    }
}

impl ModifierContainer {
    pub fn get(&self, key: &str) -> Option<&dyn Modifier> {
        self.modifiers.get(key).map(Box::as_ref)
    }
}

pub(crate) trait InsertModifier<K, M> {
    fn insert(&mut self, key: K, modifier: M);
}

impl InsertModifier<String, Box<dyn Modifier>> for ModifierContainer {
    fn insert(&mut self, key: String, modifier: Box<dyn Modifier>) {
        self.modifiers.insert(key, modifier);
    }
}

impl InsertModifier<&'static str, &'static ModifierCallback> for ModifierContainer {
    fn insert(&mut self, key: &'static str, modifier: &'static ModifierCallback) {
        self.insert(
            key.to_owned(),
            Box::new(FunctionStyleModifier {
                function: modifier,
                name: key,
            }),
        )
    }
}

#[mini_template_macro::create_modifier]
fn len_modifier(vm: &ValueManager) -> usize {
    vm.len()
}

pub trait ModifierGroup {
    fn get_modifiers(&self) -> Vec<Box<dyn Modifier>>;
}

pub trait AsModifier<M: Modifier> {
    fn as_modifier(&'static self, name: &'static str) -> M;
}

impl AsModifier<FunctionStyleModifier> for ModifierCallback {
    fn as_modifier(&'static self, name: &'static str) -> FunctionStyleModifier {
        FunctionStyleModifier {
            name,
            function: self,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::value_iter;

    use super::*;

    #[test]
    fn len() {
        let object = Value::Object(ValueManager::default());
        assert_eq!(
            len_modifier(&object, vec![]),
            Ok(Value::Number(0usize.into()))
        );

        let object = Value::Object(
            ValueManager::try_from_iter(value_iter!("a": Value::Null, "b": Value::Null)).unwrap(),
        );
        assert_eq!(
            len_modifier(&object, vec![]),
            Ok(Value::Number(2usize.into()))
        );
    }

    #[test]
    fn return_null() {
        #[mini_template_macro::create_modifier]
        fn my_modifier(i: usize) -> Option<usize> {
            if i < 10 {
                Some(i)
            } else {
                None
            }
        }
        assert_eq!(
            my_modifier(&Value::Number(5_usize.into()), vec![]),
            Ok(Value::Number(5_usize.into()))
        );
        assert_eq!(
            my_modifier(&Value::Number(15_usize.into()), vec![]),
            Ok(Value::Null)
        );
    }
}
