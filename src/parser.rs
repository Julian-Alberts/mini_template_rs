use pest::{error::LineColLocation, iterators::Pair, Parser};

use crate::{
    template::{
        AndCondition, CalculatedValue, CompareCondition, CompareOperator, Condition, Conditional,
        OrCondition, Statement, StorageMethod,
    },
    value::Value,
    Template,
};

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
                return Err(ParseError::Pos(pos));
            }
            LineColLocation::Span(start, end) => {
                return Err(ParseError::Span(start, end));
            }
        },
    }
    .next()
    .unwrap();
    compiled_template.tpl = template
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .filter_map(parse_template_content)
        .collect::<Vec<_>>();
    Ok(compiled_template)
}

fn parse_template_content(item: Pair<Rule>) -> Option<Statement> {
    match item.as_rule() {
        Rule::text => Some(Statement::Literal(item.as_str())),
        Rule::calculated => Some(parse_calculated(item)),
        Rule::conditional => Some(parse_conditional(item)),
        Rule::EOI => None,
        _ => unreachable!("Unexpected rule {:#?}", item.as_rule()),
    }
}

fn parse_conditional(conditional: Pair<Rule>) -> Statement {
    assert_eq!(conditional.as_rule(), Rule::conditional);
    let mut conditional = conditional.into_inner();

    let condition = conditional.next().unwrap();
    let condition = parse_condition(condition);
    let then_case = conditional
        .next()
        .unwrap()
        .into_inner()
        .filter_map(parse_template_content)
        .collect::<Vec<_>>();
    let else_case = conditional.next().map(|else_case| {
        else_case
            .into_inner()
            .filter_map(parse_template_content)
            .collect()
    });

    Statement::Condition(Conditional {
        condition,
        then_case,
        else_case,
    })
}

fn parse_condition(condition: Pair<Rule>) -> Condition {
    assert_eq!(condition.as_rule(), Rule::condition);
    let mut inner = condition.into_inner();

    let mut current_and = None;
    let mut current_or = Vec::new();
    let mut prev_operator = None;

    // At some point no more operators will be found and the function returns
    while let Some(c) = inner.next() {
        let c = match c.as_rule() {
            Rule::condition => parse_condition(c),
            Rule::compare_condition => Condition::Compare(parse_compare_condition(c)),
            Rule::calculated_value => Condition::CalculatedValue(parse_calculated_value(c)),
            _ => unreachable!(),
        };

        if let Some(operator) = inner.next() {
            match (operator.as_rule(), prev_operator) {
                (Rule::and_operator, _) => current_and.get_or_insert(Vec::default()).push(c),
                (Rule::or_operator, Some(Rule::and_operator)) => {
                    current_and.get_or_insert(Vec::default()).push(c);
                    current_or.push(Condition::And(AndCondition::new(
                        current_and.take().unwrap(),
                    )))
                }
                (Rule::or_operator, _) => current_or.push(c),
                _ => unreachable!(),
            }
            prev_operator = Some(operator.as_rule());
        } else {
            match prev_operator {
                Some(Rule::and_operator) => {
                    current_and.get_or_insert(Vec::default()).push(c);
                    let and = Condition::And(AndCondition::new(current_and.take().unwrap()));
                    return if !current_or.is_empty() {
                        current_or.push(and);
                        Condition::Or(OrCondition::new(current_or))
                    } else {
                        and
                    };
                }
                Some(Rule::or_operator) => {
                    current_or.push(c);
                    return Condition::Or(OrCondition::new(current_or));
                }
                None => return c,
                _ => unreachable!(),
            }
        }
    }
    unreachable!()
}

fn parse_calculated(calculated: Pair<Rule>) -> Statement {
    assert_eq!(calculated.as_rule(), Rule::calculated);
    let inner = calculated.into_inner().next().unwrap();
    Statement::Calculated(parse_calculated_value(inner))
}

fn parse_compare_condition(compare_condition: Pair<Rule>) -> CompareCondition {
    assert_eq!(compare_condition.as_rule(), Rule::compare_condition);
    let mut inner = compare_condition.into_inner();
    let calc_val_l = parse_calculated_value(inner.next().unwrap());
    let operator = parse_compare_operator(inner.next().unwrap());
    let calc_val_r = parse_calculated_value(inner.next().unwrap());
    CompareCondition {
        left: calc_val_l,
        operator,
        right: calc_val_r,
    }
}

fn parse_compare_operator(compare_operator: Pair<Rule>) -> CompareOperator {
    assert_eq!(compare_operator.as_rule(), Rule::compare_operator);
    let inner = compare_operator.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::eq_operator => CompareOperator::EQ,
        Rule::ne_operator => CompareOperator::NE,
        Rule::lt_operator => CompareOperator::LT,
        Rule::le_operator => CompareOperator::LE,
        Rule::gt_operator => CompareOperator::GT,
        Rule::ge_operator => CompareOperator::GE,
        _ => unreachable!("Unknown compare operator: {}", inner.as_str()),
    }
}

fn parse_calculated_value(calculated_value: Pair<Rule>) -> CalculatedValue {
    assert_eq!(calculated_value.as_rule(), Rule::calculated_value);
    let mut inner = calculated_value.into_inner();
    let value = parse_value(inner.next().unwrap());
    let modifiers = inner.into_iter().map(parse_modifier).collect::<Vec<_>>();
    CalculatedValue::new(value, modifiers)
}

fn parse_modifier(item: Pair<Rule>) -> (*const str, Vec<StorageMethod>) {
    assert_eq!(item.as_rule(), Rule::modifier);
    let mut items = item.into_inner();
    let name = items.next().unwrap().as_str();
    (name, items.map(parse_argument).collect())
}

fn parse_argument(argument: Pair<Rule>) -> StorageMethod {
    assert_eq!(argument.as_rule(), Rule::argument);
    let value = argument.into_inner().next().unwrap();
    parse_value(value)
}

fn parse_value(value: Pair<Rule>) -> StorageMethod {
    assert_eq!(value.as_rule(), Rule::value);
    let value = value.into_inner().next().unwrap();
    match value.as_rule() {
        Rule::identifyer => StorageMethod::Variable(value.as_str()),
        Rule::number => StorageMethod::Const(Value::Number(value.as_str().parse().unwrap())),
        Rule::string => StorageMethod::Const(Value::String(
            value.into_inner().next().unwrap().as_str().to_owned(),
        )),
        Rule::boolean => {
            let value = match value.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!("boolean must be true or false"),
            };
            StorageMethod::Const(Value::Bool(value))
        }
        _ => unreachable!("Unexpected value {:#?}", value),
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
        let item = TemplateParser::parse(Rule::text, &template);
        assert!(item.is_ok());
        let item = item.unwrap().next();
        assert!(item.is_some());
        let item = item.unwrap();
        let statement = parse_template_content(item).unwrap();
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
            Statement::Calculated(CalculatedValue::new(
                StorageMethod::Variable("var"),
                Vec::new()
            ))
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![],
                ))],
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![("modifier", vec![])]
                ))]
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![("modifier1", vec![]), ("modifier2", vec![])]
                ))]
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![("modifier", vec![StorageMethod::Variable("var2")])]
                ))]
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![(
                        "modifier",
                        vec![StorageMethod::Const(Value::Number(-32.09))]
                    )]
                ))]
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Const(Value::Number(10.0)),
                    vec![(
                        "modifier",
                        vec![StorageMethod::Const(Value::Number(-32.09))]
                    )]
                ))]
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
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![(
                        "modifier",
                        vec![
                            StorageMethod::Const(Value::Number(-32.09)),
                            StorageMethod::Const(Value::String(String::from("argument"))),
                            StorageMethod::Variable("var2"),
                            StorageMethod::Const(Value::Bool(true))
                        ]
                    )]
                ))]
            }
        )
    }

    #[test]
    fn parse_template_multi_line() {
        let template = String::from("{var|modifier}\n{10|modifier:-32.09}");
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from("{var|modifier}\n{10|modifier:-32.09}"),
                tpl: vec![
                    Statement::Calculated(CalculatedValue::new(
                        StorageMethod::Variable("var"),
                        vec![("modifier", vec![])]
                    )),
                    Statement::Literal("\n"),
                    Statement::Calculated(CalculatedValue::new(
                        StorageMethod::Const(Value::Number(10.0)),
                        vec![(
                            "modifier",
                            vec![StorageMethod::Const(Value::Number(-32.09))]
                        )]
                    ))
                ]
            }
        )
    }

    mod conditional {
        use super::*;

        #[test]
        fn parse_simple() {
            let template = "{if i < 10}HI{endif}";
            let conditional = TemplateParser::parse(Rule::conditional, template)
                .unwrap()
                .next()
                .unwrap();
            let conditional_statement = parse_conditional(conditional);
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::Compare(CompareCondition {
                            left: CalculatedValue::new(StorageMethod::Variable("i"), vec![]),
                            operator: CompareOperator::LT,
                            right: CalculatedValue::new(
                                StorageMethod::Const(Value::Number(10.)),
                                vec![]
                            )
                        }),
                        then_case: vec![Statement::Literal("HI")],
                        else_case: None
                    }
                )
            } else {
                panic!("Unexpected statement")
            }
        }

        #[test]
        fn parse_complex_condition() {
            let template = "{if (var1 || var2) && var3}HI{endif}";
            let conditional = TemplateParser::parse(Rule::conditional, template)
                .unwrap()
                .next()
                .unwrap();
            let conditional_statement = parse_conditional(conditional);
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::and(vec![
                            Condition::or(vec![
                                Condition::CalculatedValue(CalculatedValue::new(
                                    StorageMethod::Variable("var1"),
                                    vec![]
                                )),
                                Condition::CalculatedValue(CalculatedValue::new(
                                    StorageMethod::Variable("var2"),
                                    vec![]
                                ))
                            ]),
                            Condition::CalculatedValue(CalculatedValue::new(
                                StorageMethod::Variable("var3"),
                                vec![]
                            ))
                        ]),
                        then_case: vec![Statement::Literal("HI")],
                        else_case: None
                    }
                )
            } else {
                panic!("Unexpected statement")
            }
        }

        #[test]
        fn parse_else() {
            let template = "{if i < 10}HI{else}TEST{endif}";
            let conditional = TemplateParser::parse(Rule::conditional, template)
                .unwrap()
                .next()
                .unwrap();
            let conditional_statement = parse_conditional(conditional);
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::Compare(CompareCondition {
                            left: CalculatedValue::new(StorageMethod::Variable("i"), vec![]),
                            operator: CompareOperator::LT,
                            right: CalculatedValue::new(
                                StorageMethod::Const(Value::Number(10.)),
                                vec![]
                            )
                        }),
                        then_case: vec![Statement::Literal("HI")],
                        else_case: Some(vec![Statement::Literal("TEST")])
                    }
                )
            } else {
                panic!("Unexpected statement")
            }
        }

        #[test]
        fn parse_mutiple() {
            let template = "{if i < 10}HI{else}{if n == \"TEST\"}HI2{else}TEST{endif}{endif}";
            let conditional = TemplateParser::parse(Rule::conditional, template)
                .unwrap()
                .next()
                .unwrap();
            let conditional_statement = parse_conditional(conditional);
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::Compare(CompareCondition {
                            left: CalculatedValue::new(StorageMethod::Variable("i"), vec![]),
                            operator: CompareOperator::LT,
                            right: CalculatedValue::new(
                                StorageMethod::Const(Value::Number(10.)),
                                vec![]
                            )
                        }),
                        then_case: vec![Statement::Literal("HI")],
                        else_case: Some(vec![Statement::Condition(Conditional {
                            condition: Condition::Compare(CompareCondition {
                                left: CalculatedValue::new(StorageMethod::Variable("n"), vec![]),
                                operator: CompareOperator::EQ,
                                right: CalculatedValue::new(
                                    StorageMethod::Const(Value::String("TEST".to_owned())),
                                    vec![]
                                )
                            }),
                            then_case: vec![Statement::Literal("HI2")],
                            else_case: Some(vec![Statement::Literal("TEST")])
                        })])
                    }
                )
            } else {
                panic!("Unexpected statement")
            }
        }
    }

    mod condition {
        use crate::template::AndCondition;

        use super::*;

        #[test]
        fn parse() {
            let template = "bar";
            let condition = TemplateParser::parse(Rule::condition, template)
                .unwrap()
                .next()
                .unwrap();
            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::CalculatedValue(CalculatedValue::new(
                    StorageMethod::Variable("bar"),
                    vec![]
                )),
            );
        }

        #[test]
        fn parse_eq() {
            let template = "bar == 10";
            let condition = TemplateParser::parse(Rule::condition, template)
                .unwrap()
                .next()
                .unwrap();
            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::Compare(CompareCondition {
                    left: CalculatedValue::new(StorageMethod::Variable("bar"), vec![]),
                    operator: CompareOperator::EQ,
                    right: CalculatedValue::new(StorageMethod::Const(Value::Number(10.)), vec![])
                })
            );
        }

        #[test]
        fn parse_2() {
            let template = "(bar == 10)";
            let condition = TemplateParser::parse(Rule::condition, template)
                .unwrap()
                .next()
                .unwrap();
            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::Compare(CompareCondition {
                    left: CalculatedValue::new(StorageMethod::Variable("bar"), vec![]),
                    operator: CompareOperator::EQ,
                    right: CalculatedValue::new(StorageMethod::Const(Value::Number(10.)), vec![])
                })
            );
        }

        #[test]
        fn parse_complex() {
            let tpl = "var1 || var2 && var3";
            let condition = TemplateParser::parse(Rule::condition, tpl)
                .unwrap()
                .next()
                .unwrap();
            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::Or(OrCondition::new(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var1"),
                        vec![]
                    )),
                    Condition::And(AndCondition::new(vec![
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable("var2"),
                            vec![]
                        )),
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable("var3"),
                            vec![]
                        ))
                    ]))
                ]))
            )
        }

        #[test]
        fn parse_complex2() {
            let tpl = "(var1 || var2)";
            let condition = TemplateParser::parse(Rule::condition, tpl)
                .unwrap()
                .next()
                .unwrap();

            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::Or(OrCondition::new(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var1"),
                        vec![]
                    )),
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var2"),
                        vec![]
                    ))
                ])),
            )
        }

        #[test]
        fn parse_complex3() {
            let tpl = "(var1 && var2)";
            let condition = TemplateParser::parse(Rule::condition, tpl)
                .unwrap()
                .next()
                .unwrap();

            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::And(AndCondition::new(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var1"),
                        vec![]
                    )),
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var2"),
                        vec![]
                    ))
                ])),
            )
        }

        #[test]
        fn parse_complex4() {
            let tpl = "((var1 || var2) && var3)";
            let condition = TemplateParser::parse(Rule::condition, tpl)
                .unwrap()
                .next()
                .unwrap();

            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::And(AndCondition::new(vec![
                    Condition::Or(OrCondition::new(vec![
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable("var1"),
                            vec![]
                        )),
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable("var2"),
                            vec![]
                        ))
                    ])),
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var3"),
                        vec![]
                    ))
                ]))
            )
        }

        #[test]
        fn parse_complex5() {
            let tpl = "(var1 || (var2 && var3))";
            let condition = TemplateParser::parse(Rule::condition, tpl)
                .unwrap()
                .next()
                .unwrap();

            let condition = super::parse_condition(condition);
            assert_eq!(
                condition,
                Condition::or(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable("var1"),
                        vec![]
                    )),
                    Condition::and(vec![
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable("var2"),
                            vec![]
                        )),
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable("var3"),
                            vec![]
                        ))
                    ])
                ])
            )
        }
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

    #[test]
    fn test_condition() {
        test_cases(
            &[
                "bar",
                "(bar)",
                "var1 == var2",
                "(var1 == var2)",
                "var1 == var2 || var5 == var5",
                "var1 == var2 || (var5 == var5)",
                "var1 == var2 || var5 == var5 && var1 == \"foo\"",
                "var1 == var2 || (var5 == var5 && var1 == \"foo\")",
            ],
            Rule::condition,
        );
    }

    #[test]
    fn test_conditional() {
        test_cases(
            &[
                "{if i < 10}HI{endif}",
                "{if i < 10}HI{else}TEST{endif}",
                "{if i < 10}HI{else}{if i < 10}HI{else}TEST{endif}{endif}",
                "{if i}HI{endif}",
            ],
            Rule::conditional,
        );
    }

    #[test]
    fn test_template() {
        test_cases(
            &[
                "Hello world",
                r#"{"test"|modifier:arg}"#,
                "{if i < 10} HI {endif}",
                "{if i < 10}HI{else}TEST{endif}",
                "{if i < 10}HI{else}{if i < 10}HI{else}TEST{endif}{endif}",
                "{if i}HI{endif}",
            ],
            Rule::tempalte,
        );
    }

    #[test]
    fn test_template_content() {
        test_cases(
            &[
                "Hello world",
                r#"{"test"|modifier:arg}"#,
                "{if i < 10} HI {endif}",
                "{if i < 10}HI{else}TEST{endif}",
                "{if i < 10}HI{else}{if i < 10}HI{else}TEST{endif}{endif}",
                "{if i}HI{endif}",
            ],
            Rule::template_content,
        )
    }

    fn test_cases(cases: &[&str], rule: Rule) {
        cases.iter().for_each(|input| {
            let parsed = TemplateParser::parse(rule, input);
            assert!(parsed.is_ok(), "Failed to parse \"{input}\"\n{parsed:#?}");
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
                Statement::Calculated(CalculatedValue::new(StorageMethod::Variable("var"), vec![])),
                Statement::Literal(" template " as *const _),
                Statement::Calculated(CalculatedValue::new(StorageMethod::Variable("foo"), vec![]))
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
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![("test" as *const _, vec![])]
                )),
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
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![(
                        "test" as *const _,
                        vec![StorageMethod::Const(Value::String(
                            "test value".to_string()
                        ))]
                    )]
                )),
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
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![(
                        "test" as *const _,
                        vec![StorageMethod::Const(Value::Number(42_f64))]
                    )]
                )),
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
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable("var"),
                    vec![("test" as *const _, vec![StorageMethod::Variable("foobar")])]
                )),
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }
}
