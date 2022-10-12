use std::collections::BTreeMap;

use chrono::Duration;
use lazy_static::lazy_static;
use strum_macros::{EnumIter, EnumString};

use crate::parse::traits::AliasExt;
use crate::parse::util::{prepare_enum_map, split_by_longest_alias};

lazy_static! {
    static ref SORTED_IDENTIFIERS: BTreeMap<&'static str, &'static str> =
        prepare_enum_map::<TimeUnit>();
}

#[derive(Debug, Eq, PartialEq, EnumIter, EnumString)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
}

impl TimeUnit {
    pub fn to_duration(&self, value: i64) -> Duration {
        match self {
            TimeUnit::Second => Duration::seconds(value),
            TimeUnit::Minute => Duration::minutes(value),
            TimeUnit::Hour => Duration::hours(value),
            TimeUnit::Day => Duration::days(value),
        }
    }
}

impl AliasExt for TimeUnit {
    fn get_aliases(&self) -> (&'static [&'static str], &'static str) {
        match self {
            TimeUnit::Second => (&["s", "secs"][..], "Second"),
            TimeUnit::Minute => (&["m", "min", "mins", "minute"][..], "Minute"),
            TimeUnit::Hour => (&["h"][..], "Hour"),
            TimeUnit::Day => (&["d"][..], "Day"),
        }
    }

    fn split_by_longest_alias(input: &str) -> Option<(&str, &str)> {
        split_by_longest_alias(input, SORTED_IDENTIFIERS.iter().rev())
    }
}
