mod convert;
pub mod parser;

use crate::{error::Result, renderer::RenderContext, value::Value};

type VC = std::collections::HashMap<String, Value>;

#[derive(Debug, PartialEq)]
pub struct Template {
    pub(crate) tpl_str: String,
    pub(crate) tpl: Vec<Statement<'static, 'static>>,
}

impl Template {
    pub fn render(&self, writer: &mut impl std::io::Write, ctx: RenderContext<VC>) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct Block<'a, 'op> {
    name: &'a str,
    content: Vec<Statement<'a, 'op>>,
}
impl<'a, 'op> Block<'a, 'op> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct Print<'a, 'op> {
    expr: Expr<'a, 'op>,
}
impl<'a, 'op> Print<'a, 'op> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct Ident<'a>(&'a str);

#[derive(Debug, PartialEq)]
struct Assign<'a, 'op> {
    ident: Ident<'a>,
    expr: Expr<'a, 'op>,
}
impl<'a, 'op> Assign<'a, 'op> {
    fn eval(&self, ctx: &mut RenderContext<VC>) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct LiteralText<'a> {
    text: &'a str,
}
impl<'a> LiteralText<'a> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct WhileLoop<'a, 'op> {
    condition: Expr<'a, 'op>,
    body: Vec<Statement<'a, 'op>>,
}
impl<'a, 'op> WhileLoop<'a, 'op> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct EachLoop<'a, 'op> {
    target: Ident<'a>,
    src: Expr<'a, 'op>,
    body: Vec<Statement<'a, 'op>>,
}
impl<'a, 'op> EachLoop<'a, 'op> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct Conditional<'a, 'op> {
    condition: Expr<'a, 'op>,
    body: Vec<Statement<'a, 'op>>,
}

impl<'a, 'op> Conditional<'a, 'op> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
enum Statement<'a, 'op> {
    Conditional(Conditional<'a, 'op>),
    EachLoop(EachLoop<'a, 'op>),
    WhileLoop(WhileLoop<'a, 'op>),
    LitText(LiteralText<'a>),
    Assign(Assign<'a, 'op>),
    Print(Print<'a, 'op>),
    Block(Block<'a, 'op>),
}

impl<'a, 'op> Statement<'a, 'op> {
    fn eval<W>(&self, ctx: &mut RenderContext<VC>, write: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Statement::Conditional(conditional) => conditional.eval(ctx, write),
            Statement::EachLoop(each_loop) => each_loop.eval(ctx, write),
            Statement::WhileLoop(while_loop) => while_loop.eval(ctx, write),
            Statement::LitText(literal_text) => literal_text.eval(ctx, write),
            Statement::Assign(assign) => assign.eval(ctx),
            Statement::Print(print) => print.eval(ctx, write),
            Statement::Block(block) => block.eval(ctx, write),
        }
    }
}

impl<'a> Modifier<'a> {
    fn eval(&self, ctx: &mut RenderContext<VC>) -> Result<()> {
        todo!()
    }
}

impl Value {
    fn eval(&self, ctx: &mut RenderContext<VC>) -> Result<()> {
        todo!()
    }
}

trait BinaryOperation<'a, 'op> {
    fn eval(
        &self,
        left: &Expr<'a, 'op>,
        right: &Expr<'a, 'op>,
        ctx: &RenderContext<std::collections::HashMap<String, Value>>,
    ) -> Value;
    fn op(&self) -> &str;
}

pub struct Binary<'a, 'op> {
    left: Box<Expr<'a, 'op>>,
    op: &'op dyn BinaryOperation<'a, 'op>,
    right: Box<Expr<'a, 'op>>,
}

impl<'a, 'op> std::fmt::Debug for Binary<'a, 'op> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Binary")
            .field("left", &self.left)
            .field("op", &self.op.op())
            .field("right", &self.right)
            .finish()
    }
}

impl<'a, 'op> Binary<'a, 'op> {
    fn eval(&self, ctx: &mut RenderContext<VC>) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
enum Expr<'a, 'op> {
    Value(Value),
    Modifier(Modifier<'a, 'op>),
    Binary(Binary<'a, 'op>),
}

impl<'a, 'op> Expr<'a, 'op> {
    fn eval(&self, ctx: &mut RenderContext<VC>) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct Modifier<'a, 'op> {
    name: &'a str,
    args: Vec<Expr<'a, 'op>>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{template::parser::parse, value::Value};

    #[test]
    fn move_template() {
        let template = parse("template {{test}} template").unwrap();
        let old_addr = &template as *const _ as usize;
        let template = Box::new(template);
        let new_addr = template.as_ref() as *const _ as usize;
        assert_ne!(new_addr, old_addr);
        let mut buf = Vec::new();
        template
            .render(
                &mut crate::renderer::RenderContext {
                    modifier: &Default::default(),
                    variables: HashMap::from_iter([("test".to_string(), Value::Number(12.0))]),
                },
                &mut buf,
            )
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "template 12 template");
    }
}
