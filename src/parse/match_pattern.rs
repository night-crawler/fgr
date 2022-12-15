use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub enum MatchPattern {
    Regex(regex::Regex),
    Glob(globset::GlobMatcher),
}

impl PartialEq<Self> for MatchPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Regex(this), Self::Regex(other)) => this.as_str() == other.as_str(),
            (Self::Glob(this), Self::Glob(other)) => {
                this.glob().to_string() == other.glob().to_string()
            }
            unexpected => panic!("Unexpected: {unexpected:?}"),
        }
    }
}

impl From<globset::Glob> for MatchPattern {
    fn from(g: globset::Glob) -> Self {
        Self::Glob(g.compile_matcher())
    }
}

impl From<regex::Regex> for MatchPattern {
    fn from(r: regex::Regex) -> Self {
        Self::Regex(r)
    }
}

impl Eq for MatchPattern {}

impl Display for MatchPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchPattern::Regex(rx) => write!(f, "{}", rx.as_str()),
            MatchPattern::Glob(matcher) => write!(f, "{}", matcher.glob()),
        }
    }
}

impl Debug for MatchPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl MatchPattern {
    pub fn is_match<P>(&self, text: P) -> bool
    where
        P: AsRef<str>,
    {
        match self {
            MatchPattern::Regex(rx) => rx.is_match(text.as_ref()),
            MatchPattern::Glob(glob) => glob.is_match(text.as_ref()),
        }
    }
}
