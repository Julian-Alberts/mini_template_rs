# :warning: I am currently working on 0.2.0. Please expect breaking changes.
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
### Loops
mini_template currently only knows while loops. While loops repeat as in any other language
as long as the condition is `true`. A simple for loop looks like this.
```
{i = 0}
{while i < 10}
    {i}
    {i = i|add:1}
{endwhile}
```
### Assigning a value
To assign a value, you have to give it an identifier followed by an equals sign. After that, you can specify what value to set.
```
{new_var = "foo"|upper}
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
    let render = mini_template.render(&0, variables);
    println!("{}", render.unwrap())
}
```
### Creating a custom modifier
Modifiers are normal rust functions with a special header. A simple modifier could look like this:
```rust
fn always_true(value: &Value, args: Vec<&Value>) -> Result<Value> {
    Ok(Value::Bool(true))
}
```
The first argument is the value the modifier gets called on. The vector contains all arguments behind the modifier.
#### Basic macro use
Creating modifiers this way can be a tedious process. If you do not need the extra flexibility, you can use the provided `create_modifier` macro.
```rust
#[mini_template::macros::create_modifier]
fn is_even(num: usize) -> bool {
    num % 2 == 0
}
```
The resulting code will look somewhat like this:
```rust
fn is_even(value: &Value, args: Vec<&Value>) -> mini_template::modifier::error::Result<Value> {
    let num: usize = match value.try_into() {
        Ok(inner) => inner,
        Err(e) => return Err(
            Error::Type{
                value: value.to_string(), 
                type_error: e
            }
        )
    };

    fn is_even(num: usize) -> bool {
        num % 2 == 0
    }

    let result = is_even(num);
    Ok(result.into())
}
```
By default, the function will be overridden. If you want to keep your original function 
you can add the `modifier_ident` attribute. This will generate a new method for your 
modifier with the given name.
```rust
#[mini_template::macros::create_modifier(modifier_ident = "is_even_modifier")]
fn is_even(num: usize) -> bool {
    num % 2 == 0
}

fn main() {
    assert!(is_even(12));
    assert!(is_even_modifier(&Value::Number(12.0), &Vec::default()))
}
```


#### Results
In case your modifier needs to return a result you can write your modifier like this.
```rust
#[mini_template::macros::create_modifier(returns_result = true)]
fn parse(s: &str) -> Result<usize, String> {
    match s.parse::<usize>() {
        Ok(s) => Ok(s),
        Err(_) => Err(format!("Can not convert \"{}\" to usize", s))
    }
}
```
Note the `returns_result` argument for `create_modifier`. The Result must be of type `Result<_, String>`.

#### Use function as modifier
Sometimes you want to use an existing function as a modifier. That can be achieved by using 
the `mini_template::fn_as_modifier` macro. The first part describes the function 
signature followed by an `=>` and the actual function to call.
```rust
fn_as_modifier!(fn add(a: f64, b: f64) -> f64 => f64::add);
```

#### Registering a modifier
To register a new modifier you can call MiniTemplate::add_modifier. Modifiers are always registered for all templates.
```rust
fn main() {
    let mut mini_template = MiniTemplate::default();
    mini_template.add_modifier("is_even", &is_even);
}
```
## Todo
* More tests
* Allow arrays in templates
* Allow objects in templates
* NULL values
