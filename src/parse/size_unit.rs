use std::collections::BTreeMap;

use lazy_static::lazy_static;
use strum_macros::EnumIter;
use strum_macros::EnumString;

use crate::parse::traits::AliasExt;
use crate::parse::util::{prepare_enum_map, split_by_longest_alias};

lazy_static! {
    static ref SORTED_IDENTIFIERS: BTreeMap<&'static str, &'static str> =
        prepare_enum_map::<SizeUnit>();
}

#[derive(Debug, Eq, PartialEq, EnumIter, EnumString)]
pub enum SizeUnit {
    Byte,
    Kilobyte,
    Megabyte,
    Terabyte,
}

impl SizeUnit {
    pub fn to_bytes(&self, value: usize) -> usize {
        match self {
            SizeUnit::Byte => value,
            SizeUnit::Kilobyte => value * 1000,
            SizeUnit::Megabyte => value * 1000 * 1000,
            SizeUnit::Terabyte => value * 1000 * 1000 * 1000,
        }
    }
}

impl AliasExt for SizeUnit {
    fn get_aliases(&self) -> (&'static [&'static str], &'static str) {
        match self {
            SizeUnit::Byte => (&["B", "byte", "bytes"][..], "Byte"),
            SizeUnit::Kilobyte => (&["Kb", "kilobyte"][..], "Kilobyte"),
            SizeUnit::Megabyte => (&["Mb"][..], "Megabyte"),
            SizeUnit::Terabyte => (&["Tb"][..], "Terabyte"),
        }
    }

    fn split_by_longest_alias(input: &str) -> Option<(&str, &str)> {
        split_by_longest_alias(input, SORTED_IDENTIFIERS.iter().rev())
    }
}
