use super::{parser, Assign, Block, LiteralText, Print, Statement, WhileLoop};

pub struct ConvertContext<'tm> {
    modifiers: &'tm (),
}

pub trait ConvertFrom<'tm, T> {
    fn convert_from(value: T, ctx: &ConvertContext<'tm>) -> Self;
}

pub trait ConvertInto<'tm, T> {
    fn convert_into(self, ctx: &ConvertContext<'tm>) -> T;
}

impl<'tm, A, B> ConvertInto<'tm, A> for B
where
    A: ConvertFrom<'tm, B>,
{
    fn convert_into(self, ctx: &ConvertContext<'tm>) -> A {
        ConvertFrom::convert_from(self, ctx)
    }
}

impl<'a, 'tm> ConvertFrom<'tm, parser::Statement<'a>> for Statement<'a, 'tm> {
    fn convert_from(value: parser::Statement<'a>, ctx: &ConvertContext) -> Self {
        match value {
            parser::Statement::Block(block) => Self::Block(block.convert_into(ctx)),
            parser::Statement::Print(print) => Self::Print(print.convert_into(ctx)),
            parser::Statement::Assign(assign) => Self::Assign(assign.convert_into(ctx)),
            parser::Statement::LiteralText(literal_text) => {
                Self::LitText(literal_text.convert_into(ctx))
            }
            parser::Statement::WhileLoop(while_loop) => {
                Self::WhileLoop(while_loop.convert_into(ctx))
            }
            parser::Statement::EachLoop(each_loop) => Self::EachLoop(each_loop.convert_into(ctx)),
            parser::Statement::Conditional(conditional) => {
                Self::Conditional(conditional.convert_into(ctx))
            }
        }
    }
}

impl<'a, 'tm> ConvertFrom<'tm, parser::Block<'a>> for Block<'a, 'tm> {
    fn convert_from(value: parser::Block<'a>, _ctx: &ConvertContext) -> Self {
        let (parser::BlockName::Ident(parser::Ident { ident: name })
        | parser::BlockName::String(parser::LitString { str: name })) = value.block_name;
        let content = value.content;
        Self {
            name,
            content: vec![],
        }
    }
}

impl<'a, 'tm> ConvertFrom<'tm, parser::Print<'a>> for Print<'a, 'tm> {
    fn convert_from(value: parser::Print<'a>, ctx: &ConvertContext) -> Self {
        Print {
            expr: value.expr.convert_into(ctx),
        }
    }
}

impl<'a, 'tm> ConvertFrom<'tm, parser::Assign<'a>> for Assign<'a, 'tm> {
    fn convert_from(value: parser::Assign<'a>, ctx: &ConvertContext) -> Self {
        Self {
            ident: value.ident.convert_into(ctx),
            expr: value.statement.convert_into(ctx),
        }
    }
}

impl<'a, 'tm> ConvertFrom<'tm, parser::LiteralText<'a>> for LiteralText<'a> {
    fn convert_from(value: parser::LiteralText<'a>, ctx: &ConvertContext) -> Self {
        Self { text: value.text }
    }
}

impl<'a, 'tm> ConvertFrom<'tm, parser::WhileLoop<'a>> for WhileLoop<'a, 'tm> {
    fn convert_from(value: parser::WhileLoop<'a>, ctx: &ConvertContext<'tm>) -> Self {
        Self {
            condition: value.cond.convert_into(ctx),
            body: value
                .block
                .content
                .into_iter()
                .filter_map(|c| match c {
                    parser::BlockContentValue::Instruction(statement) => Some(statement),
                    parser::BlockContentValue::Comment => None,
                })
                .map(|s| s.convert_into(ctx))
                .collect::<Vec<_>>(),
        }
    }
}
