use crate::{TypeError, Value, ValueManager, value::Number};

use super::{ident::ResolvedIdent, ValueContainer};

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = TypeError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => {
                let s = &s[..];
                Ok(s)
            }
            _ => Err(TypeError {
                expected_type: stringify!(&str),
                storage_type: stringify!(String),
            }),
        }
    }
}

macro_rules! value_impl {
    ($name: ident => [$($type: ty $(as $store_type: ty)?),+]) => {
        $(
            impl TryFrom<&Value> for $type {
                type Error = TypeError;
                fn try_from(value: &Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::$name(Number::ISize(s)) => Ok(*s as $type),
                        Value::$name(Number::USize(s)) => Ok(*s as $type),
                        Value::$name(Number::F32(s)) => Ok(*s as $type),
                        Value::$name(Number::F64(s)) => Ok(*s as $type),
                        _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
                    }
                }
            }

            impl TryFrom<Value> for $type {
                type Error = TypeError;
                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::$name(Number::ISize(s)) => Ok(s as $type),
                        Value::$name(Number::USize(s)) => Ok(s as $type),
                        Value::$name(Number::F32(s)) => Ok(s as $type),
                        Value::$name(Number::F64(s)) => Ok(s as $type),
                        _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
                    }
                }
            }

            impl From<$type> for Value {
                fn from(s: $type) -> Self {
                    Self::$name((s$(as $store_type)?).into())
                }
            }

        )+
    };
    ($name: ident => $type: ty) => {
        value_impl!(try_from_type $name => $type);
        value_impl!(from_value $name => $type);
    };
    (try_from_type $name: ident => $type: ty) => {
        impl TryFrom<&Value> for $type {
            type Error = TypeError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok(s.clone()),
                    _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
                }
            }
        }

        impl TryFrom<Value> for $type {
            type Error = TypeError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok(s),
                    _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
                }
            }
        }
    };
    (from_value $name: ident => $type: ty) => {
        impl From<$type> for Value {
            fn from(s: $type) -> Self {
                Self::$name(s)
            }
        }
    }

}

impl From<ValueManager> for Value {
    fn from(vm: ValueManager) -> Self {
        Self::Object(vm)
    }
}

impl <'a> TryFrom<&'a Value> for &'a ValueManager {
    type Error = TypeError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(s) => Ok(s),
            _ => Err(TypeError{ expected_type: stringify!($type), storage_type: stringify!($name)})
        }
    }
}

impl <T> ValueContainer for Vec<T> 
where T: Into<Value> {}

impl <T> From<Vec<T>> for ValueManager 
    where T: Into<Value>
{

    fn from(values: Vec<T>) -> Self {
        let mut vm = ValueManager::default();
        values.into_iter().enumerate().for_each(|(index, value)| {
            vm.set_value(
                ResolvedIdent::from(index.to_string()), 
                value.into()
            ).unwrap();
        });
        vm
    }

}

value_impl!(String => String);
value_impl!(Bool => bool);
value_impl!(Number => [f32, f64, i8 as isize, u8 as usize, i16 as isize, u16 as usize, i32 as isize, u32 as usize, i64 as isize, u64 as usize, isize, usize]);
