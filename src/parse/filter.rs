use std::fmt::{Debug, Display, Formatter};
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;

use chrono::Duration;
use strum_macros::IntoStaticStr;

use crate::parse::comparison::Comparison;
use crate::parse::file_type::FileType;
use crate::parse::match_pattern::MatchPattern;

#[derive(Eq, PartialEq, IntoStaticStr)]
pub enum Filter {
    Size {
        value: usize,
        comparison: Comparison,
    },
    Depth {
        value: usize,
        comparison: Comparison,
    },
    Type {
        value: FileType,
        comparison: Comparison,
    },
    AccessTime {
        value: Duration,
        comparison: Comparison,
    },
    ModificationTime {
        value: Duration,
        comparison: Comparison,
    },
    Name {
        value: MatchPattern,
        comparison: Comparison,
    },
    Extension {
        value: MatchPattern,
        comparison: Comparison,
    },
    Contains {
        value: MatchPattern,
        comparison: Comparison,
    },
    User {
        value: u32,
        comparison: Comparison,
    },
    Group {
        value: u32,
        comparison: Comparison,
    },
    Permissions {
        value: Permissions,
        comparison: Comparison,
    },

    #[cfg(test)]
    Bool {
        value: bool,
        comparison: Comparison,
    },
}

impl Filter {
    pub fn negate(&mut self) {
        match self {
            Self::Size { comparison, .. } => comparison.negate(),
            Self::Depth { comparison, .. } => comparison.negate(),
            Self::Type { comparison, .. } => comparison.negate(),
            Self::AccessTime { comparison, .. } => comparison.negate(),
            Self::ModificationTime { comparison, .. } => comparison.negate(),
            Self::Name { comparison, .. } => comparison.negate(),
            Self::Extension { comparison, .. } => comparison.negate(),
            Self::Contains { comparison, .. } => comparison.negate(),
            Self::User { comparison, .. } => comparison.negate(),
            Self::Group { comparison, .. } => comparison.negate(),
            Self::Permissions { comparison, .. } => comparison.negate(),

            #[cfg(test)]
            Self::Bool { comparison, .. } => comparison.negate(),
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let self_repr: &'static str = self.into();
        write!(f, "{} ", self_repr)?;

        match self {
            Self::Size { comparison, value } => write!(f, "{comparison} {value}"),
            Self::Depth { comparison, value } => write!(f, "{comparison} {value}"),
            Self::Type { comparison, value } => write!(f, "{comparison} {value}"),
            Self::AccessTime { comparison, value } => write!(f, "{comparison} {value}"),
            Self::ModificationTime { comparison, value } => {
                write!(f, "{comparison} {value}")
            }
            Self::Name { comparison, value } => write!(f, "{comparison} {value}"),
            Self::Extension { comparison, value } => write!(f, "{comparison} {value}"),
            Self::Contains { comparison, value } => write!(f, "{comparison} {value}"),
            Self::User { comparison, value } => write!(f, "{comparison} {value}"),
            Self::Group { comparison, value } => write!(f, "{comparison} {value}"),

            Self::Permissions { comparison, value } => {
                write!(f, "{comparison} {}", unix_mode::to_string(value.mode()))
            }
            #[cfg(test)]
            Self::Bool { comparison, value } => write!(f, "{comparison} {value}"),
        }
    }
}

impl Debug for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
