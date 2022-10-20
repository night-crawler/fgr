use nom::multi::many0;
use nom::sequence::tuple;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::map,
    sequence::delimited, IResult,
};

use crate::errors::GenericError;
use crate::parse::expression_node::ExpressionNode;
use crate::parse::primitives::parse_attribute_name;
use crate::parse::traits::GenericParser;
use crate::parse::util::ws;

pub mod ast_node;
pub mod attribute_token;
pub mod comparison;
pub mod expression_node;
pub mod file_type;
pub mod filter;
pub mod match_pattern;
pub mod primitives;
pub mod render;
pub mod size_unit;
pub mod time_unit;
pub mod traits;
pub mod util;

fn parse_attribute(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, attribute) = parse_attribute_name(input)?;
    let (input, filter) = attribute.parse(input)?;

    Ok((input, ExpressionNode::Leaf(filter)))
}

fn parse_parens(input: &str) -> IResult<&str, ExpressionNode> {
    let expressions = delimited(ws(char('(')), parse_or, ws(char(')')));
    ws(expressions)(input)
}

fn parse_parens_or_attribute(input: &str) -> IResult<&str, ExpressionNode> {
    alt((parse_parens, parse_attribute, parse_not))(input)
}

#[rustfmt::skip]
fn parse_not(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, _) = ws(tag("not"))(input)?;
    map(
        alt((
            parse_attribute,
            parse_parens_or_attribute
        )),
        |expression| ExpressionNode::Not(expression.into()),
    )(input)
}

#[rustfmt::skip]
fn parse_or(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, left) = parse_and(input)?;
    let (input, expressions) = many0(
        tuple((
            ws(tag("or")),
            parse_and
        ))
    )(input)?;

    Ok((input, parse_expression(left, expressions)))
}

#[rustfmt::skip]
fn parse_and(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, left) = parse_parens_or_attribute(input)?;
    let (input, expressions) = many0(
        tuple((
            ws(tag("and")),
            parse_and
        ))
    )(input)?;

    Ok((input, parse_expression(left, expressions)))
}

#[rustfmt::skip]
fn parse_expression(expr: ExpressionNode, rem: Vec<(&str, ExpressionNode)>) -> ExpressionNode {
    rem.into_iter().fold(
        expr,
        |acc, val| parse_operator(val, acc),
    )
}

fn parse_operator(
    (operator, expression_right): (&str, ExpressionNode),
    expression_left: ExpressionNode,
) -> ExpressionNode {
    match operator {
        "and" => ExpressionNode::And(expression_left.into(), expression_right.into()),
        "or" => ExpressionNode::Or(expression_left.into(), expression_right.into()),
        _ => panic!("Unknown operator: {operator}"),
    }
}

pub fn parse_root(input: &str) -> Result<ExpressionNode, GenericError> {
    let (remainder, expression) = parse_or(input)?;
    if !remainder.trim().is_empty() {
        return Err(GenericError::SomeTokensWereNotParsed(remainder.to_string()));
    }

    Ok(expression)
}

#[cfg(test)]
mod test {
    use chrono::Duration;
    use regex::Regex;

    use crate::parse::comparison::Comparison;
    use crate::parse::file_type::FileType;
    use crate::parse::filter::Filter;

    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(
            parse_attribute("size <= 1 B"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::Size {
                    value: 1,
                    comparison: Comparison::Lte,
                })
            ))
        );

        assert_eq!(
            parse_attribute(" size != 10B"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::Size {
                    value: 10,
                    comparison: Comparison::Neq,
                })
            ))
        );
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(
            parse_attribute("mtime <= now - 2d"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::ModificationTime {
                    value: Duration::days(-2),
                    comparison: Comparison::Lte,
                })
            ))
        );

        assert_eq!(
            parse_attribute("atime <= now - 2d"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::AccessTime {
                    value: Duration::days(-2),
                    comparison: Comparison::Lte,
                })
            ))
        );
    }

    #[test]
    fn test_parse_name() {
        assert_eq!(
            parse_attribute("name = '.*sa mple*.json'"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::Name {
                    value: globset::Glob::new(".*sa mple*.json").unwrap().into(),
                    comparison: Comparison::Eq,
                })
            ))
        );

        assert_eq!(
            parse_attribute("contains != r'пример.json' remainder"),
            Ok((
                " remainder",
                ExpressionNode::Leaf(Filter::Contains {
                    value: Regex::new("пример.json").unwrap().into(),
                    comparison: Comparison::Neq,
                })
            ))
        );
    }

    #[test]
    fn test_parse_depth() {
        assert_eq!(
            parse_attribute("depth != 2"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::Depth {
                    value: 2,
                    comparison: Comparison::Neq,
                })
            ))
        );
    }

    #[test]
    fn test_parse_file_type() {
        assert_eq!(
            parse_attribute("type != vid"),
            Ok((
                "",
                ExpressionNode::Leaf(Filter::Type {
                    value: FileType::Video,
                    comparison: Comparison::Neq,
                })
            ))
        );
    }

    #[test]
    fn parse_sample_1() {
        let input = "name = aaaa and mtime <= now - 1d and size <= 1B and not (not type = vid and size >= 2B or size != 3B) or size = 4B";
        let result = parse_root(input);

        assert!(result.is_ok());
    }

    #[test]
    fn parse_sample_2() {
        let input = "name = .*sample.*' and not (name = '.*.xml' or name = '.*.html')";
        let result = parse_root(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_sample_3() {
        let input = "name=*s* and perm=777 or (name=*rs and contains = *birth*)";
        let result = parse_root(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_sample_4() {
        let input = "name=*s* and perm=777 or (   name=*rs and contains = *birth*  )";
        let result = parse_root(input);
        assert!(result.is_ok());
    }
}
