use pest::{error::LineColLocation, iterators::Pair, Parser};

use crate::{value::Value, Statement, StorageMethod, Template};

#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[grammar = "template.pest"]
pub struct TemplateParser;

pub fn parse(input: String) -> Result<Template, ParseError> {
    let mut compiled_template = Template {
        tpl: Vec::new(),
        tpl_str: input,
    };
    let template = match TemplateParser::parse(Rule::tempalte, &compiled_template.tpl_str) {
        Ok(t) => t,
        Err(e) => match e.line_col {
            LineColLocation::Pos(pos) => {
                error!("parse error {:?}", pos);
                return Err(ParseError::Pos(pos));
            }
            LineColLocation::Span(start, end) => {
                error!("parse error {:?} => {:?}", start, end);
                return Err(ParseError::Span(start, end));
            }
        },
    }
    .next()
    .unwrap();
    compiled_template.tpl = template
        .into_inner()
        .filter_map(parse_template_item)
        .collect::<Vec<_>>();
    Ok(compiled_template)
}

fn parse_template_item(item: Pair<Rule>) -> Option<Statement> {
    match item.as_rule() {
        Rule::literal => Some(Statement::Literal(item.as_str())),
        Rule::calculated => Some(parse_calculated(item)),
        Rule::EOI => None,
        _ => unreachable!(),
    }
}

fn parse_calculated(calculated: Pair<Rule>) -> Statement {
    let mut inner = calculated.into_inner();
    let value = parse_argument(inner.next().unwrap());
    let modifiers = inner.into_iter().map(parse_modifier).collect::<Vec<_>>();
    Statement::Calculated { value, modifiers }
}

fn parse_modifier(item: Pair<Rule>) -> (*const str, Vec<StorageMethod>) {
    let mut items = item.into_inner();
    let name = items.next().unwrap().as_str();
    (name, items.map(parse_argument).collect())
}

fn parse_argument(item: Pair<Rule>) -> StorageMethod {
    let item = item.into_inner().next().unwrap();
    match item.as_rule() {
        Rule::identifyer => StorageMethod::Variable(item.as_str()),
        Rule::number => StorageMethod::Const(Value::Number(item.as_str().parse().unwrap())),
        Rule::string => StorageMethod::Const(Value::String(
            item.into_inner().next().unwrap().as_str().to_owned(),
        )),
        Rule::boolean => {
            let value = match item.as_str() {
                "true" => true,
                "false" => false,
                _ => panic!(),
            };
            StorageMethod::Const(Value::Bool(value))
        }
        _ => panic!(),
    }
}

#[derive(Debug)]
pub enum ParseError {
    Pos((usize, usize)),
    Span((usize, usize), (usize, usize)),
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn parse_template_item_literal() {
        let template = String::from("test literal");
        let item = TemplateParser::parse(Rule::literal, &template);
        assert!(item.is_ok());
        let item = item.unwrap().next();
        assert!(item.is_some());
        let item = item.unwrap();
        let statement = parse_template_item(item).unwrap();
        assert_eq!(statement, Statement::Literal("test literal"))
    }

    #[test]
    fn parse_template_single_literal() {
        let template = String::from("test literal");
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl: vec![Statement::Literal("test literal")],
                tpl_str: String::from("test literal")
            }
        );
    }

    #[test]
    fn parse_template_item_calculated() {
        let template = String::from("{var}");
        let item = TemplateParser::parse(Rule::calculated, &template);
        assert!(item.is_ok());
        let item = item.unwrap().next();
        assert!(item.is_some());
        let item = item.unwrap();
        let statement = parse_calculated(item);
        assert_eq!(
            statement,
            Statement::Calculated {
                modifiers: Vec::new(),
                value: StorageMethod::Variable("var"),
            }
        )
    }

    #[test]
    fn parse_template_single_computed() {
        let template = String::from("{var}");
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl: vec![Statement::Calculated {
                    modifiers: vec![],
                    value: StorageMethod::Variable("var"),
                }],
                tpl_str: String::from("{var}")
            }
        );
    }

    #[test]
    fn parse_template_single_computed_modifier() {
        let template = String::from("{var|modifier}");
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from("{var|modifier}"),
                tpl: vec![Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![("modifier", vec![])]
                }]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_multiple_modifier() {
        let template = String::from("{var|modifier1|modifier2}");
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from("{var|modifier1|modifier2}"),
                tpl: vec![Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![("modifier1", vec![]), ("modifier2", vec![])]
                }]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_modifier_var_param() {
        let template = String::from("{var|modifier:var2}");
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from("{var|modifier:var2}"),
                tpl: vec![Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![("modifier", vec![StorageMethod::Variable("var2")])]
                }]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_modifier_number_param() {
        let template = String::from(r#"{var|modifier:-32.09}"#);
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from(r#"{var|modifier:-32.09}"#),
                tpl: vec![Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![(
                        "modifier",
                        vec![StorageMethod::Const(Value::Number(-32.09))]
                    )]
                }]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_literal_before_modifier() {
        let template = String::from(r#"{10|modifier:-32.09}"#);
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from(r#"{10|modifier:-32.09}"#),
                tpl: vec![Statement::Calculated {
                    value: StorageMethod::Const(Value::Number(10.0)),
                    modifiers: vec![(
                        "modifier",
                        vec![StorageMethod::Const(Value::Number(-32.09))]
                    )]
                }]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_modifier_multiple_args() {
        let template = String::from(r#"{var|modifier:-32.09:"argument":var2:true}"#);
        let template = parse(template);
        assert!(template.is_ok(), "{:#?}", template);
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from(r#"{var|modifier:-32.09:"argument":var2:true}"#),
                tpl: vec![Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![(
                        "modifier",
                        vec![
                            StorageMethod::Const(Value::Number(-32.09)),
                            StorageMethod::Const(Value::String(String::from("argument"))),
                            StorageMethod::Variable("var2"),
                            StorageMethod::Const(Value::Bool(true))
                        ]
                    )]
                }]
            }
        )
    }
}

#[cfg(test)]
mod pest_tests {

    use super::*;

    const NUMBER_CASES: [&str; 5] = ["42", "42.0", "0.815", "-0.815", "+0.815"];

    const IDENTIFYER_CASES: [&str; 3] = ["onlylowercase", "camelCase", "snail_case"];

    const INNER_STRING_CASES: [&str; 4] = [
        "Hello world",
        "123Hello World!",
        "!Hello World",
        r#"Hello\"World"#,
    ];

    macro_rules! test_cases {
        ($list: ident, $cases_var: ident, $format: tt) => {
            let $cases_var = $list
                .iter()
                .map(|c| format!($format, c))
                .collect::<Vec<_>>();
            let $cases_var = $cases_var.iter().map(String::as_str).collect::<Vec<_>>();
        };
    }

    #[test]
    fn number() {
        test_cases(&NUMBER_CASES, Rule::number)
    }

    #[test]
    fn identifyer() {
        test_cases(&IDENTIFYER_CASES, Rule::identifyer)
    }

    #[test]
    fn argument() {
        test_cases!(NUMBER_CASES, number_cases, ":{}");
        test_cases(&number_cases, Rule::argument);

        test_cases!(IDENTIFYER_CASES, ident_cases, ":{}");
        test_cases(&ident_cases, Rule::argument)
    }

    #[test]
    fn inner_string() {
        test_cases(&INNER_STRING_CASES, Rule::inner_string)
    }

    #[test]
    fn string() {
        test_cases!(INNER_STRING_CASES, cases, r#""{}""#);
        test_cases(&cases, Rule::string)
    }

    #[test]
    fn string_before_modifier() {
        test_cases(&[r#"{"test"|modifier:arg}"#], Rule::calculated)
    }

    fn test_cases(cases: &[&str], rule: Rule) {
        cases.iter().for_each(|input| {
            let parsed = TemplateParser::parse(rule, input);
            assert!(parsed.is_ok(), "{:#?}", parsed);
            let parsed = parsed.unwrap().next();
            assert!(parsed.is_some(), "{:#?}", parsed);
            let identifyer = parsed.unwrap();
            assert_eq!(identifyer.as_str(), *input);
        })
    }
}

#[cfg(test)]
mod legacy_tests {

    use super::*;

    #[test]
    fn simple_compile() {
        let tpl = parse("Simple template string".to_owned()).unwrap();
        assert_eq!(
            vec![Statement::Literal("Simple template string" as *const _)],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value() {
        let tpl = parse("Simple more {var} template {foo}".to_owned()).unwrap();
        assert_eq!(
            vec![
                Statement::Literal("Simple more " as *const _),
                Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![]
                },
                Statement::Literal(" template " as *const _),
                Statement::Calculated {
                    value: StorageMethod::Variable("foo"),
                    modifiers: vec![]
                }
            ],
            tpl.tpl
        )
    }

    #[test]
    fn variable_value_simple_modifier() {
        let tpl = parse("Simple {var|test} template".to_owned()).unwrap();
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![("test" as *const _, vec![])]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value_modifier_string_value() {
        let tpl = parse(r#"Simple {var|test:"test value"} template"#.to_owned()).unwrap();
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![(
                        "test" as *const _,
                        vec![StorageMethod::Const(Value::String(
                            "test value".to_string()
                        ))]
                    )]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value_modifier_num_value() {
        let tpl = parse(r#"Simple {var|test:42} template"#.to_owned()).unwrap();
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![(
                        "test" as *const _,
                        vec![StorageMethod::Const(Value::Number(42_f64))]
                    )]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value_modifier_var_value() {
        let tpl = parse(r#"Simple {var|test:foobar} template"#.to_owned()).unwrap();
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    value: StorageMethod::Variable("var"),
                    modifiers: vec![("test" as *const _, vec![StorageMethod::Variable("foobar")])]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }
}
