use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Eq, PartialEq)]
pub enum Comparison {
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
}

impl Comparison {
    pub fn negate(&mut self) {
        *self = match self {
            Self::Lt => Self::Gte,
            Self::Gt => Self::Lte,
            Self::Lte => Self::Gt,
            Self::Gte => Self::Lt,
            Self::Eq => Self::Neq,
            Self::Neq => Self::Eq,
        }
    }
}

impl TryFrom<&str> for Comparison {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<=" => Ok(Comparison::Lte),
            ">=" => Ok(Comparison::Gte),
            "!=" => Ok(Comparison::Neq),
            "<" => Ok(Comparison::Lt),
            ">" => Ok(Comparison::Gt),
            "=" => Ok(Comparison::Eq),
            _ => Err(()),
        }
    }
}

impl Debug for Comparison {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Comparison {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Comparison::Lt => "<",
            Comparison::Gt => ">",
            Comparison::Lte => "<=",
            Comparison::Gte => ">=",
            Comparison::Eq => "=",
            Comparison::Neq => "!=",
        };

        write!(f, "{repr}")
    }
}
