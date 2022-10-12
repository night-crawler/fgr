use nom::IResult;

use crate::parse::filter::Filter;

pub trait AliasExt {
    fn get_aliases(&self) -> (&'static [&'static str], &'static str);
    fn split_by_longest_alias(input: &str) -> Option<(&str, &str)>;
}

pub trait GenericParser {
    fn parse(self, input: &str) -> IResult<&str, Filter>;
}
