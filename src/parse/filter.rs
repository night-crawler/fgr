use std::fmt::{Debug, Display, Formatter};
use std::fs::Permissions;
use std::ops::Not;
use std::os::unix::prelude::PermissionsExt;

use chrono::Duration;
use strum_macros::IntoStaticStr;

use crate::parse::comparison::Comparison;
use crate::parse::file_type::FileType;
use crate::parse::match_pattern::MatchPattern;

#[derive(Eq, PartialEq, Clone, IntoStaticStr)]
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

impl Not for Filter {
    type Output = Filter;

    fn not(mut self) -> Self::Output {
        match self {
            Self::Size { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Depth { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Type { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::AccessTime { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::ModificationTime { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Name { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Extension { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Contains { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::User { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Group { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
            Self::Permissions { ref mut comparison, .. } => {
                comparison.negate();
                self
            }

            #[cfg(test)]
            Self::Bool { ref mut comparison, .. } => {
                comparison.negate();
                self
            }
        }
    }
}

impl Filter {
    pub fn weight(&self) -> usize {
        match self {
            Filter::Name { value, .. } => match value {
                MatchPattern::Regex(_) => 2,
                MatchPattern::Glob(_) => 1,
            },
            Filter::Extension { value, .. } => match value {
                MatchPattern::Regex(_) => 2,
                MatchPattern::Glob(_) => 1,
            },
            Filter::Depth { .. } => 1,

            Filter::Size { .. } => 4,
            Filter::AccessTime { .. } => 4,
            Filter::ModificationTime { .. } => 4,
            Filter::User { .. } => 4,
            Filter::Group { .. } => 4,
            Filter::Permissions { .. } => 4,

            Filter::Type { .. } => 16,
            Filter::Contains { .. } => 8,

            #[cfg(test)]
            Filter::Bool { .. } => 1,
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(test)]
        {
            if let Self::Bool { .. } = self {
            } else {
                let self_repr: &'static str = self.into();
                write!(f, "{} ", self_repr)?;
            }
        }

        #[cfg(not(test))]
        {
            let self_repr: &'static str = self.into();
            write!(f, "{} ", self_repr)?;
        }

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
            Self::Bool { comparison: _, value } => {
                write!(f, "{}", &format!("{value}")[..1])
            }
        }
    }
}

impl Debug for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
