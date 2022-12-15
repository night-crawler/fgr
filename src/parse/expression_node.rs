use crate::parse::filter::Filter;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ExpressionNode {
    Leaf(Filter),
    And(Box<ExpressionNode>, Box<ExpressionNode>),
    Or(Box<ExpressionNode>, Box<ExpressionNode>),
    Not(Box<ExpressionNode>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CnfNode {
    Leaf(Filter),
    And(Vec<CnfNode>),
    Or(Vec<CnfNode>),
}

impl CnfNode {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (left @ CnfNode::Leaf(_), right @ CnfNode::Leaf(_)) => {
                CnfNode::And(vec![left, right])
            }
            (left @ CnfNode::Leaf(_), CnfNode::And(mut nodes)) => {
                nodes.push(left);
                CnfNode::And(nodes)
            }
            (left @ CnfNode::Leaf(_), CnfNode::Or(nodes)) => {
                let mut result = vec![];
                for node in nodes {
                    result.push(node.and(left.clone()));
                }

                CnfNode::Or(result)
            }

            (left @ CnfNode::And(_), right @ CnfNode::Leaf(_)) => right.and(left),
            (CnfNode::And(mut nodes_left), CnfNode::And(nodes_right)) => {
                nodes_left.extend(nodes_right);
                CnfNode::And(nodes_left)
            }
            (CnfNode::And(and_nodes), CnfNode::Or(or_nodes)) => {
                let mut distribution = vec![];
                for or_node in or_nodes {
                    let mut nodes = and_nodes.clone();
                    nodes.push(or_node);
                    distribution.push(CnfNode::And(nodes));
                }
                CnfNode::Or(distribution)
            }

            (left @ CnfNode::Or(_), right @ CnfNode::Leaf(_)) => right.and(left),

            (left @ CnfNode::Or(_), right @ CnfNode::And(_)) => right.and(left),

            (CnfNode::Or(left_nodes), CnfNode::Or(right_nodes)) => {
                let mut distribution = vec![];
                for left_node in left_nodes {
                    for right_node in right_nodes.clone() {
                        distribution.push(left_node.clone().and(right_node));
                    }
                }
                CnfNode::Or(distribution)
            }
        }
    }
}

impl From<ExpressionNode> for CnfNode {
    fn from(expression_node: ExpressionNode) -> Self {
        match expression_node {
            ExpressionNode::Leaf(filter) => Self::Leaf(filter),
            ExpressionNode::And(left, right) => {
                let left: CnfNode = (*left).into();
                let right: CnfNode = (*right).into();

                match (left, right) {
                    (left @ CnfNode::Leaf(_), right @ CnfNode::Leaf(_)) => {
                        CnfNode::And(vec![left, right])
                    }

                    (leaf @ CnfNode::Leaf(_), CnfNode::And(mut nodes))
                    | (CnfNode::And(mut nodes), leaf @ CnfNode::Leaf(_)) => {
                        nodes.push(leaf);
                        CnfNode::And(nodes)
                    }

                    (CnfNode::And(mut left_nodes), CnfNode::And(right_nodes)) => {
                        left_nodes.extend(right_nodes);
                        CnfNode::And(left_nodes)
                    }

                    (left, right) => {
                        CnfNode::And(vec![left, right])
                    }
                }
            }
            ExpressionNode::Or(left, right) => {
                let left: CnfNode = (*left).into();
                let right: CnfNode = (*right).into();

                match (left, right) {
                    (left @ CnfNode::Leaf(_), right @ CnfNode::Leaf(_)) => {
                        CnfNode::Or(vec![left, right])
                    }
                    (leaf @ CnfNode::Leaf(_), CnfNode::Or(mut nodes))
                    | (CnfNode::Or(mut nodes), leaf @ CnfNode::Leaf(_)) => {
                        nodes.push(leaf);
                        CnfNode::Or(nodes)
                    }
                    _ => unimplemented!(),
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

    fn distribute_or(left: Self, right: Self) -> Self {
        match (left, right) {
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
                Self::distribute_or(right, left)
            }
            (left @ Self::Leaf(_), right @ Self::And(_, _)) => {
                Self::distribute_or(right, left)
            }

            // do nothing
            (left, right) => Self::Or(left.into(), right.into()),
        }
    }

    pub fn to_cnf(self) -> Self {
        match self {
            Self::Not(_) => unimplemented!("It must never happen"),
            expression_node @ Self::Leaf(_) => expression_node,
            Self::And(left, right) => {
                Self::And(left.to_cnf().into(), right.to_cnf().into())
            }
            Self::Or(left, right) => {
                let left = left.to_cnf();
                let right = right.to_cnf();

                Self::distribute_or(left, right)
            }
        }
    }

    pub fn simplify_not(self) -> Self {
        match self {
            expression_node @ ExpressionNode::Leaf(_) => expression_node,
            Self::And(left, right) => {
                Self::And(left.simplify_not().into(), right.simplify_not().into())
            }
            Self::Or(left, right) => {
                Self::Or(left.simplify_not().into(), right.simplify_not().into())
            }

            Self::Not(node) => match *node {
                Self::Leaf(mut filter) => {
                    filter.negate();
                    Self::Leaf(filter)
                }
                Self::And(left, right) => Self::Or(
                    left.negate().simplify_not().into(),
                    right.negate().simplify_not().into(),
                ),

                Self::Or(left, right) => Self::And(
                    left.negate().simplify_not().into(),
                    right.negate().simplify_not().into(),
                ),
                Self::Not(expression_node) => expression_node.simplify_not(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::evaluate::traits::Evaluate;
    use crate::parse::expression_node::CnfNode;
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

                    let simplified = node.simplify_not();
                    let result = simplified.evaluate(&DirEntryMock::default()).unwrap();
                    assert_eq!(
                        result, expected,
                        "Simplification failed for expression `{expression}`"
                    );

                    let cnf = simplified.to_cnf();
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

        // let expression = "bool=false and (bool=false and bool=true) or not(bool=true or bool=false or not (bool=false or bool=true))";
        let expression = "bool=true and (bool=false or (bool=true and bool=false))";
        let expression_node = parse_root(expression).unwrap();
        println!("{}", expression_node.render());

        let cnf = expression_node.simplify_not().to_cnf().to_cnf().to_cnf();
        println!("{}", cnf.render());

        let q = CnfNode::from(cnf);
        println!("{}", q.render());
    }
}
