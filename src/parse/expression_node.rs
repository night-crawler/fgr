use crate::parse::filter::Filter;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ExpressionNode {
    Leaf(Filter),
    And(Box<ExpressionNode>, Box<ExpressionNode>),
    Or(Box<ExpressionNode>, Box<ExpressionNode>),
    Not(Box<ExpressionNode>),
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
