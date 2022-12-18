use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::parse::filter::Filter;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ExpressionNode {
    Leaf(Filter),
    And(Box<ExpressionNode>, Box<ExpressionNode>),
    Or(Box<ExpressionNode>, Box<ExpressionNode>),
    Not(Box<ExpressionNode>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NnfNode {
    Leaf(Filter),
    And(Vec<NnfNode>),
    Or(Vec<NnfNode>),
}

impl Display for NnfNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NnfNode::Leaf(filter) => {
                write!(f, "{filter}")
            }
            NnfNode::And(nodes) => {
                let a = nodes.iter().map(|node| node.to_string()).join(" ∧ ");
                write!(f, "({a})")
            }
            NnfNode::Or(nodes) => {
                let a = nodes.iter().map(|node| node.to_string()).join(" ∨ ");
                write!(f, "({a})")
            }
        }
    }
}

impl NnfNode {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (left @ NnfNode::Leaf(_), right @ NnfNode::Leaf(_)) => {
                NnfNode::And(vec![left, right])
            }
            (left @ NnfNode::Leaf(_), NnfNode::And(mut nodes)) => {
                nodes.push(left);
                NnfNode::And(nodes)
            }
            (left @ NnfNode::Leaf(_), NnfNode::Or(nodes)) => {
                let mut result = vec![];
                for node in nodes {
                    result.push(node.and(left.clone()));
                }

                NnfNode::Or(result)
            }

            (left @ NnfNode::And(_), right @ NnfNode::Leaf(_)) => right.and(left),
            (NnfNode::And(mut nodes_left), NnfNode::And(nodes_right)) => {
                nodes_left.extend(nodes_right);
                NnfNode::And(nodes_left)
            }
            (NnfNode::And(and_nodes), NnfNode::Or(or_nodes)) => {
                let mut distribution = vec![];
                for or_node in or_nodes {
                    let mut nodes = and_nodes.clone();
                    nodes.push(or_node);
                    distribution.push(NnfNode::And(nodes));
                }
                NnfNode::Or(distribution)
            }

            (left @ NnfNode::Or(_), right @ NnfNode::Leaf(_)) => right.and(left),

            (left @ NnfNode::Or(_), right @ NnfNode::And(_)) => right.and(left),

            (NnfNode::Or(left_nodes), NnfNode::Or(right_nodes)) => {
                let mut distribution = vec![];
                for left_node in left_nodes {
                    for right_node in right_nodes.clone() {
                        distribution.push(left_node.clone().and(right_node));
                    }
                }
                NnfNode::Or(distribution)
            }
        }
    }
}

impl From<ExpressionNode> for NnfNode {
    fn from(expression_node: ExpressionNode) -> Self {
        match expression_node {
            ExpressionNode::Leaf(filter) => Self::Leaf(filter),
            ExpressionNode::And(left, right) => {
                let left: NnfNode = (*left).into();
                let right: NnfNode = (*right).into();

                match (left, right) {
                    (left @ NnfNode::Leaf(_), right @ NnfNode::Leaf(_)) => {
                        NnfNode::And(vec![left, right])
                    }

                    (leaf @ NnfNode::Leaf(_), NnfNode::And(mut nodes))
                    | (NnfNode::And(mut nodes), leaf @ NnfNode::Leaf(_)) => {
                        nodes.push(leaf);
                        NnfNode::And(nodes)
                    }

                    (NnfNode::And(mut left_nodes), NnfNode::And(right_nodes)) => {
                        left_nodes.extend(right_nodes);
                        NnfNode::And(left_nodes)
                    }

                    (left, right) => NnfNode::And(vec![left, right]),
                }
            }
            ExpressionNode::Or(left, right) => {
                let left: NnfNode = (*left).into();
                let right: NnfNode = (*right).into();

                match (left, right) {
                    (left @ NnfNode::Leaf(_), right @ NnfNode::Leaf(_)) => {
                        NnfNode::Or(vec![left, right])
                    }
                    (leaf @ NnfNode::Leaf(_), NnfNode::Or(mut nodes))
                    | (NnfNode::Or(mut nodes), leaf @ NnfNode::Leaf(_)) => {
                        nodes.push(leaf);
                        NnfNode::Or(nodes)
                    }
                    wtf => {
                        unimplemented!("Not implemented for {wtf:?}")
                    }
                }
            }
            ExpressionNode::Not(_) => unimplemented!("Must never happen"),
        }
    }
}

impl ExpressionNode {
    pub fn negate(mut self) -> Self {
        match self {
            Self::Leaf(ref mut filter) => {
                filter.negate();
                self
            }
            Self::And(left, right) => {
                Self::Or(left.negate().into(), right.negate().into())
            }
            Self::Or(left, right) => {
                Self::And(left.negate().into(), right.negate().into())
            }
            Self::Not(expression_node) => *expression_node,
        }
    }

    fn distribute_or(left: Self, right: Self) -> Result<Self, Self> {
        Ok(match (left, right) {
            (Self::And(left_left, left_right), Self::And(right_left, right_right)) => {
                let pair1 = Self::Or(left_left.clone(), right_left.clone()).into();
                let pair2 = Self::Or(left_left.clone(), right_right.clone()).into();

                let pair3 = Self::Or(left_right.clone(), right_left.clone()).into();
                let pair4 = Self::Or(left_right.clone(), right_right.clone()).into();

                Self::And(Self::And(pair1, pair2).into(), Self::And(pair3, pair4).into())
            }
            (Self::And(left_left, left_right), Self::Or(right_left, right_right)) => {
                let triple1 = Self::Or(
                    left_left,
                    Self::Or(right_left.clone(), right_right.clone()).into(),
                )
                .into();

                let triple2 =
                    Self::Or(left_right, Self::Or(right_left, right_right).into()).into();

                Self::And(triple1, triple2)
            }
            (Self::And(left_left, left_right), right @ Self::Leaf(_)) => {
                let tuple1 = Self::Or(left_left, right.clone().into()).into();
                let tuple2 = Self::Or(left_right, right.into()).into();

                Self::And(tuple1, tuple2)
            }

            (left @ Self::Or(_, _), right @ Self::And(_, _)) => {
                Self::distribute_or(right, left).unwrap()
            }
            (left @ Self::Leaf(_), right @ Self::And(_, _)) => {
                Self::distribute_or(right, left).unwrap()
            }

            (left, right) => return Err(Self::Or(left.into(), right.into())),
        })
    }

    pub fn to_cnf(self) -> Self {
        let mut count = 1;
        let mut root = self;

        while count > 0 {
            count = 0;
            root = root.to_cnf_step(&mut count);
        }

        root
    }

    fn to_cnf_step(self, count: &mut u8) -> Self {
        match self {
            Self::Not(_) => unimplemented!("It must never happen"),
            expression_node @ Self::Leaf(_) => expression_node,
            Self::And(left, right) => {
                Self::And(left.to_cnf_step(count).into(), right.to_cnf_step(count).into())
            }
            Self::Or(left, right) => {
                let left = left.to_cnf_step(count);
                let right = right.to_cnf_step(count);

                match Self::distribute_or(left, right) {
                    Ok(node) => {
                        *count += 1;
                        node
                    }
                    Err(node) => node,
                }
            }
        }
    }

    pub fn to_nnf(self) -> Self {
        match self {
            expression_node @ ExpressionNode::Leaf(_) => expression_node,
            Self::And(left, right) => {
                Self::And(left.to_nnf().into(), right.to_nnf().into())
            }
            Self::Or(left, right) => {
                Self::Or(left.to_nnf().into(), right.to_nnf().into())
            }

            Self::Not(node) => match *node {
                Self::Leaf(mut filter) => {
                    filter.negate();
                    Self::Leaf(filter)
                }
                Self::And(left, right) => Self::Or(
                    left.negate().to_nnf().into(),
                    right.negate().to_nnf().into(),
                ),

                Self::Or(left, right) => Self::And(
                    left.negate().to_nnf().into(),
                    right.negate().to_nnf().into(),
                ),
                Self::Not(expression_node) => expression_node.to_nnf(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::evaluate::traits::Evaluate;
    use crate::parse::parse_root;
    use crate::test_utils::DirEntryMock;

    macro_rules! cnf_test {
        ($fn_name:ident, $template:literal, $len:expr) => {
            #[test]
            fn $fn_name() {
                let combinations =
                    [true, false].iter().copied().combinations_with_replacement($len);

                for combination in combinations {
                    let expression = interpolate($template, &combination);
                    let node = parse_root(&expression).unwrap();
                    let expected = node.evaluate(&DirEntryMock::default()).unwrap();

                    let nnf = node.to_nnf();
                    let result = nnf.evaluate(&DirEntryMock::default()).unwrap();
                    assert_eq!(
                        result, expected,
                        "NNF failed for expression `{expression}`"
                    );

                    let cnf = nnf.to_cnf();
                    let result = cnf.evaluate(&DirEntryMock::default()).unwrap();
                    assert_eq!(
                        result, expected,
                        "CNF failed for expression `{expression}`"
                    );
                }
            }
        };
    }

    fn interpolate(template: &str, values: &[bool]) -> String {
        let mut result = template.to_string();
        for (index, &value) in values.iter().enumerate() {
            result = result.replace(&format!(":{index}"), &value.to_string());
        }
        result
    }

    cnf_test!(plain_or, "bool=:0 or bool=:0", 1);
    cnf_test!(plain_and, "bool=:0 and bool=:0", 1);
    cnf_test!(plain_not, "not bool=:0", 1);
    cnf_test!(plain_simple, "bool=:0", 1);

    cnf_test!(not_or, "not (bool=:0 or bool=:1)", 2);
    cnf_test!(not_and, "not (bool=:0 and bool=:1)", 2);

    cnf_test!(not_not_and, "not (not (bool=:0 and bool=:1))", 2);
    cnf_test!(not_not_or, "not (not (bool=:0 or bool=:1))", 2);

    cnf_test!(nested_or_1, "(bool=:0 and bool=:1) or (bool=:2 or bool=:3)", 4);
    cnf_test!(not_nested_or_1, "not ((bool=:0 and bool=:1) or (bool=:2 or bool=:3))", 4);

    cnf_test!(nested_or_2, "bool=:0 or bool=:1 or bool=:2 or bool=:3 or bool=:4", 5);
    cnf_test!(
        not_nested_or_2,
        "not (bool=:0 or bool=:1 or bool=:2 or bool=:3 or bool=:4)",
        5
    );

    cnf_test!(
        qweqwe,
        "bool=:0 and (bool=:1 and bool=:2) or not(bool=:3 or bool=:4 or not (bool=:5 or bool=:6))",
        7
    );

    #[test]
    fn test_1() {
        use crate::parse::render::Render;

        // let expression = "name=a* and (mtime>now-1d and perms<777) or not(name=sample or name=lol or not (size>1B or atime<now-1d))";
        let expression = "bool=false and (bool=false and bool=true) or not(bool=true or bool=false or not (bool=false or bool=true))";

        // let expression = "bool=true and (bool=false or (bool=true and bool=false))";
        let expression_node = parse_root(expression).unwrap();
        println!("{}", expression_node.render());

        let cnf = expression_node.to_nnf();
        println!("{}", cnf.render());

        // let q = CnfNode::from(cnf);
        // println!("{}", q.render());
        // println!("{}", q.to_string());
    }
}
