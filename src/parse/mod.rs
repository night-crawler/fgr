use nom::character::complete::{multispace0, multispace1, space0};
use nom::combinator::{consumed, map_res, recognize};
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    Err, IResult, Parser,
};
use strum::IntoEnumIterator;

use size_unit::SizeUnit;

mod size_unit;
mod time_unit;

enum Token {
    Size(SizeUnit),
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Comparison {
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
}

impl TryFrom<&str> for Comparison {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<=" => Ok(Comparison::Lte),
            ">=" => Ok(Comparison::Gte),
            "!=" => Ok(Comparison::Neq),
            "<" => Ok(Comparison::Lt),
            ">" => Ok(Comparison::Gt),
            "=" => Ok(Comparison::Eq),
            _ => Err(()),
        }
    }
}

fn space<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(input)
}

#[rustfmt::skip]
fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(
        many1(
            terminated(
                one_of("0123456789"),
                many0(char('_')),
            )
        )
    )(input)
}

#[rustfmt::skip]
fn number(input: &str) -> IResult<&str, isize> {
    alt((
        map_res(
            preceded(
                opt(char('+')),
                decimal,
            ),
            |res| res.replace('_', "").parse(),
        ),
        map_res(
            preceded(
                char('-'),
                decimal,
            ),
            |res| res.replace('_', "").parse().map(|num: isize| -num),
        )
    ))(input)
}

#[rustfmt::skip]
fn comparison(input: &str) -> IResult<&str, Comparison> {
    let ops = (
        tag("<="),
        tag(">="),
        tag("!="),
        tag("<"),
        tag(">"),
        tag("="),
    );

    map_res(
        recognize(alt(ops)),
        Comparison::try_from,
    )(input)
}

#[rustfmt::skip]
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        multispace0,
        inner,
        multispace0,
    )
}

#[rustfmt::skip]
fn size(input: &str) -> IResult<&str, SizeUnit> {
    let tags = alt((
        tag("Mb"),
        tag("Kb"),
        tag("Tb")
    ));

    let size_parser = tuple((
        terminated(number, opt(multispace0)),
        recognize(opt(tags))
    ));

    map_res(
        size_parser,
        SizeUnit::try_from,
    )(input)
}

#[rustfmt::skip]
fn key_value(input: &str) {


}

#[rustfmt::skip]
pub fn root() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(number("-12_3_ "), Ok((" ", -123)));

        match number("-12_3_ ") {
            Ok(qwe) => {
                println!("{:?}", qwe);
            }
            Err(e) => match e {
                Err::Incomplete(needed) => println!("Incomplete {:?}", needed),
                Err::Error(error) => println!("Error"),
                Err::Failure(error) => println!("Failure"),
            },
        }
    }

    #[test]
    fn test_comparison() {
        assert_eq!(comparison("<=<="), Ok(("<=", Comparison::Lte)));
        assert_eq!(comparison("<"), Ok(("", Comparison::Lt)));
        assert_eq!(comparison(">="), Ok(("", Comparison::Gte)));
        assert_eq!(comparison(">"), Ok(("", Comparison::Gt)));
        assert_eq!(comparison("="), Ok(("", Comparison::Eq)));
        assert_eq!(comparison("!="), Ok(("", Comparison::Neq)));
    }

    #[test]
    fn size_test() {
        assert_eq!(size("42 Mb"), Ok(("", SizeUnit::Mb(42))));
        assert_eq!(size("42Mb"), Ok(("", SizeUnit::Mb(42))));
        assert_eq!(size("42"), Ok(("", SizeUnit::Byte(42))));
    }
}
