use std::collections::BTreeMap;
use std::fs::Permissions;
use std::ops::Deref;
use std::os::unix::prelude::PermissionsExt;

use chrono::Duration;
use lazy_static::lazy_static;
use nom::branch::alt;
use nom::character::complete::{alphanumeric1, digit1, multispace0};
use nom::combinator::{map, map_res, opt};
use nom::error::ErrorKind;
use nom::sequence::terminated;
use nom::IResult;
use uzers::{Groups, Users, UsersCache};

use crate::parse::comparison::Comparison;
use crate::parse::filter::Filter;
use crate::parse::match_pattern::MatchPattern;
use crate::parse::primitives::{
    parse_comparison, parse_duration, parse_file_type, parse_pattern,
    parse_positive_number, parse_size_unit,
};
use crate::parse::traits::GenericParser;
use crate::parse::util::{prepare_enum_map, ws};
use crate::{mk_filter_enum, GenericError};

lazy_static! {
    static ref SORTED_IDENTIFIERS: BTreeMap<&'static str, &'static str> =
        prepare_enum_map::<AttributeToken>();

    // SAFETY: We will not share the UserCache instance and use it once while parsing a query
    // in the main thread
    static ref USERS: UnsafeWrapper<UsersCache> = unsafe {
        UnsafeWrapper::new(UsersCache::new())
    };
}

struct UnsafeWrapper<T> {
    inner: T,
}

impl<T> Deref for UnsafeWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> UnsafeWrapper<T> {
    unsafe fn new(inner: T) -> Self {
        Self { inner }
    }
}

unsafe impl<T> Send for UnsafeWrapper<T> {}
unsafe impl<T> Sync for UnsafeWrapper<T> {}

#[cfg(not(test))]
mk_filter_enum!(AttributeToken, ATTRIBUTE_TOKEN_ALIASES, [
    Name: "name",
    ModificationTime: "mtime",
    AccessTime: "atime",
    Size: "size",
    Extension: "ext", "extension",
    Contains: "contains",
    Depth: "depth",
    Permissions: "permissions", "perms", "perm",
    Group: "group",
    User: "user",
    Type: "type"
]);

#[cfg(test)]
mk_filter_enum!(AttributeToken, ATTRIBUTE_TOKEN_ALIASES, [
    Name: "name",
    ModificationTime: "mtime",
    AccessTime: "atime",
    Size: "size",
    Extension: "ext", "extension",
    Contains: "contains",
    Depth: "depth",
    Permissions: "permissions", "perms", "perm",
    Group: "group",
    User: "user",
    Type: "type",
    Bool: "bool"
]);

fn parse_comparison_and_pattern(
    input: &str,
) -> IResult<&str, (Comparison, MatchPattern)> {
    let (input, comparison) = parse_comparison(input)?;
    let (input, pattern) = parse_pattern(input)?;

    Ok((input, (comparison, pattern)))
}

fn parse_comparison_and_duration(input: &str) -> IResult<&str, (Comparison, Duration)> {
    let (input, comparison) = parse_comparison(input)?;
    let (input, duration) = parse_duration(input)?;

    Ok((input, (comparison, duration)))
}

fn filter_eq_neq(input: &str, comparison: Comparison) -> IResult<&str, Comparison> {
    if comparison != Comparison::Eq && comparison != Comparison::Neq {
        return Err(nom::Err::Failure(nom::error::Error::new(input, ErrorKind::Fail)));
    }
    Ok((input, comparison))
}

fn get_user(name: &str) -> Result<u32, GenericError> {
    if let Some(value) = USERS.get_user_by_name(name).map(|user| user.uid() as u32) {
        return Ok(value);
    }

    Err(GenericError::WrongTokenType(name.to_string()))
}

fn get_group(name: &str) -> Result<u32, GenericError> {
    if let Some(value) = USERS.get_group_by_name(name).map(|user| user.gid() as u32) {
        return Ok(value);
    }

    Err(GenericError::WrongTokenType(name.to_string()))
}

fn parse_user_or_group(
    f: fn(&str) -> Result<u32, GenericError>,
) -> impl FnMut(&str) -> IResult<&str, u32> {
    move |input: &str| {
        alt((map(parse_positive_number, |num| num as u32), map_res(alphanumeric1, f)))(
            input,
        )
    }
}

impl GenericParser for AttributeToken {
    fn parse(self, input: &str) -> IResult<&str, Filter> {
        Ok(match self {
            Self::Name => {
                let (input, (comparison, pattern)) = parse_comparison_and_pattern(input)?;
                let (input, comparison) = filter_eq_neq(input, comparison)?;

                (input, Filter::Name { value: pattern, comparison })
            }
            Self::Extension => {
                let (input, (comparison, pattern)) = parse_comparison_and_pattern(input)?;
                let (input, comparison) = filter_eq_neq(input, comparison)?;

                (input, Filter::Extension { value: pattern, comparison })
            }
            Self::Contains => {
                let (input, (comparison, pattern)) = parse_comparison_and_pattern(input)?;
                let (input, comparison) = filter_eq_neq(input, comparison)?;

                (input, Filter::Contains { value: pattern, comparison })
            }
            Self::Group => {
                let (input, comparison) = parse_comparison(input)?;
                let (input, value) = parse_user_or_group(get_group)(input)?;

                (input, Filter::User { comparison, value })
            }
            Self::User => {
                let (input, comparison) = parse_comparison(input)?;
                let (input, value) = parse_user_or_group(get_user)(input)?;

                (input, Filter::User { comparison, value })
            }

            Self::AccessTime => {
                let (input, (comparison, duration)) =
                    parse_comparison_and_duration(input)?;
                (input, Filter::AccessTime { value: duration, comparison })
            }
            Self::ModificationTime => {
                let (input, (comparison, duration)) =
                    parse_comparison_and_duration(input)?;
                (input, Filter::ModificationTime { value: duration, comparison })
            }
            Self::Size => {
                let (input, comparison) = parse_comparison(input)?;
                let (input, number) =
                    terminated(parse_positive_number, opt(multispace0))(input)?;
                let (input, unit) = parse_size_unit(input)?;
                let num_bytes = unit.to_bytes(number);

                (input, Filter::Size { value: num_bytes, comparison })
            }
            Self::Depth => {
                let (input, comparison) = parse_comparison(input)?;
                let (input, value) = ws(parse_positive_number)(input)?;

                (input, Filter::Depth { value, comparison })
            }
            Self::Permissions => {
                let (input, comparison) = parse_comparison(input)?;

                let (input, mode) =
                    map_res(ws(digit1), |value| u32::from_str_radix(value, 8))(input)?;

                let value = Permissions::from_mode(mode);

                (input, Filter::Permissions { value, comparison })
            }
            Self::Type => {
                let (input, comparison) = parse_comparison(input)?;
                let (input, value) = ws(parse_file_type)(input)?;

                (input, Filter::Type { value, comparison })
            }

            #[cfg(test)]
            Self::Bool => {
                use nom::bytes::complete::tag;

                let (input, comparison) = parse_comparison(input)?;
                let (input, value) = ws(alt((tag("true"), tag("false"))))(input)?;

                let value = value == "true";

                (input, Filter::Bool { value, comparison })
            }
        })
    }
}
