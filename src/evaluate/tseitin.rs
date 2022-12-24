use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hint::unreachable_unchecked;

use crate::evaluate::nnf::macros::or;
use crate::evaluate::nnf::Nnf;

pub struct TseitinTransform<A, V> {
    aux_factory: A,
    clauses: BTreeSet<Nnf<V>>,
}

impl<A, V> TseitinTransform<A, V>
where
    A: FnMut() -> Nnf<V>,
    V: Ord + Clone+ Debug,
{
    pub fn new(aux_factory: A) -> Self {
        Self { aux_factory, clauses: BTreeSet::default() }
    }

    fn process_required(&mut self, root: Nnf<V>) {
        match root {
            var @ Nnf::Var(_, _) => {
                self.clauses.insert(or!(var));
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
                children.into_iter().for_each(|child| self.process_required(child));
            }
        }
    }

    fn process_node(&mut self, root: Nnf<V>) -> Nnf<V> {
        let processed_node = match root {
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

        match processed_node {
            Nnf::And(_) if processed_node.has_inversions() => {
                self.clauses.insert(or!(!aux.clone()));
            }

            Nnf::Or(_) if processed_node.has_inversions() => {
                self.clauses.insert(or!(aux.clone()));
            }

            Nnf::And(children) => {
                self.clauses.insert(Nnf::or(
                    children
                        .iter()
                        .map(|child| !child)
                        .chain(std::iter::once(aux.clone())),
                ));

                for child in children {
                    self.clauses.insert(or!(!aux.clone(), child));
                }
            }

            Nnf::Or(children) => {
                self.clauses.insert(Nnf::or(
                    children.iter().cloned().chain(std::iter::once(!aux.clone())),
                ));

                for child in children {
                    self.clauses.insert(or!(!child, aux.clone()));
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
mod tests {
    use std::collections::BTreeSet;

    use crate::evaluate::nnf::macros::{and, or, var};
    use crate::evaluate::nnf::Nnf;
    use crate::evaluate::tseitin::TseitinTransform;
    use crate::parse::render::Render;

    fn transform(sentence: Nnf<&'static str>) -> Nnf<&'static str> {
        let mut counter = 0;
        let transformer = TseitinTransform::new(|| {
            let name: &'static str = Box::leak(Box::new(format!("aux_{}", counter)));
            counter += 1;
            var!(name)
        });

        transformer.transform(sentence)
    }

    #[test]
    fn test_sanity() {
        let sentence = or!(
            and!(var!("g", true), and!(var!("e", true), var!("f", true))),
            and!(
                or!(var!("a", false), var!("b", true)),
                and!(var!("c", true), var!("d", false))
            )
        );
        println!("{}", sentence.render());

        assert!(!sentence.is_cnf(), "Expression is not a cnf yet");

        let sentence = transform(sentence);
        assert!(sentence.is_cnf(), "Expression must be a cnf after transformation");
        println!("{}", sentence.render());

        let transformed_twice = transform(sentence.clone());
        assert!(
            transformed_twice.is_cnf(),
            "Double transformation must preserve the cnf form"
        );
        assert_eq!(sentence, transformed_twice);
    }

    #[test]
    fn test_tseitin_required_detection() {
        let [a, b, c] = [var!("a"), var!("b"), var!("c")];

        assert_eq!(transform(a.clone()), and!(or!(a.clone())));
        assert_eq!(
            transform(Nnf::<&str>::And(BTreeSet::new())),
            Nnf::And(BTreeSet::new())
        );

        assert_eq!(
            transform(Nnf::<&str>::Or(BTreeSet::new())),
            and!(Nnf::Or(BTreeSet::new()))
        );

        assert_eq!(transform(a.clone() | b.clone()), and!(a.clone() | b.clone()));

        assert_eq!(
            transform(and!(a.clone() | b.clone(), b.clone() | c.clone())),
            and!(a.clone() | b.clone(), b.clone() | c.clone())
        );

        assert_eq!(transform(and!(and!(or!(and!(!a.clone()))))), and!(or!(!a.clone())));
    }
}
