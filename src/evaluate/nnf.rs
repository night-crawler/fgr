use std::collections::BTreeSet;
use std::hint::unreachable_unchecked;
use std::ops::{BitAnd, BitOr, Not};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Nnf<V> {
    Var(V, bool),
    And(BTreeSet<Nnf<V>>),
    Or(BTreeSet<Nnf<V>>),
}

#[macro_export]
macro_rules! var {
    ($name:expr, $val:expr) => {
        self::Nnf::Var($name, $val)
    };
    ($name:expr) => {
        self::Nnf::Var($name, true)
    };
}

#[macro_export]
macro_rules! or {
    (
        $($expression:expr),+
    ) => {
        self::Nnf::Or({
            let mut children = std::collections::BTreeSet::new();
            $(
                children.insert($expression);
            )+
            children
        })
    };
}

#[macro_export]
macro_rules! and {
    (
        $($expression:expr),+
    ) => {
        self::Nnf::And({
            let mut children = std::collections::BTreeSet::new();
            $(
                children.insert($expression);
            )+
            children
        })
    };
}

impl<V: Ord + PartialOrd + Clone> Nnf<V> {
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
}

impl<V: Ord + PartialOrd + Clone> BitOr for Nnf<V> {
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

impl<V: Ord + PartialOrd + Clone> BitAnd for Nnf<V> {
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

struct TseitinTransform<A, V> {
    aux_factory: A,
    clauses: BTreeSet<Nnf<V>>,
}

impl<A, V> TseitinTransform<A, V>
where
    A: FnMut() -> Nnf<V>,
    V: Ord + PartialOrd + Clone,
{
    pub fn new(aux_factory: A) -> Self {
        Self { aux_factory, clauses: BTreeSet::default() }
    }

    fn process_required(&mut self, root: Nnf<V>) {
        match root {
            var @ Nnf::Var(_, _) => {
                self.clauses.insert(Nnf::or([var]));
            }
            Nnf::And(children) | Nnf::Or(children) if children.len() == 1 => {
                self.process_required(children.into_iter().next().unwrap());
            }
            Nnf::Or(children) => {
                let or =
                    Nnf::or(children.into_iter().map(|child| self.process_node(child)));
                if or.has_inversions() {
                    return;
                }
                self.clauses.insert(or);
            }
            Nnf::And(children) => {
                for child in children {
                    self.process_required(child);
                }
            }
        }
    }

    fn process_node(&mut self, root: Nnf<V>) -> Nnf<V> {
        let node = match root {
            var @ Nnf::Var(_, _) => return var,
            Nnf::And(children) | Nnf::Or(children) if children.len() == 1 => {
                self.process_node(children.into_iter().next().unwrap())
            }
            Nnf::And(children) => {
                Nnf::and(children.into_iter().map(|child| self.process_node(child)))
            }
            Nnf::Or(children) => {
                Nnf::or(children.into_iter().map(|child| self.process_node(child)))
            }
        };

        let aux = (&mut self.aux_factory)();

        match node {
            Nnf::And(_) if node.has_inversions() => {
                self.clauses.insert(Nnf::or([!aux.clone()]));
            }
            Nnf::Or(_) if node.has_inversions() => {
                self.clauses.insert(Nnf::or([aux.clone()]));
            }

            Nnf::And(children) => {
                self.clauses.insert(Nnf::or(
                    children
                        .iter()
                        .map(|child| !child)
                        .chain(std::iter::once(aux.clone())),
                ));

                for child in children {
                    self.clauses.insert(Nnf::or([!aux.clone(), child]));
                }
            }

            Nnf::Or(children) => {
                self.clauses.insert(Nnf::or(
                    children.iter().cloned().chain(std::iter::once(!aux.clone())),
                ));

                for child in children {
                    self.clauses.insert(Nnf::or([!child, aux.clone()]));
                }
            }

            Nnf::Var(_, _) => unsafe { unreachable_unchecked() },
        }

        aux
    }

    pub fn transform(mut self, root: Nnf<V>) -> Nnf<V> {
        self.process_required(root);
        Nnf::And(self.clauses)
    }
}

#[cfg(test)]
mod test {
    use crate::evaluate::nnf::{Nnf, TseitinTransform};

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
    fn validate() {
        let sentence = or!(
            and!(var!("g", true), and!(var!("e", true), var!("f", true))),
            and!(
                or!(var!("a", false), var!("b", true)),
                and!(var!("c", true), var!("d", false))
            )
        );

        assert!(!sentence.is_cnf());

        let mut counter = 0;
        let transformer = TseitinTransform::new(|| {
            let name: &'static str = Box::leak(Box::new(format!("aux_{}", counter)));
            counter += 1;
            Nnf::Var(name, true)
        });

        let sentence = transformer.transform(sentence);

        assert!(sentence.is_cnf());
    }
}
