use crate::fn_as_modifier;

use core::ops::{Add, Div, Mul, Sub};

use super::{ModifierGroup, AsModifier, ModifierCallback, Modifier};

fn_as_modifier!(fn add(a: f64, b: f64) -> f64 => f64::add);

fn_as_modifier!(fn sub(a: f64, b: f64) -> f64 => f64::sub);

fn_as_modifier!(fn mul(a: f64, b: f64) -> f64 => f64::mul);

fn_as_modifier!(fn div(a: f64, b: f64) -> f64 => f64::div);

pub struct MathModifierGroup;
impl ModifierGroup for MathModifierGroup {
    fn get_modifiers(&self) -> Vec<Box<dyn Modifier>> {
        let add: &'static ModifierCallback = &add;
        let sub: &'static ModifierCallback = &sub;
        let mul: &'static ModifierCallback = &mul;
        let div: &'static ModifierCallback = &div;
        vec![
            Box::new(add.as_modifier("add")),
            Box::new(sub.as_modifier("sub")),
            Box::new(mul.as_modifier("mul")),
            Box::new(div.as_modifier("div"))
        ]
    }
}