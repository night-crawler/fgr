use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use infer::MatcherType;

use lazy_static::lazy_static;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

use crate::parse::traits::AliasExt;
use crate::parse::util::{prepare_enum_map, split_by_longest_alias};

lazy_static! {
    static ref SORTED_IDENTIFIERS: BTreeMap<&'static str, &'static str> =
        prepare_enum_map::<FileType>();
}

#[derive(Debug, Eq, PartialEq, EnumIter, EnumString, IntoStaticStr)]
pub enum FileType {
    App,
    Archive,
    Audio,
    Book,
    Doc,
    Font,
    Image,
    Text,
    Video,
    Custom
}

impl AliasExt for FileType {
    fn get_aliases(&self) -> (&'static [&'static str], &'static str) {
        match self {
            Self::Text => (&["t", "text"][..], "Text"),
            Self::App => (&["app"][..], "App"),
            Self::Archive => (&["archive"][..], "Archive"),
            Self::Audio => (&["audio"][..], "Audio"),
            Self::Book => (&["book"][..], "Book"),
            Self::Doc => (&["doc"][..], "Doc"),
            Self::Font => (&["font"][..], "Font"),
            Self::Image => (&["image", "img"][..], "Image"),
            Self::Video => (&["video", "vid"][..], "Video"),
            Self::Custom => (&["custom"][..], "Custom"),
        }
    }

    fn split_by_longest_alias(input: &str) -> Option<(&str, &str)> {
        split_by_longest_alias(input, SORTED_IDENTIFIERS.iter().rev())
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let repr: &'static str = self.into();
        write!(f, "{repr}")
    }
}

impl From<MatcherType> for FileType {
    fn from(matcher_type: MatcherType) -> Self {
        match matcher_type {
            MatcherType::App => Self::App,
            MatcherType::Archive => Self::Archive,
            MatcherType::Audio => Self::Audio,
            MatcherType::Book => Self::Book,
            MatcherType::Doc => Self::Doc,
            MatcherType::Font => Self::Font,
            MatcherType::Image => Self::Image,
            MatcherType::Text => Self::Text,
            MatcherType::Video => Self::Video,
            MatcherType::Custom => Self::Custom,
        }
    }
}