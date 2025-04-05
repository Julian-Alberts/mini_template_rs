use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "template-new.pest"]
struct TemplateParser;

pub type Error = pest::error::Error<Rule>;

pub fn parse(template: &str) -> Result<Template, pest::error::Error<Rule>> {
    let mut t = TemplateParser::parse(Rule::template, template)?;
    let Some(template) = t.next_as() else {
        unreachable!()
    };
    Ok(template)
}

trait Parse<'a>: Sized {
    fn parse(item: Pair<'a, Rule>) -> Self;
}

#[derive(Debug, PartialEq)]
pub struct Template<'a> {
    block_content: BlockContent<'a>,
}

impl<'a> Parse<'a> for Template<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::template);
        let Some(block_content) = item.into_inner().next_as() else {
            unreachable!()
        };
        Template { block_content }
    }
}

#[derive(Debug, PartialEq)]
pub struct BlockContent<'a> {
    pub content: Vec<BlockContentValue<'a>>,
}

impl<'a> Parse<'a> for BlockContent<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::block_content);
        let content = item
            .into_inner()
            .map(BlockContentValue::parse)
            .collect::<_>();
        BlockContent { content }
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockContentValue<'a> {
    Instruction(Statement<'a>),
    Comment,
}

impl<'a> Parse<'a> for BlockContentValue<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert!(matches!(item.as_rule(), Rule::instruction | Rule::comment));
        let content = match item.as_rule() {
            Rule::instruction => Self::Instruction(Parse::parse(item)),
            Rule::comment => Self::Comment,
            _ => unreachable!(),
        };
        content
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement<'a> {
    Block(Block<'a>),
    Print(Print<'a>),
    Assign(Assign<'a>),
    LiteralText(LiteralText<'a>),
    WhileLoop(WhileLoop<'a>),
    EachLoop(EachLoop<'a>),
    Conditional(Conditional<'a>),
}

impl<'a> Parse<'a> for Statement<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::instruction);
        let Some(inner_item) = item.into_inner().next() else {
            unreachable!()
        };
        match inner_item.as_rule() {
            Rule::block => Self::Block(Block::parse(inner_item)),
            Rule::print => Self::Print(Print::parse(inner_item)),
            Rule::assign => Self::Assign(Assign::parse(inner_item)),
            Rule::literal_text => Self::LiteralText(LiteralText::parse(inner_item)),
            Rule::while_loop => Self::WhileLoop(WhileLoop::parse(inner_item)),
            Rule::each_loop => Self::EachLoop(EachLoop::parse(inner_item)),
            Rule::conditional => Self::Conditional(Conditional::parse(inner_item)),
            r => unreachable!("Unexpected {r:?}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Print<'a> {
    pub expr: Expr<'a>,
}

impl<'a> Parse<'a> for Print<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::print);
        let Some(expr) = item.into_inner().next_as() else {
            unreachable!()
        };
        Print { expr }
    }
}

#[derive(Debug, PartialEq)]
pub struct Assign<'a> {
    pub ident: Ident<'a>,
    pub statement: Expr<'a>,
}

impl<'a> Parse<'a> for Assign<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::assign);
        let mut inner = item.into_inner();
        let ident = Ident::parse(inner.next().unwrap());
        let statement = Expr::parse(inner.next().unwrap());
        Assign { ident, statement }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    Modifier(Modifier<'a>),
    Value(Value<'a>),
    Parenthesised(Parenthesised<'a>),
    Binary(Binary<'a>),
}

impl<'a> Parse<'a> for Expr<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::expr);
        let mut inner = item.into_inner();
        let Some(next) = inner.next() else {
            unreachable!()
        };
        let stmt = match next.as_rule() {
            Rule::modifier => Self::Modifier(Parse::parse(next)),
            Rule::value => Self::Value(Parse::parse(next)),
            Rule::parenthesised => Self::Parenthesised(Parse::parse(next)),
            _ => unreachable!(),
        };
        if let Some(next) = inner.next_as::<Operation>() {
            Self::Binary(Binary {
                left: Box::new(stmt),
                op: next.op,
                right: next.right,
            })
        } else {
            stmt
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Binary<'a> {
    left: Box<Expr<'a>>,
    op: Operator,
    right: Box<Expr<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Modifier<'a> {
    modifier: Ident<'a>,
    args: ModifierArgs<'a>,
}

impl<'a> Parse<'a> for Modifier<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::modifier);
        let mut inner = item.into_inner();
        let Some(modifier) = inner.next_as() else {
            unreachable!()
        };
        let Some(args) = inner.next_as() else {
            unreachable!()
        };
        Self { modifier, args }
    }
}

#[derive(Debug, PartialEq)]
pub struct ModifierArgs<'a> {
    args: Vec<Expr<'a>>,
}

impl<'a> Parse<'a> for ModifierArgs<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::modifier_args);
        let args = item.into_inner().map(Parse::parse).collect::<_>();
        Self { args }
    }
}

#[derive(Debug, PartialEq)]
pub struct Parenthesised<'a> {
    statement: Box<Expr<'a>>,
}

impl<'a> Parse<'a> for Parenthesised<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        let mut inner = item.into_inner();
        let Some(statement) = inner.next_as().map(Box::new) else {
            unreachable!()
        };
        Self { statement }
    }
}

#[derive(Debug, PartialEq)]
struct Operation<'a> {
    op: Operator,
    right: Box<Expr<'a>>,
}

impl<'a> Parse<'a> for Operation<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::operation);
        let mut inner = item.into_inner();
        let Some(op) = inner.next_as() else {
            unreachable!()
        };
        let Some(right) = inner.next_as().map(Box::new) else {
            unreachable!()
        };
        Self { op, right }
    }
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
    And,
    Or,
    Xor,
}

impl<'a> Parse<'a> for Operator {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::operator);
        let inner = item.into_inner().next().unwrap();
        let op = match inner.as_rule() {
            Rule::bool_op_and => Operator::And,
            Rule::bool_op_or => Operator::Or,
            Rule::bool_op_xor => Operator::Xor,
            Rule::compare_op_lt => Operator::Lt,
            Rule::compare_op_le => Operator::Le,
            Rule::compare_op_eq => Operator::Eq,
            Rule::compare_op_ne => Operator::Ne,
            Rule::compare_op_ge => Operator::Ge,
            Rule::compare_op_gt => Operator::Gt,
            _ => unimplemented!(),
        };
        op
    }
}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    String(LitString<'a>),
    Number(&'a str),
    Boolean(bool),
    Ident(Ident<'a>),
}

impl<'a> Parse<'a> for Value<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::value);
        let item = item.into_inner().next().unwrap();
        match item.as_rule() {
            Rule::string => Value::String(LitString::parse(item)),
            Rule::number => Value::Number(item.as_str()),
            Rule::boolean => Value::Boolean(item.as_str() == "true"),
            Rule::ident => Value::Ident(Ident::parse(item)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Block<'a> {
    pub(super) block_name: BlockName<'a>,
    pub(super) content: BlockContent<'a>,
}

impl<'a> Parse<'a> for Block<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::block);
        let mut inner = item.into_inner();
        let block_name = Parse::parse(inner.next().unwrap());
        let content = Parse::parse(inner.next().unwrap());
        Block {
            block_name,
            content,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockName<'a> {
    String(LitString<'a>),
    Ident(Ident<'a>),
}

impl<'a> Parse<'a> for BlockName<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::block_name);
        let inner = item.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ident => BlockName::Ident(Parse::parse(inner)),
            Rule::string => BlockName::String(Parse::parse(inner)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LitString<'a> {
    pub(super) str: &'a str,
}

impl<'a> Parse<'a> for LitString<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::string);
        let inner = item.into_inner().next().unwrap();
        debug_assert_eq!(inner.as_rule(), Rule::inner_string);
        Self {
            str: inner.as_str(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Ident<'a> {
    pub(super) ident: &'a str,
}

impl<'a> Parse<'a> for Ident<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::ident);
        Ident {
            ident: item.as_str(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LiteralText<'a> {
    text: &'a str,
}

impl<'a> Parse<'a> for LiteralText<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::literal_text);
        LiteralText {
            text: item.as_str(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Conditional<'a> {
    cond: Expr<'a>,
    then_case: BlockContent<'a>,
    else_case: Option<BlockContent<'a>>,
}

impl<'a> Parse<'a> for Conditional<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(item.as_rule(), Rule::conditional);
        let mut items = item.into_inner();
        let Some(cond) = items.next_as() else {
            unreachable!()
        };
        let Some(then_case) = items.next_as() else {
            unreachable!()
        };
        let else_case = items.next_as();
        Conditional {
            cond,
            then_case,
            else_case,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WhileLoop<'a> {
    pub cond: Expr<'a>,
    pub block: BlockContent<'a>,
}

impl<'a> Parse<'a> for WhileLoop<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        let mut inner_items = item.into_inner();
        let Some(cond) = inner_items.next_as() else {
            unreachable!()
        };
        let Some(block) = inner_items.next_as() else {
            unreachable!()
        };
        WhileLoop { cond, block }
    }
}

#[derive(Debug, PartialEq)]
pub struct EachLoop<'a> {
    target: Ident<'a>,
    src: Expr<'a>,
    block: BlockContent<'a>,
}

impl<'a> Parse<'a> for EachLoop<'a> {
    fn parse(item: Pair<'a, Rule>) -> Self {
        let mut inner_items = item.into_inner();
        let Some(target) = inner_items.next_as() else {
            unreachable!()
        };
        let Some(src) = inner_items.next_as() else {
            unreachable!()
        };
        let Some(block) = inner_items.next_as() else {
            unreachable!()
        };
        EachLoop { target, src, block }
    }
}

trait ItemFromPair<'a> {
    fn next_as<T>(&mut self) -> Option<T>
    where
        T: Parse<'a>;
}

impl<'a> ItemFromPair<'a> for pest::iterators::Pairs<'a, Rule> {
    fn next_as<T>(&mut self) -> Option<T>
    where
        T: Parse<'a>,
    {
        self.next().map(Parse::parse)
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn parse_string_value() {
        let input = "\"dies ist ein test\"";
        let pair = TemplateParser::parse(Rule::value, input).unwrap();
        let value = Value::parse(pair.into_iter().next().unwrap());
        assert_eq!(
            value,
            Value::String(LitString {
                str: &input[1..(input.len() - 1)]
            })
        )
    }

    #[test]
    fn parse_number_value() {
        fn test<'a>(input: &'a str) -> Value<'a> {
            let pair = TemplateParser::parse(Rule::value, input).unwrap();
            Value::parse(pair.into_iter().next().unwrap())
        }
        assert_eq!(test("1"), Value::Number("1"));
        assert_eq!(test("1.0"), Value::Number("1.0"));
        assert_eq!(test("+1.0"), Value::Number("+1.0"));
        assert_eq!(test("-1.0"), Value::Number("-1.0"));
        assert_eq!(test("1.234"), Value::Number("1.234"));
        assert_eq!(test("1234.0"), Value::Number("1234.0"));
    }

    #[test]
    fn parse_boolean_value() {
        let input = "true";
        let pair = TemplateParser::parse(Rule::value, input).unwrap();
        let value = Value::parse(pair.into_iter().next().unwrap());
        assert_eq!(value, Value::Boolean(true));
        let input = "false";
        let pair = TemplateParser::parse(Rule::value, input).unwrap();
        let value = Value::parse(pair.into_iter().next().unwrap());
        assert_eq!(value, Value::Boolean(false));
    }

    #[test]
    fn parse_ident_value() {
        let input = "MyIdentTest123_";
        let pair = TemplateParser::parse(Rule::value, input).unwrap();
        let value = Value::parse(pair.into_iter().next().unwrap());
        assert_eq!(
            value,
            Value::Ident(Ident {
                ident: "MyIdentTest123_"
            })
        );
    }

    #[test]
    fn parse_ident() {
        let input = "MyIdentTest123_";
        let pair = TemplateParser::parse(Rule::ident, input).unwrap();
        let value = Ident::parse(pair.into_iter().next().unwrap());
        assert_eq!(
            value,
            Ident {
                ident: "MyIdentTest123_"
            }
        );
    }

    #[test]
    fn parse_string_literal() {
        fn test<'a>(input: &'a str) -> LitString<'a> {
            let pair = TemplateParser::parse(Rule::string, input).unwrap();
            LitString::parse(pair.into_iter().next().unwrap())
        }
        assert_eq!(
            test(r#""basic string""#),
            LitString {
                str: "basic string"
            }
        );

        assert_eq!(
            test(r#""my \"basic\" string""#),
            LitString {
                str: r#"my \"basic\" string"#
            }
        );
    }

    #[test]
    fn parse_block_name_ident() {
        let input = "ident_block_name";
        let pair = TemplateParser::parse(Rule::block_name, input).unwrap();
        let value = BlockName::parse(pair.into_iter().next().unwrap());
        assert_eq!(
            value,
            BlockName::Ident(Ident {
                ident: "ident_block_name"
            })
        );
    }
    #[test]
    fn parse_block_name_string() {
        let input = r#""string block name""#;
        let pair = TemplateParser::parse(Rule::block_name, input).unwrap();
        let value = BlockName::parse(pair.into_iter().next().unwrap());
        assert_eq!(
            value,
            BlockName::String(LitString {
                str: "string block name"
            })
        );
    }

    #[test]
    fn parse_operator() {
        let ops = ["&&", "||", "^", "<", "<=", "==", "!=", ">=", ">"];
        let expected = [
            Operator::And,
            Operator::Or,
            Operator::Xor,
            Operator::Lt,
            Operator::Le,
            Operator::Eq,
            Operator::Ne,
            Operator::Ge,
            Operator::Gt,
        ];
        ops.into_iter()
            .zip(expected.into_iter())
            .for_each(|(input, exp)| {
                let pair = TemplateParser::parse(Rule::operator, input).unwrap();
                let value = Operator::parse(pair.into_iter().next().unwrap());
                assert_eq!(value, exp)
            });
    }

    #[test]
    fn parse_full_template() {
        let template_str = r##"{{my_ident}}{%if my_bool_ident && custom_modifier(arg1, arg2, "abc")%}
            {%while a < b%}
                {%a = inc(a)%}
                {{b}}
            {%endwhile%}
        {%endif%}"##;
        let template = super::parse(template_str);
        assert_eq!(
            template,
            Ok(Template {
                block_content: BlockContent {
                    content: vec![
                        BlockContentValue::Instruction(Statement::Print(Print {
                            expr: Expr::Value(Value::Ident(Ident { ident: "my_ident" }))
                        })),
                        BlockContentValue::Instruction(Statement::Conditional(Conditional {
                            cond: Expr::Binary(Binary {
                                left: Box::new(Expr::Value(Value::Ident(Ident {
                                    ident: "my_bool_ident"
                                }))),
                                op: Operator::And,
                                right: Box::new(Expr::Modifier(Modifier {
                                    modifier: Ident {
                                        ident: "custom_modifier"
                                    },
                                    args: ModifierArgs {
                                        args: vec![
                                            Expr::Value(Value::Ident(Ident { ident: "arg1" })),
                                            Expr::Value(Value::Ident(Ident { ident: "arg2" })),
                                            Expr::Value(Value::String(LitString { str: "abc" }))
                                        ]
                                    }
                                }))
                            }),
                            then_case: BlockContent {
                                content: vec![BlockContentValue::Instruction(
                                    Statement::WhileLoop(WhileLoop {
                                        cond: Expr::Binary(Binary {
                                            left: Box::new(Expr::Value(Value::Ident(Ident {
                                                ident: "a"
                                            }))),
                                            op: Operator::Lt,
                                            right: Box::new(Expr::Value(Value::Ident(Ident {
                                                ident: "b"
                                            })))
                                        }),
                                        block: BlockContent {
                                            content: vec![
                                                BlockContentValue::Instruction(Statement::Assign(
                                                    Assign {
                                                        ident: Ident { ident: "a" },
                                                        statement: Expr::Modifier(Modifier {
                                                            modifier: Ident { ident: "inc" },
                                                            args: ModifierArgs {
                                                                args: vec![Expr::Value(
                                                                    Value::Ident(Ident {
                                                                        ident: "a"
                                                                    })
                                                                )]
                                                            }
                                                        })
                                                    }
                                                )),
                                                {
                                                    BlockContentValue::Instruction(
                                                        Statement::Print(Print {
                                                            expr: Expr::Value(Value::Ident(
                                                                Ident { ident: "b" },
                                                            )),
                                                        }),
                                                    )
                                                }
                                            ]
                                        }
                                    })
                                )]
                            },
                            else_case: None
                        }))
                    ]
                }
            })
        )
    }
}
