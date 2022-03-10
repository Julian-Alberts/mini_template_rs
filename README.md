# mini_template
mini_template is a template engine written in Rust and is inspired by [smarty](https://smarty.net) and [twig](https://twig.symfony.com/). You can use this crate in production, but it is still missing some basic features.
## Motivation
In 2021 I started development on [i3_notify_bar](https://github.com/Julian-Alberts/i3_notify_bar), a program that allows for customizable notifications in i3-wm. Back then, I needed a template engine that allowed me to modify variables inside the template with a user-friendly syntax. After experimenting with different crates, I decided to develop my own.
## Template syntax
### Printing a value
To print a value, you write its name inside curly braces.
```
Hello {username}!
```
If you want to modify the value before printing, you can add a modifier like this:
Hello {username|upper}
Some modifiers might take one or more arguments. Arguments can be added by using a colon behind the modifier name and can be literal values or variables. Conditions are currently not supported.
```
Hello {username|my_modifier:var:3}
```
### Conditional branch
mini_template also supports if statements. Conditions behave as they would in almost every other language. Modifiers are also completely supported inside conditions.
```
{if var|lower == "foo" && flag}
    bar
{else}
    {var}
{endif}
```
## Usage in Rust
The mini_template API is quite simple. Most of the time, you will interact with mini_template::MiniTemplate this is the template manager responsible for all interactions with a template.
Adding a new template
```rust
use mini_template::MiniTemplate;

const TEMPLATE: &str = include_str!("./my_template.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_template("template_key", TEMPLATE.to_owned()).unwrap();
}
```
Adding default modifiers
By default, no modifiers are loaded. You can load them like this.
```rust
use mini_template::MiniTemplate;

fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_default_modifiers();
}
```
### Rendering a template
Variables used by the template are currently stored inside a HashMap.
```rust
use std::collections::HashMap;

use mini_template::MiniTemplate;

const TEMPLATE: &str = include_str!("./my_template.tpl");

fn main() {
    let mut mini_template = MiniTemplate::default();
    // Register new template
    mini_template.add_template(0, TEMPLATE.to_owned()).unwrap();
    
    // Prepare variables
    let mut variables = HashMap::default();
    variables.insert("my_var".to_owned(), Value::Bool(true));
    
    // actual rendering
    let render = mini_template.render(&0, &variables);
    println!("{}", render.unwrap())
}
```
### Creating a custom modifier
Modifiers are normal rust functions with a special header. A simple modifier could look like this:
```rust
fn always_true(&Value, Vec<&Value>) -> Result<Value> {
    Ok(Value::Bool(true))
}
```
The first argument is the value the modifier gets called on. The vector contains all arguments behind the modifier.
Creating modifiers this way can be a tedious process. If you do not need the extra flexibility, you can use the provided create_modifier! macro.
```rust
mini_template::create_modifier!(
    fn is_even(num: usize) -> bool {
        num % 2 == 0
    }
);
```
The resulting code will look some what like this:
```rust
fn is_even(value: &Value, args: Vec<&Value>) -> Result<Value> {
    let n: usize = match value.try_into() {
        Ok(inner) => inner,
        Err(e) => return Err(
            Error::Type{
                value: value.to_string(), 
                type_error: e
            }
        )
    }

    fn inner(n: usize) -> bool {
        num % 2 == 0
    }

    let result = inner(n);
    Ok(result.into())
}
```
To register a new modifier you can call MiniTemplate::add_modifier. Modifiers are always registered for all templates.
```rust
let mut mini_template = MiniTemplate::default();
mini_template.add_modifier("is_even", &is_even);
```
## Todo
* More tests
* Allow arrays in templates
* Replace hash map with new variable storage
* Assign values in a template
* Allow disabling of features
* Add loop
* Rewrite create_modifier! macro
* Allow objects in templates
* NULL values
