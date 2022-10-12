use std::collections::BTreeMap;

use nom::character::complete::multispace0;
use nom::error::{ErrorKind, ParseError};
use nom::sequence::delimited;
use nom::Err::Error;
use nom::IResult;
use strum::IntoEnumIterator;

use crate::parse::traits::AliasExt;

pub fn prepare_enum_map<Q>() -> BTreeMap<&'static str, &'static str>
where
    Q: IntoEnumIterator,
    <<Q as IntoEnumIterator>::Iterator as Iterator>::Item: AliasExt,
{
    let mut map = BTreeMap::new();
    for (aliases, canonical) in Q::iter().map(|unit| unit.get_aliases()) {
        for &alias in aliases {
            if let Some(existing) = map.insert(alias, canonical) {
                panic!("Duplicate alias: {existing} for {canonical}");
            }
        }
    }
    map
}

pub fn split_by_longest_alias<'a>(
    input: &'a str,
    identifiers: impl Iterator<Item = (&'a &'a str, &'a &'a str)>,
) -> Option<(&'a str, &'a str)> {
    for (alias, canonical_name) in identifiers {
        if let Some(suffix) = input.strip_prefix(alias) {
            if suffix.is_empty() {
                return Some((suffix, canonical_name));
            }
            if suffix.chars().next().unwrap().is_alphanumeric() {
                return None;
            }
            return Some((suffix, canonical_name));
        }
    }

    None
}

pub fn parse_enum_alias<Q>() -> impl FnMut(&str) -> IResult<&str, &str>
where
    Q: AliasExt,
{
    move |input| match Q::split_by_longest_alias(input) {
        Some(result) => Ok(result),
        None => Err(Error(nom::error::Error::new(input, ErrorKind::NoneOf))),
    }
}

#[rustfmt::skip]
pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        multispace0,
        inner,
        multispace0,
    )
}
