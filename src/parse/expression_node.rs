use crate::parse::filter::Filter;

#[derive(Debug, Eq, PartialEq)]
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
            expression_node @ Self::And(_, _) => Self::Not(expression_node.into()),
            expression_node @ Self::Or(_, _) => Self::Not(expression_node.into()),
            Self::Not(expression_node) => *expression_node,
        }
    }

    pub fn to_cnf(self) -> ExpressionNode {
        match self {
            expression_node @ ExpressionNode::Leaf(_) => expression_node,
            Self::And(left, right) => {
                Self::And(left.to_cnf().into(), right.to_cnf().into())
            }
            Self::Or(left, right) => Self::Not(
                Self::And(left.to_cnf().negate().into(), right.to_cnf().negate().into())
                    .into(),
            ),
            Self::Not(node) => match *node {
                Self::Leaf(mut filter) => {
                    filter.negate();
                    Self::Leaf(filter)
                }
                Self::And(left, right) => Self::Not(
                    Self::And(left.to_cnf().into(), right.to_cnf().into()).into(),
                ),

                Self::Or(left, right) => Self::And(
                    left.to_cnf().negate().into(),
                    right.to_cnf().negate().into(),
                ),
                Self::Not(expression_node) => expression_node.to_cnf(),
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

                    let cnf_node = node.to_cnf();
                    let result = cnf_node.evaluate(&DirEntryMock::default()).unwrap();
                    assert_eq!(result, expected, "Failed for `{expression}`");
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

    #[test]
    fn test_1() {
        use crate::parse::render::render_expression_tree;
        let expression = "bool=true or bool=false or bool=false or bool=true";
        let expression_node = parse_root(expression).unwrap();
        println!("{}", render_expression_tree(&expression_node));

        let cnf = expression_node.to_cnf();
        println!("{}", render_expression_tree(&cnf));
    }
}
