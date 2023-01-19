use std::str::FromStr;

use chrono::Duration;
use globset::GlobBuilder;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char, one_of};
use nom::combinator::{map, map_res, opt, recognize};
use nom::error::{ErrorKind, FromExternalError};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use regex::RegexBuilder;

use crate::parse::attribute_token::AttributeToken;
use crate::parse::comparison::Comparison;
use crate::parse::file_type::FileType;
use crate::parse::match_pattern::MatchPattern;
use crate::parse::size_unit::SizeUnit;
use crate::parse::time_unit::TimeUnit;
use crate::parse::util::{parse_enum_alias, ws};

const SINGLE_QUOTE_CHAR: char = '\'';
const SINGLE_QUOTE_BYTE: u8 = b'\'';

const DOUBLE_QUOTE_CHAR: char = '"';
const DOUBLE_QUOTE_BYTE: u8 = b'"';

const BACK_SLASH_BYTE: u8 = b'\\';

#[rustfmt::skip]
pub fn parse_decimal(input: &str) -> IResult<&str, &str> {
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
pub fn parse_positive_number(input: &str) -> IResult<&str, usize> {
    map_res(
        preceded(
            opt(char('+')),
            parse_decimal,
        ),
        |res| res.replace('_', "").parse(),
    )(input)
}

#[rustfmt::skip]
pub fn parse_negative_number(input: &str) -> IResult<&str, isize> {
    map(
        preceded(
            char('-'),
            parse_positive_number,
        ),
        |positive| -(positive as isize),
    )(input)
}

#[rustfmt::skip]
pub fn parse_comparison(input: &str) -> IResult<&str, Comparison> {
    let ops = (
        tag("<="),
        tag(">="),
        tag("!="),
        tag("<"),
        tag(">"),
        tag("="),
    );

    map_res(
        ws(recognize(alt(ops))),
        Comparison::try_from,
    )(input)
}

fn parse_signed_delta(input: &str) -> IResult<&str, Duration> {
    let (input, sign) = ws(alt((char('+'), char('-'))))(input)?;
    let (input, number) = parse_positive_number(input)?;
    let (input, time_unit) = parse_time_unit(input)?;

    let mut duration = time_unit.to_duration(number as i64);
    if sign == '-' {
        duration = -duration;
    }

    Ok((input, duration))
}

pub fn parse_duration(input: &str) -> IResult<&str, Duration> {
    let (input, _) = ws(tag("now"))(input)?;
    let (input, duration) = opt(parse_signed_delta)(input)?;
    let duration = duration.unwrap_or_else(|| TimeUnit::Second.to_duration(0));

    Ok((input, duration))
}

pub fn parse_size_unit(input: &str) -> IResult<&str, SizeUnit> {
    map_res(ws(parse_enum_alias::<SizeUnit>()), SizeUnit::from_str)(input)
}

pub fn parse_time_unit(input: &str) -> IResult<&str, TimeUnit> {
    map_res(ws(parse_enum_alias::<TimeUnit>()), TimeUnit::from_str)(input)
}

pub fn parse_file_type(input: &str) -> IResult<&str, FileType> {
    map_res(ws(parse_enum_alias::<FileType>()), FileType::from_str)(input)
}

pub fn parse_attribute_name(input: &str) -> IResult<&str, AttributeToken> {
    map_res(ws(parse_enum_alias::<AttributeToken>()), AttributeToken::from_str)(input)
}

pub fn parse_first_non_escaped_quote(
    quote: u8,
) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let bytes = input.as_bytes();

        for (index, (&left, &right)) in bytes.iter().tuple_windows().enumerate() {
            if right == quote && left != BACK_SLASH_BYTE {
                return Ok((&input[index + 1..], &input[..index + 1]));
            }
        }

        Ok(("", input))
    }
}

pub fn parse_quote_escaped_string(input: &str) -> IResult<&str, &str> {
    let single_quote = delimited(
        char(SINGLE_QUOTE_CHAR),
        parse_first_non_escaped_quote(SINGLE_QUOTE_BYTE),
        char(SINGLE_QUOTE_CHAR),
    );
    let double_quote = delimited(
        char(DOUBLE_QUOTE_CHAR),
        parse_first_non_escaped_quote(DOUBLE_QUOTE_BYTE),
        char(DOUBLE_QUOTE_CHAR),
    );

    alt((single_quote, double_quote))(input)
}

pub fn parse_pattern_till_first_space(input: &str) -> IResult<&str, (bool, &str)> {
    let (input, pattern) =
        take_while(|ch: char| !ch.is_whitespace() && !"()".contains(ch))(input)?;

    Ok((input, (false, pattern)))
}

pub fn parse_glob_pattern(input: &str) -> IResult<&str, MatchPattern> {
    let (input, (ignore_case, pattern)) = alt((
        parse_ignore_case_quote_escaped_string,
        parse_pattern_till_first_space,
    ))(input)?;

    match GlobBuilder::new(pattern).case_insensitive(ignore_case).build() {
        Ok(glob) => Ok((input, glob.into())),
        Err(err) => Err(nom::Err::Error(nom::error::Error::from_external_error(
            input,
            ErrorKind::Alt,
            err,
        ))),
    }
}

pub fn parse_ignore_case_quote_escaped_string(
    input: &str,
) -> IResult<&str, (bool, &str)> {
    let (input, (ignore_case, pattern)) =
        tuple((opt(char('i')), parse_quote_escaped_string))(input)?;

    Ok((input, (ignore_case.is_some(), pattern)))
}

pub fn parse_regex_pattern(input: &str) -> IResult<&str, MatchPattern> {
    let (input, (ignore_case, pattern)) =
        preceded(char('r'), parse_ignore_case_quote_escaped_string)(input)?;
    compile_regex(input, ignore_case, pattern)
}

pub fn parse_pattern(input: &str) -> IResult<&str, MatchPattern> {
    alt((parse_regex_pattern, parse_glob_pattern))(input)
}

fn compile_regex<'a, 'b>(
    input: &'a str,
    ignore_case: bool,
    pattern: &'b str,
) -> IResult<&'a str, MatchPattern> {
    match RegexBuilder::new(pattern).case_insensitive(ignore_case).build() {
        Ok(rx) => Ok((input, MatchPattern::Regex(rx))),
        Err(err) => Err(nom::Err::Error(nom::error::Error::from_external_error(
            input,
            ErrorKind::Alt,
            err,
        ))),
    }
}

#[cfg(test)]
mod test_primitives {
    use globset::Glob;
    use regex::Regex;

    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(parse_negative_number("-12_3_ "), Ok((" ", -123)));
        assert_eq!(parse_positive_number("12_3_ "), Ok((" ", 123)));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(parse_comparison("<=<="), Ok(("<=", Comparison::Lte)));
        assert_eq!(parse_comparison("<"), Ok(("", Comparison::Lt)));
        assert_eq!(parse_comparison(">="), Ok(("", Comparison::Gte)));
        assert_eq!(parse_comparison(">"), Ok(("", Comparison::Gt)));
        assert_eq!(parse_comparison("="), Ok(("", Comparison::Eq)));
        assert_eq!(parse_comparison("!="), Ok(("", Comparison::Neq)));
    }

    #[test]
    fn test_parse_time_unit() {
        assert_eq!(parse_time_unit("minute"), Ok(("", TimeUnit::Minute)));
        assert!(parse_time_unit("minu").is_err());
    }

    #[test]
    fn test_parse_size_unit() {
        assert_eq!(parse_size_unit("Kb"), Ok(("", SizeUnit::Kilobyte)));
        assert!(parse_size_unit("k").is_err());

        assert_eq!(parse_size_unit("B"), Ok(("", SizeUnit::Byte)));
    }

    #[test]
    fn test_parse_filter() {
        assert_eq!(parse_attribute_name("size"), Ok(("", AttributeToken::Size)));
        assert!(parse_attribute_name("s").is_err());
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("now - 1d"), Ok(("", Duration::days(-1))));
        assert_eq!(parse_duration("now"), Ok(("", Duration::days(0))));
    }

    #[test]
    fn test_parse_glob_pattern() {
        fn g(pattern: &str) -> MatchPattern {
            MatchPattern::Glob(Glob::new(pattern).unwrap().compile_matcher())
        }

        assert_eq!(
            parse_glob_pattern(r"'привет sample\'.jsoon' lol"),
            Ok((" lol", g(r"привет sample\'.jsoon")))
        );

        assert_eq!(parse_glob_pattern(r"'json'"), Ok(("", g("json"))));

        assert_eq!(parse_glob_pattern(r"' '"), Ok(("", g(" "))));

        assert_eq!(parse_glob_pattern(r"'\''"), Ok(("", g(r"\'"))));

        assert_eq!(parse_glob_pattern(r"sample?*="), Ok(("", g("sample?*="))));

        assert_eq!(parse_glob_pattern("\"a json\""), Ok(("", g("a json"))));
    }

    #[test]
    fn test_parse_glob_ignore_case_pattern() {
        fn g(pattern: &str) -> MatchPattern {
            GlobBuilder::new(pattern).case_insensitive(true).build().unwrap().into()
        }

        assert_eq!(parse_glob_pattern(r"i'sample?*='"), Ok(("", g("sample?*="))));
    }

    #[test]
    fn test_parse_regex_pattern() {
        fn r(pattern: &str) -> MatchPattern {
            MatchPattern::Regex(Regex::new(pattern).unwrap())
        }

        assert_eq!(
            parse_regex_pattern(r"r'sample.+привет.+'"),
            Ok(("", r(r"sample.+привет.+")))
        );

        assert_eq!(parse_regex_pattern(r##"r"sample.+""##), Ok(("", r(r"sample.+"))));
    }

    #[test]
    fn test_parse_regex_ignore_case_pattern() {
        fn r(pattern: &str) -> MatchPattern {
            RegexBuilder::new(pattern).case_insensitive(true).build().unwrap().into()
        }

        assert_eq!(
            parse_regex_pattern(r"ri'sample'"),
            Ok(("", r(r"sample")))
        );
    }

    #[test]
    fn test_parse_pattern_till_first_space() {
        assert_eq!(parse_pattern_till_first_space("sample"), Ok(("", (false, "sample"))));
    }
}
