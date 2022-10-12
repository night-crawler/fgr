use crate::parse::comparison::Comparison;

impl Comparison {
    pub fn evaluate<T>(&self, left: T, right: T) -> bool
    where
        T: PartialOrd,
    {
        match self {
            Comparison::Lt => left < right,
            Comparison::Gt => left > right,
            Comparison::Lte => left <= right,
            Comparison::Gte => left >= right,
            Comparison::Eq => left == right,
            Comparison::Neq => left != right,
        }
    }
}
