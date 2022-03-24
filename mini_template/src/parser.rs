use std::fmt::{Display, Formatter};
use pest::{error::LineColLocation, iterators::Pair, Parser};

#[cfg(feature = "condition")]
use crate::template::condition::{
    AndCondition, CompareCondition, CompareOperator, Condition, OrCondition,
};
#[cfg(feature = "assign")]
use crate::template::Assign;
#[cfg(feature = "conditional")]
use crate::template::Conditional;
#[cfg(feature = "loop")]
use crate::template::Loop;
use crate::template::Modifier;
use crate::value::ident::{Ident, IdentPart};
use crate::{template::{CalculatedValue, Statement}, value::{StorageMethod, Value}, Template, util};

#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[grammar = "template.pest"]
pub struct TemplateParser;

pub fn parse(input: String) -> Result<Template, ParseError> {
    let mut compiled_template = Template {
        tpl: Vec::new(),
        tpl_str: input,
    };
    let template = match TemplateParser::parse(Rule::template, &compiled_template.tpl_str) {
        Ok(t) => t,
        Err(e) => match e.line_col {
            LineColLocation::Pos(pos) => {
                return Err(ParseError::Syntax(pos, pos, compiled_template.tpl_str));
            }
            LineColLocation::Span(start, end) => {
                return Err(ParseError::Syntax(start, end, compiled_template.tpl_str));
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
        .collect::<Result<Vec<_>, _>>()?;
    Ok(compiled_template)
}

fn parse_template_content(item: Pair<Rule>) -> Option<Result<Statement, ParseError>> {
    match item.as_rule() {
        Rule::text => Some(Ok(Statement::Literal(item.as_str()))),
        Rule::calculated => Some(parse_calculated(item)),
        #[cfg(feature = "conditional")]
        Rule::conditional => Some(parse_conditional(item)),
        #[cfg(not(feature = "conditional"))]
        Rule::conditional => Some(Err(ParseError::DisabledFeature(
            UnsupportedFeature::Conditional,
        ))),
        #[cfg(feature = "assign")]
        Rule::assign => {
            let assign = match parse_assign(item) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            };
            Some(Ok(Statement::Assign(assign)))
        }
        #[cfg(not(feature = "assign"))]
        Rule::assign => Some(Err(ParseError::DisabledFeature(UnsupportedFeature::Assign))),
        #[cfg(feature = "loop")]
        Rule::while_loop => match parse_loop(item) {
            Ok(l) => Some(Ok(Statement::Loop(l))),
            Err(e) => Some(Err(e)),
        },
        #[cfg(not(feature = "loop"))]
        Rule::while_loop => Some(Err(ParseError::DisabledFeature(UnsupportedFeature::Loop))),
        Rule::EOI => None,
        _ => unreachable!("Unexpected rule {:#?}", item.as_rule()),
    }
}

#[cfg(feature = "conditional")]
fn parse_conditional(conditional: Pair<Rule>) -> Result<Statement, ParseError> {
    assert_eq!(conditional.as_rule(), Rule::conditional);
    let mut conditional = conditional.into_inner();

    let condition = conditional.next().unwrap();
    let condition = parse_condition(condition)?;
    let then_case = conditional
        .next()
        .unwrap()
        .into_inner()
        .filter_map(parse_template_content)
        .collect::<Result<Vec<_>, _>>()?;
    let else_case = conditional.next().map(|else_case| {
        else_case
            .into_inner()
            .filter_map(parse_template_content)
            .collect::<Result<Vec<_>, _>>()
    });

    let else_case = if let Some(e) = else_case {
        Some(e?)
    } else {
        None
    };

    Ok(Statement::Condition(Conditional {
        condition,
        then_case,
        else_case,
    }))
}

#[cfg(feature = "condition")]
fn parse_condition(condition: Pair<Rule>) -> Result<Condition, ParseError> {
    assert_eq!(condition.as_rule(), Rule::condition);
    let mut inner = condition.into_inner();

    let mut current_and = None;
    let mut current_or = Vec::new();
    let mut prev_operator = None;

    // At some point no more operators will be found and the function returns
    while let Some(c) = inner.next() {
        let c = match c.as_rule() {
            Rule::condition => parse_condition(c)?,
            Rule::compare_condition => Condition::Compare(parse_compare_condition(c)?),
            Rule::calculated_value => Condition::CalculatedValue(parse_calculated_value(c)?),
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
                        Ok(Condition::Or(OrCondition::new(current_or)))
                    } else {
                        Ok(and)
                    };
                }
                Some(Rule::or_operator) => {
                    current_or.push(c);
                    return Ok(Condition::Or(OrCondition::new(current_or)));
                }
                None => return Ok(c),
                _ => unreachable!(),
            }
        }
    }
    unreachable!()
}

fn parse_calculated(calculated: Pair<Rule>) -> Result<Statement, ParseError> {
    assert_eq!(calculated.as_rule(), Rule::calculated);
    let inner = calculated.into_inner().next().unwrap();
    Ok(Statement::Calculated(parse_calculated_value(inner)?))
}

#[cfg(feature = "condition")]
fn parse_compare_condition(compare_condition: Pair<Rule>) -> Result<CompareCondition, ParseError> {
    assert_eq!(compare_condition.as_rule(), Rule::compare_condition);
    let mut inner = compare_condition.into_inner();
    let calc_val_l = parse_calculated_value(inner.next().unwrap())?;
    let operator = parse_compare_operator(inner.next().unwrap());
    let calc_val_r = parse_calculated_value(inner.next().unwrap())?;
    Ok(CompareCondition {
        left: calc_val_l,
        operator,
        right: calc_val_r,
    })
}

#[cfg(feature = "condition")]
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

fn parse_calculated_value(calculated_value: Pair<Rule>) -> Result<CalculatedValue, ParseError> {
    assert_eq!(calculated_value.as_rule(), Rule::calculated_value);
    let mut inner = calculated_value.into_inner();
    let value = parse_value(inner.next().unwrap())?;
    let modifiers = inner
        .into_iter()
        .map(parse_modifier)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CalculatedValue::new(value, modifiers))
}

fn parse_modifier(item: Pair<Rule>) -> Result<Modifier, ParseError> {
    assert_eq!(item.as_rule(), Rule::modifier);
    let span = item.as_span().into();
    let mut items = item.into_inner();
    let name = items.next().unwrap().as_str();
    let args = items.map(parse_argument).collect::<Result<_, _>>()?;

    Ok(Modifier { name, args, span })
}

fn parse_argument(argument: Pair<Rule>) -> Result<StorageMethod, ParseError> {
    assert_eq!(argument.as_rule(), Rule::argument);
    let value = argument.into_inner().next().unwrap();
    parse_value(value)
}

fn parse_value(value: Pair<Rule>) -> Result<StorageMethod, ParseError> {
    assert_eq!(value.as_rule(), Rule::value);
    let value = value.into_inner().next().unwrap();
    let value = match value.as_rule() {
        Rule::identifier => StorageMethod::Variable(parse_identifier(value)?),
        Rule::number => StorageMethod::Const(Value::Number(value.as_str().parse().unwrap())),
        Rule::string => StorageMethod::Const(Value::String(
            value
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .replace("\\\"", "\""),
        )),
        Rule::boolean => {
            let value = match value.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!("boolean must be true or false"),
            };
            StorageMethod::Const(Value::Bool(value))
        }
        Rule::null_literal => StorageMethod::Const(Value::Null),
        _ => unreachable!("Unexpected value {:#?}", value),
    };
    Ok(value)
}

#[cfg(feature = "assign")]
fn parse_assign(assign: Pair<Rule>) -> Result<Assign, ParseError> {
    assert_eq!(assign.as_rule(), Rule::assign);
    let mut inner = assign.into_inner();
    let ident = inner.next().unwrap();
    assert_eq!(ident.as_rule(), Rule::identifier);
    let ident = parse_identifier(ident)?;
    let calc_val = parse_calculated_value(inner.next().unwrap())?;
    Ok(Assign::new(ident, calc_val))
}

#[cfg(feature = "loop")]
fn parse_loop(l: Pair<Rule>) -> Result<Loop, ParseError> {
    assert_eq!(l.as_rule(), Rule::while_loop);
    let mut inner = l.into_inner();
    let condition = parse_condition(inner.next().unwrap())?;
    let template = inner
        .next()
        .unwrap()
        .into_inner()
        .filter_map(parse_template_content)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Loop::new(condition, template))
}

fn parse_identifier(ident: Pair<Rule>) -> Result<Ident, ParseError> {
    assert_eq!(ident.as_rule(), Rule::identifier);
    let ident_span = ident.as_span().into();
    let mut inner = ident.into_inner().rev();

    fn ident_to_part(ident: Pair<Rule>) -> Result<IdentPart, ParseError> {
        let ident = match ident.as_rule() {
            Rule::ident_static => IdentPart::Static(ident.as_str()),
            Rule::ident_dynamic => {
                IdentPart::Dynamic(parse_value(ident.into_inner().next().unwrap())?)
            }
            _ => unreachable!(),
        };
        Ok(ident)
    }

    let ident = inner.next().unwrap();
    let ident_part = ident_to_part(ident)?;

    let ident = inner.try_fold(
        Ident {
            part: Box::new(ident_part),
            next: None,
            span: ident_span,
        },
        |next, ident| {
            let ident_span = ident.as_span().into();
            let ident_part = ident_to_part(ident)?;
            Ok(Ident {
                part: Box::new(ident_part),
                next: Some(Box::new(next)),
                span: ident_span,
            })
        },
    )?;

    #[cfg(feature = "dynamic_global_access")]
    return Ok(ident);

    #[cfg(not(feature = "dynamic_global_access"))]
    if let IdentPart::Dynamic(_) = &*ident.part {
        Err(ParseError::DisabledFeature(
            UnsupportedFeature::DynamicGlobalAccess,
        ))
    } else {
        Ok(ident)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Syntax((usize, usize), (usize, usize), String),
    DisabledFeature(UnsupportedFeature),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Syntax(start, end, template) => util::mark_between_points(*start, *end, template, f),
            ParseError::DisabledFeature(u) => write!(f, "{:#?}", u)
        }
    }

}

#[derive(Debug, PartialEq)]
pub enum UnsupportedFeature {
    #[cfg(not(feature = "assign"))]
    Assign,
    #[cfg(not(feature = "conditional"))]
    Conditional,
    #[cfg(not(feature = "loop"))]
    Loop,
    #[cfg(not(feature = "dynamic_global_access"))]
    DynamicGlobalAccess,
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
        let statement = parse_template_content(item).unwrap().unwrap();
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
        let statement = parse_calculated(item).unwrap();
        assert_eq!(
            statement,
            Statement::Calculated(CalculatedValue::new(
                StorageMethod::Variable(Ident::new_static("var")),
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
                    StorageMethod::Variable(Ident::new_static("var")),
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier{name: "modifier", args: vec![], span: Default::default()}]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier{name: "modifier1", args: vec![], span: Default::default()}, Modifier{name: "modifier2", args: vec![], span: Default::default()}]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier{name: "modifier", args: vec![StorageMethod::Variable(Ident::new_static("var2"))], span: Default::default()}]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier{name: "modifier", args: vec![StorageMethod::Const(Value::Number(-32.09))], span: Default::default()}]
                ))]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_modifier_null_param() {
        let template = String::from(r#"{var|modifier:null}"#);
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from(r#"{var|modifier:null}"#),
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier{name: "modifier", args: vec![StorageMethod::Const(Value::Null)], span: Default::default()}]
                ))]
            }
        )
    }

    #[test]
    fn parse_template_single_computed_modifier_null_value() {
        let template = String::from(r#"{null|modifier:-32.09}"#);
        let template = parse(template);
        assert!(template.is_ok());
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from(r#"{null|modifier:-32.09}"#),
                tpl: vec![Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Const(Value::Null),
                    vec![Modifier{name: "modifier", args: vec![StorageMethod::Const(Value::Number(-32.09))], span: Default::default()}]
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
                    vec![Modifier{name: "modifier", args: vec![StorageMethod::Const(Value::Number(-32.09))], span: Default::default()}]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier {
                        name: "modifier",
                        args: vec![
                            StorageMethod::Const(Value::Number(-32.09)),
                            StorageMethod::Const(Value::String(String::from("argument"))),
                            StorageMethod::Variable(Ident::new_static("var2")),
                            StorageMethod::Const(Value::Bool(true))
                        ],
                        span: Default::default()
                    }]
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
                        StorageMethod::Variable(Ident::new_static("var")),
                        vec![Modifier{name: "modifier", args: vec![], span: Default::default()}]
                    )),
                    Statement::Literal("\n"),
                    Statement::Calculated(CalculatedValue::new(
                        StorageMethod::Const(Value::Number(10.0)),
                        vec![Modifier {
                            name: "modifier",
                            args: vec![StorageMethod::Const(Value::Number(-32.09))],
                            span: Default::default()
                        }]
                    ))
                ]
            }
        )
    }

    #[cfg(feature = "assign")]
    #[test]
    fn parse_template_assign() {
        let template = String::from("{var = 10|modifier:-32.09}");
        let template = parse(template);
        assert!(template.is_ok(), "{template:#?}");
        let template = template.unwrap();
        assert_eq!(
            template,
            Template {
                tpl_str: String::from("{var = 10|modifier:-32.09}"),
                tpl: vec![Statement::Assign(Assign::new(
                    Ident::new_static("var"),
                    CalculatedValue::new(
                        StorageMethod::Const(Value::Number(10.0)),
                        vec![Modifier{name: "modifier", args: vec![StorageMethod::Const(Value::Number(-32.09))], span: Default::default()}]
                    )
                ))]
            }
        )
    }

    #[cfg(feature = "conditional")]
    mod conditional {
        use super::*;

        #[test]
        fn parse_simple() {
            let template = "{if i < 10}HI{endif}";
            let conditional = TemplateParser::parse(Rule::conditional, template)
                .unwrap()
                .next()
                .unwrap();
            let conditional_statement = parse_conditional(conditional).unwrap();
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::Compare(CompareCondition {
                            left: CalculatedValue::new(
                                StorageMethod::Variable(Ident::new_static("i")),
                                vec![]
                            ),
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
            let conditional_statement = parse_conditional(conditional).unwrap();
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::and(vec![
                            Condition::or(vec![
                                Condition::CalculatedValue(CalculatedValue::new(
                                    StorageMethod::Variable(Ident::new_static("var1")),
                                    vec![]
                                )),
                                Condition::CalculatedValue(CalculatedValue::new(
                                    StorageMethod::Variable(Ident::new_static("var2")),
                                    vec![]
                                ))
                            ]),
                            Condition::CalculatedValue(CalculatedValue::new(
                                StorageMethod::Variable(Ident::new_static("var3")),
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
            let conditional_statement = parse_conditional(conditional).unwrap();
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::Compare(CompareCondition {
                            left: CalculatedValue::new(
                                StorageMethod::Variable(Ident::new_static("i")),
                                vec![]
                            ),
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
        fn parse_multiple() {
            let template = "{if i < 10}HI{else}{if n == \"TEST\"}HI2{else}TEST{endif}{endif}";
            let conditional = TemplateParser::parse(Rule::conditional, template)
                .unwrap()
                .next()
                .unwrap();
            let conditional_statement = parse_conditional(conditional).unwrap();
            if let Statement::Condition(conditional) = conditional_statement {
                assert_eq!(
                    conditional,
                    Conditional {
                        condition: Condition::Compare(CompareCondition {
                            left: CalculatedValue::new(
                                StorageMethod::Variable(Ident::new_static("i")),
                                vec![]
                            ),
                            operator: CompareOperator::LT,
                            right: CalculatedValue::new(
                                StorageMethod::Const(Value::Number(10.)),
                                vec![]
                            )
                        }),
                        then_case: vec![Statement::Literal("HI")],
                        else_case: Some(vec![Statement::Condition(Conditional {
                            condition: Condition::Compare(CompareCondition {
                                left: CalculatedValue::new(
                                    StorageMethod::Variable(Ident::new_static("n")),
                                    vec![]
                                ),
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

    mod ident {
        use crate::parser::{Rule, TemplateParser};
        use crate::value::ident::{Ident, IdentPart};
        use crate::value::StorageMethod;
        use pest::Parser;

        #[test]
        fn simple_ident() {
            let template = "var";
            let value = TemplateParser::parse(Rule::identifier, template)
                .unwrap()
                .next()
                .unwrap();
            let value = super::parse_identifier(value).unwrap();
            assert_eq!(value, Ident::new_static("var"));
        }

        #[test]
        fn path_ident() {
            let template = "var.my";
            let value = TemplateParser::parse(Rule::identifier, template)
                .unwrap()
                .next()
                .unwrap();
            let value = super::parse_identifier(value).unwrap();
            assert_eq!(
                value,
                Ident {
                    part: Box::new(IdentPart::Static("var")),
                    next: Some(Box::new(Ident::new_static("my"))),
                    span: Default::default()
                }
            );
        }

        #[test]
        fn path_dynamic() {
            let template = "var[my]";
            let value = TemplateParser::parse(Rule::identifier, template)
                .unwrap()
                .next()
                .unwrap();
            let value = super::parse_identifier(value).unwrap();
            assert_eq!(
                value,
                Ident {
                    part: Box::new(IdentPart::Static("var")),
                    next: Some(Box::new(Ident {
                        part: Box::new(IdentPart::Dynamic(StorageMethod::Variable(
                            Ident::new_static("my")
                        ))),
                        next: None,
                        span: Default::default()
                    })),
                    span: Default::default()
                }
            );
        }
    }

    mod value {
        use crate::parser::{Rule, TemplateParser};
        use crate::value::Value;
        use crate::value::{ident::Ident, StorageMethod};
        use pest::Parser;

        #[test]
        fn parse_bool_true() {
            let template = "true";
            let value = TemplateParser::parse(Rule::value, template)
                .unwrap()
                .next()
                .unwrap();
            let value = super::parse_value(value).unwrap();
            assert_eq!(value, StorageMethod::Const(Value::Bool(true)));
        }

        #[test]
        fn parse_bool_false() {
            let template = "false";
            let value = TemplateParser::parse(Rule::value, template)
                .unwrap()
                .next()
                .unwrap();
            let value = super::parse_value(value).unwrap();
            assert_eq!(value, StorageMethod::Const(Value::Bool(false)));
        }

        #[test]
        fn parse_ident() {
            let templates = ["foo", "foo_bar", "fooBar", "foo2"];
            templates.iter().for_each(|template| {
                let value = TemplateParser::parse(Rule::value, template)
                    .unwrap()
                    .next()
                    .unwrap();
                let value = super::parse_value(value).unwrap();
                assert_eq!(value, StorageMethod::Variable(Ident::new_static(*template)));
            })
        }

        #[test]
        fn parse_number() {
            let templates = ["12", "-5", "-5.23", "+12.3", "0.2135"];
            templates.iter().for_each(|template| {
                let value = TemplateParser::parse(Rule::value, template)
                    .unwrap()
                    .next()
                    .unwrap();
                let value = super::parse_value(value).unwrap();
                assert_eq!(
                    value,
                    StorageMethod::Const(Value::Number(template.parse::<f64>().unwrap()))
                );
            })
        }

        #[test]
        fn parse_string() {
            let templates = [
                ("\"My test string\"", "My test string"),
                ("\"c\"", "c"),
                ("\"FOO\\\"Bar\"", "FOO\"Bar"),
            ];
            templates.iter().for_each(|(template, expected)| {
                let value = TemplateParser::parse(Rule::value, template)
                    .unwrap()
                    .next()
                    .unwrap();
                let value = super::parse_value(value).unwrap();
                assert_eq!(
                    value,
                    StorageMethod::Const(Value::String((*expected).to_owned()))
                );
            })
        }

        #[test]
        fn parse_null() {
            let template = "null";
            let value = TemplateParser::parse(Rule::value, template)
                .unwrap()
                .next()
                .unwrap();
            let value = super::parse_value(value).unwrap();
            assert_eq!(value, StorageMethod::Const(Value::Null));
        }
    }

    #[cfg(feature = "conditional")]
    mod condition {
        use crate::template::condition::AndCondition;

        use super::*;

        #[test]
        fn parse() {
            let template = "bar";
            let condition = TemplateParser::parse(Rule::condition, template)
                .unwrap()
                .next()
                .unwrap();
            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::CalculatedValue(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("bar")),
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
            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::Compare(CompareCondition {
                    left: CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("bar")),
                        vec![]
                    ),
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
            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::Compare(CompareCondition {
                    left: CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("bar")),
                        vec![]
                    ),
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
            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::Or(OrCondition::new(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var1")),
                        vec![]
                    )),
                    Condition::And(AndCondition::new(vec![
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var2")),
                            vec![]
                        )),
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var3")),
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

            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::Or(OrCondition::new(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var1")),
                        vec![]
                    )),
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var2")),
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

            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::And(AndCondition::new(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var1")),
                        vec![]
                    )),
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var2")),
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

            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::And(AndCondition::new(vec![
                    Condition::Or(OrCondition::new(vec![
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var1")),
                            vec![]
                        )),
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var2")),
                            vec![]
                        ))
                    ])),
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var3")),
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

            let condition = super::parse_condition(condition).unwrap();
            assert_eq!(
                condition,
                Condition::or(vec![
                    Condition::CalculatedValue(CalculatedValue::new(
                        StorageMethod::Variable(Ident::new_static("var1")),
                        vec![]
                    )),
                    Condition::and(vec![
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var2")),
                            vec![]
                        )),
                        Condition::CalculatedValue(CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var3")),
                            vec![]
                        ))
                    ])
                ])
            )
        }
    }

    #[cfg(feature = "assign")]
    mod assign {
        use crate::value::ident::Ident;
        use crate::{
            parser::{parse_assign, Parser, Rule, TemplateParser},
            template::{Assign, CalculatedValue},
            value::{StorageMethod, Value},
        };

        //r#"{my_var = "test"|modifier:arg}"#,

        #[test]
        fn parse_assign_simple() {
            let tpl = "{my_var=12}";
            let assign = TemplateParser::parse(Rule::assign, tpl)
                .unwrap()
                .next()
                .unwrap();
            let assign = parse_assign(assign).unwrap();
            assert_eq!(
                assign,
                Assign::new(
                    Ident::new_static("my_var"),
                    CalculatedValue::new(StorageMethod::Const(Value::Number(12.)), vec![])
                )
            )
        }
    }

    #[cfg(feature = "loop")]
    mod while_loop {
        use crate::value::ident::Ident;
        use crate::{
            parser::{Parser, Rule, TemplateParser},
            template::{
                condition::{CompareCondition, CompareOperator, Condition},
                CalculatedValue, Loop, Statement,
            },
            value::{StorageMethod, Value},
        };

        #[test]
        fn parse_loop() {
            let template = "{while var==0}Foo{endwhile}";

            let l = TemplateParser::parse(Rule::while_loop, template)
                .unwrap()
                .next()
                .unwrap();
            let l = crate::parser::parse_loop(l).unwrap();
            assert_eq!(
                l,
                Loop::new(
                    Condition::Compare(CompareCondition {
                        left: CalculatedValue::new(
                            StorageMethod::Variable(Ident::new_static("var")),
                            vec![]
                        ),
                        operator: CompareOperator::EQ,
                        right: CalculatedValue::new(
                            StorageMethod::Const(Value::Number(0.)),
                            vec![]
                        )
                    }),
                    vec![Statement::Literal("Foo")]
                )
            )
        }
    }
}

#[cfg(test)]
mod pest_tests {

    use super::*;

    const NUMBER_CASES: [&str; 5] = ["42", "42.0", "0.815", "-0.815", "+0.815"];

    const IDENTIFIER_CASES: [&str; 5] = [
        "onlylowercase",
        "camelCase",
        "snail_case",
        "var[0]",
        "test[dyn]",
    ];

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
    fn identifier() {
        test_cases(&IDENTIFIER_CASES, Rule::identifier)
    }

    #[test]
    fn argument() {
        test_cases!(NUMBER_CASES, number_cases, ":{}");
        test_cases(&number_cases, Rule::argument);

        test_cases!(IDENTIFIER_CASES, ident_cases, ":{}");
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
            Rule::template,
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

    #[test]
    fn test_assign() {
        test_cases(
            &["{my_var=12}", r#"{my_var = "test"|modifier:arg}"#],
            Rule::assign,
        )
    }

    #[test]
    fn test_while() {
        test_cases(
            &[
                "{while var==0}1{endwhile}",
                "{ while var == 0 } 1 { endwhile }",
                "{while var==0}\n1\n{endwhile}",
            ],
            Rule::while_loop,
        )
    }

    fn test_cases(cases: &[&str], rule: Rule) {
        cases.iter().for_each(|input| {
            let parsed = TemplateParser::parse(rule, input);
            assert!(parsed.is_ok(), "Failed to parse \"{input}\"\n{parsed:#?}");
            let parsed = parsed.unwrap().next();
            assert!(parsed.is_some(), "{:#?}", parsed);
            let identifier = parsed.unwrap();
            assert_eq!(identifier.as_str(), *input);
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
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![]
                )),
                Statement::Literal(" template " as *const _),
                Statement::Calculated(CalculatedValue::new(
                    StorageMethod::Variable(Ident::new_static("foo")),
                    vec![]
                ))
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier{name: "test", args: vec![], span: Default::default()}]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier {
                        name: "test",
                        args: vec![StorageMethod::Const(Value::String(
                            "test value".to_string()
                        ))],
                        span: Default::default()
                    }]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier {
                        name: "test",
                        args: vec![StorageMethod::Const(Value::Number(42_f64))],
                        span: Default::default()
                    }]
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
                    StorageMethod::Variable(Ident::new_static("var")),
                    vec![Modifier {
                        name: "test",
                        args: vec![StorageMethod::Variable(Ident::new_static("foobar"))],
                        span: Default::default()
                    }]
                )),
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }
}
