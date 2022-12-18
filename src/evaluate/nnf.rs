use std::collections::BTreeSet;
use std::hint::unreachable_unchecked;
use std::ops::{BitAnd, BitOr, Not};

pub trait Aux {
    fn aux() -> Self;
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Nnf<V> {
    Var(V, bool),
    And(BTreeSet<Nnf<V>>),
    Or(BTreeSet<Nnf<V>>),
}

impl<V: Ord + PartialOrd + Clone + Aux> Nnf<V> {
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

    pub fn to_cnf(self) -> Nnf<V> {
        let mut clauses = BTreeSet::new();
        self.process_required(&mut clauses);
        Self::And(clauses)
    }

    fn process_required(self, clauses: &mut BTreeSet<Nnf<V>>) {
        match self {
            var @ Nnf::Var(_, _) => {
                clauses.insert(Self::or([var]));
            }
            Nnf::And(children) | Nnf::Or(children) if children.len() == 1 => {
                children.into_iter().next().unwrap().process_required(clauses)
            }
            Nnf::Or(children) => {
                let or = Self::or(
                    children.into_iter().map(|child| child.process_node(clauses)),
                );
                if or.has_inversions() {
                    return;
                }
                clauses.insert(or);
            }
            Nnf::And(children) => {
                for child in children {
                    child.process_required(clauses);
                }
            }
        }
    }

    fn has_inversions(&self) -> bool {
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

    fn process_node(self, clauses: &mut BTreeSet<Nnf<V>>) -> Nnf<V> {
        let node = match self {
            var @ Nnf::Var(_, _) => return var,
            Nnf::And(children) | Nnf::Or(children) if children.len() == 1 => {
                return children.into_iter().next().unwrap().process_node(clauses)
            }
            Nnf::And(children) => {
                Nnf::and(children.into_iter().map(|child| child.process_node(clauses)))
            }
            Nnf::Or(children) => {
                Nnf::or(children.into_iter().map(|child| child.process_node(clauses)))
            }
        };

        let aux = Self::Var(V::aux(), true);

        match node {
            Nnf::And(_) if node.has_inversions() => {
                clauses.insert(Self::or([!aux.clone()]));
            }
            Nnf::Or(_) if node.has_inversions() => {
                clauses.insert(Self::or([aux.clone()]));
            }

            Nnf::And(children) => {
                clauses.insert(Self::or(
                    children
                        .iter()
                        .map(|child| !child)
                        .chain(std::iter::once(aux.clone())),
                ));

                for child in children {
                    clauses.insert(Self::or([!aux.clone(), child]));
                }
            }

            Nnf::Or(children) => {
                clauses.insert(Self::or(
                    children.iter().cloned().chain(std::iter::once(!aux.clone())),
                ));

                for child in children {
                    clauses.insert(Self::or([!child, aux.clone()]));
                }
            }

            Self::Var(_, _) => unsafe { unreachable_unchecked() },
        }

        aux
    }
}

impl<V: Ord + PartialOrd + Clone + Aux> BitOr for Nnf<V> {
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

impl<V: Ord + PartialOrd + Clone + Aux> BitAnd for Nnf<V> {
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use lazy_static::lazy_static;

    use crate::evaluate::nnf::{Nnf, Aux};

    lazy_static! {
        static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
    }

    impl Aux for &str {
        fn aux() -> &'static str {
            let name = format!("aux_{}", COUNTER.fetch_add(1, Ordering::Relaxed));
            Box::leak(Box::new(name))
        }
    }

    #[test]
    fn test_or() {
        let a = Nnf::Var("a", true);
        let b = Nnf::Var("b", false);
        let c = Nnf::Var("c", true);
        let d = Nnf::Var("d", false);
        let e = Nnf::Var("e", true);

        assert_eq!(
            a.clone() | b.clone(),
            Nnf::or([Nnf::Var("a", true), Nnf::Var("b", false)])
        );

        let or = Nnf::or([a.clone(), b.clone()]);
        assert_eq!(
            or | c.clone() | Nnf::and([d.clone()]) | Nnf::or([e.clone()]),
            Nnf::or([a.clone(), b.clone(), c.clone(), Nnf::and([d.clone()]), e])
        )
    }

    #[test]
    fn test_and() {
        let a = Nnf::Var("a", true);
        let b = Nnf::Var("b", false);
        let c = Nnf::Var("c", true);
        let d = Nnf::Var("d", false);
        let e = Nnf::Var("e", true);

        assert_eq!(
            a.clone() & b.clone(),
            Nnf::and([Nnf::Var("a", true), Nnf::Var("b", false)])
        );

        let and = Nnf::and([a.clone(), b.clone()]);
        assert_eq!(
            and & c.clone() & Nnf::or([d.clone()]) & Nnf::and([e.clone()]),
            Nnf::and([a.clone(), b.clone(), c.clone(), Nnf::or([d.clone()]), e.clone()])
        )
    }

    #[test]
    fn validate() {
        let sentence = Nnf::or([
            Nnf::and([
                Nnf::Var("g", true),
                Nnf::and([Nnf::Var("e", true), Nnf::Var("f", true)]),
            ]),
            Nnf::and([
                Nnf::or([Nnf::Var("a", false), Nnf::Var("b", true)]),
                Nnf::and([Nnf::Var("c", true), Nnf::Var("d", false)]),
            ]),
        ]);

        assert!(!sentence.is_cnf());

        let sentence = sentence.to_cnf();
        assert!(sentence.is_cnf());
    }
}
