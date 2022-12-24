use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, Not};

#[derive(Debug, Clone)]
pub enum Nnf<V> {
    Var(V, bool),
    And(BTreeSet<Nnf<V>>),
    Or(BTreeSet<Nnf<V>>),
}

impl<V: Eq + Ord> Eq for Nnf<V> {}

impl<V: Eq + Ord> PartialEq<Self> for Nnf<V> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Var(left_name, left_value), Self::Var(right_name, right_value)) => {
                left_name == right_name && left_value == right_value
            }
            (Self::And(ch1), Self::And(ch2)) => ch1 == ch2,
            (Self::Or(ch1), Self::Or(ch2)) => ch1 == ch2,
            _ => false,
        }
    }
}

impl<V: Eq + Ord + Debug> PartialOrd<Self> for Nnf<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Var(left_name, left_value), Self::Var(right_name, right_value)) => {
                (left_name, left_value).partial_cmp(&(right_name, right_value))
            }
            (Self::Var(_, _), Self::And(_)) => Some(Ordering::Less),
            (Self::And(_), Self::Var(_, _)) => Some(Ordering::Greater),

            (Self::Var(_, _), Self::Or(_)) => Some(Ordering::Less),
            (Self::Or(_), Self::Var(_, _)) => Some(Ordering::Greater),

            (Self::And(left_children), Self::And(right_children))
            | (Self::And(left_children), Self::Or(right_children))
            | (Self::Or(left_children), Self::And(right_children))
            | (Self::Or(left_children), Self::Or(right_children)) => {
                left_children.iter().rev().partial_cmp(right_children.iter().rev())
            }
        }
    }
}

impl<V: Eq + Ord + Debug> Ord for Nnf<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub mod macros {
    macro_rules! var {
        ($name:expr, $val:expr) => {
            crate::evaluate::nnf::Nnf::Var($name, $val)
        };
        ($name:expr) => {
            crate::evaluate::nnf::Nnf::Var($name, true)
        };
    }

    macro_rules! or {
        (
            $($expression:expr),+
        ) => {
            crate::evaluate::nnf::Nnf::Or({
                let mut children = std::collections::BTreeSet::new();
                $(
                    children.insert($expression);
                )+
                children
            })
        };
    }

    macro_rules! and {
        (
            $($expression:expr),+
        ) => {
            crate::evaluate::nnf::Nnf::And({
                let mut children = std::collections::BTreeSet::new();
                $(
                    children.insert($expression);
                )+
                children
            })
        };
    }

    pub(crate) use and;
    pub(crate) use or;
    pub(crate) use var;
}

impl<V: Ord + PartialOrd + Clone + Debug> Nnf<V> {
    pub fn or<I: IntoIterator<Item = Nnf<V>>>(iter: I) -> Self {
        Self::Or(BTreeSet::from_iter(iter))
    }

    pub fn and<I: IntoIterator<Item = Nnf<V>>>(iter: I) -> Self {
        Self::And(BTreeSet::from_iter(iter))
    }

    pub fn is_clause(&self) -> bool {
        match self {
            or @ Nnf::Or(_) => or.is_simple(),
            _ => false,
        }
    }

    pub fn is_simple(&self) -> bool {
        match self {
            Nnf::Var(_, _) => true,
            Nnf::And(children) | Nnf::Or(children) => {
                let mut unique_var_names = BTreeSet::new();
                for child in children {
                    match child {
                        Nnf::Var(name, _) => {
                            if !unique_var_names.insert(name) {
                                return false;
                            }
                        }
                        Nnf::And(_) | Nnf::Or(_) => return false,
                    }
                }

                true
            }
        }
    }

    pub fn is_cnf(&self) -> bool {
        match self {
            Nnf::And(children) => children.iter().all(|child| child.is_clause()),
            _ => false,
        }
    }

    pub fn has_inversions(&self) -> bool {
        let children = match self {
            Nnf::Var(_, _) => return false,
            Nnf::And(children) | Nnf::Or(children) => children,
        };

        for child in children {
            if children.contains(&!child) {
                return true;
            }
        }

        false
    }

    pub fn dump_vars(&self) -> Vec<&Nnf<V>> {
        let mut vars = vec![];
        self.dump_vars_internal(&mut vars);
        vars.sort_unstable();
        vars
    }

    fn dump_vars_internal<'a>(&'a self, vars: &mut Vec<&'a Nnf<V>>) {
        match self {
            var @ Nnf::Var(_, _) => {
                vars.push(var);
            }
            Nnf::And(children) | Nnf::Or(children) => {
                children.iter().for_each(|child| child.dump_vars_internal(vars));
            }
        }
    }
}

impl<V: Ord + PartialOrd + Clone + Debug> BitOr for Nnf<V> {
    type Output = Nnf<V>;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Var | Var
            (left @ Self::Var(_, _), right @ Self::Var(_, _)) => Self::or([left, right]),

            // Or | Var
            (Self::Or(mut children), var @ Self::Var(_, _))
            | (var @ Self::Var(_, _), Self::Or(mut children)) => {
                children.insert(var);
                Self::Or(children)
            }
            // Or | And
            (Self::Or(mut children), right @ Self::And(_))
            | (right @ Self::And(_), Self::Or(mut children)) => {
                children.insert(right);
                Self::Or(children)
            }
            // Or | Or
            (Self::Or(mut left_children), Self::Or(right_children)) => {
                left_children.extend(right_children);
                Self::Or(left_children)
            }

            // Var | And
            (left @ Self::Var(_, _), right @ Self::And(_))
            | (left @ Self::And(_), right @ Self::Var(_, _)) => Self::or([left, right]),

            // And | And
            (left @ Self::And(_), right @ Self::And(_)) => Self::or([left, right]),
        }
    }
}

impl<V: Ord + PartialOrd + Clone + Debug> BitAnd for Nnf<V> {
    type Output = Nnf<V>;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Var & Var
            (left @ Self::Var(_, _), right @ Self::Var(_, _)) => Self::and([left, right]),

            // Var & And
            (var @ Self::Var(_, _), Self::And(mut children))
            | (Self::And(mut children), var @ Self::Var(_, _)) => {
                children.insert(var);
                Self::And(children)
            }

            // Var & Or
            (var @ Self::Var(_, _), or @ Self::Or(_))
            | (or @ Self::Or(_), var @ Self::Var(_, _)) => Self::and([var, or]),

            // And & And
            (Self::And(mut left_children), Self::And(right_children)) => {
                left_children.extend(right_children);
                Self::And(left_children)
            }

            // And & Or
            (Self::And(mut and_children), or @ Self::Or(_))
            | (or @ Self::Or(_), Self::And(mut and_children)) => {
                and_children.insert(or);
                Self::And(and_children)
            }

            (left @ Self::Or(_), right @ Self::Or(_)) => Self::and([left, right]),
        }
    }
}

impl<V: Ord + PartialOrd + Clone> Not for Nnf<V> {
    type Output = Nnf<V>;

    fn not(self) -> Self::Output {
        match self {
            Nnf::Var(name, value) => Nnf::Var(name, !value),
            Nnf::And(_) => unimplemented!("Cannot apply NOT to AND"),
            Nnf::Or(_) => unimplemented!("Cannot apply NOT to OR"),
        }
    }
}

impl<V: Ord + PartialOrd + Clone> Not for &Nnf<V> {
    type Output = Nnf<V>;

    fn not(self) -> Self::Output {
        match self {
            Nnf::Var(name, value) => Nnf::Var(name.clone(), !*value),
            Nnf::And(_) => unimplemented!("Cannot apply NOT to AND"),
            Nnf::Or(_) => unimplemented!("Cannot apply NOT to OR"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::evaluate::nnf::macros::{and, or, var};
    use crate::evaluate::nnf::Nnf;

    #[test]
    fn test_or() {
        let a = var!("a", true);
        let b = var!("b", false);
        let c = var!("c", true);
        let d = var!("d", false);
        let e = var!("e", true);

        assert_eq!(a.clone() | b.clone(), Nnf::or([var!("a", true), var!("b", false)]));

        let or = or!(a.clone(), b.clone());
        assert_eq!(
            or | c.clone() | and!(d.clone()) | or!(e.clone()),
            or!(a.clone(), b.clone(), c.clone(), and!(d.clone()), e)
        )
    }

    #[test]
    fn test_and() {
        let a = var!("a", true);
        let b = var!("b", false);
        let c = var!("c", true);
        let d = var!("d", false);
        let e = var!("e", true);

        assert_eq!(a.clone() & b.clone(), and!(var!("a", true), var!("b", false)));

        let and = and!(a.clone(), b.clone());
        assert_eq!(
            and & c.clone() & Nnf::or([d.clone()]) & and!(e.clone()),
            and!(a.clone(), b.clone(), c.clone(), or!(d.clone()), e.clone())
        )
    }

    #[test]
    fn test_order() {
        assert!(or!(var!("a")) < or!(var!("b")));
        assert!(or!(var!("a"), var!("b")) < or!(var!("c", false), var!("d", false)));
    }
}
