use crate::parse::filter::Filter;

#[derive(Debug, Eq, PartialEq)]
pub enum ExpressionNode {
    Leaf(Filter),
    And(Box<ExpressionNode>, Box<ExpressionNode>),
    Or(Box<ExpressionNode>, Box<ExpressionNode>),
    Not(Box<ExpressionNode>),
}

impl ExpressionNode {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn negate(&mut self) {
        match self {
            ExpressionNode::Leaf(filter) => filter.negate(),
            ExpressionNode::And(left, right) => {
                left.negate();
                right.negate();
            }
            ExpressionNode::Or(left, right) => {
                left.negate();
                right.negate();
            }
            ExpressionNode::Not(e) => e.negate(),
        }
    }

    /// Applies De Morgan's law to the original tree.
    /// We are trying to get rid of all possible OR expressions in favour of AND,
    /// wo we could join multiple nested ANDs with the root level.
    pub fn optimize(self) -> ExpressionNode {
        match self {
            filter @ ExpressionNode::Leaf(_) => filter,

            // Trivial case: the root node we have is AND. Optimize left and right branches and return
            // the node as is.
            ExpressionNode::And(left, right) => ExpressionNode::And(
                (*left).optimize().boxed(),
                (*right).optimize().boxed(),
            ),

            // The root not is OR. Apply the law in a manner:
            // if a == 1 || b == 37 {}
            // if !(a != 1 && b != 37) {}
            ExpressionNode::Or(left, right) => {
                let mut left = (*left).optimize().boxed();
                let mut right = (*right).optimize().boxed();
                left.negate();
                right.negate();

                ExpressionNode::Not(ExpressionNode::And(left, right).boxed())
            }

            // The root not is NOT. We need to check the underlying expression first:
            ExpressionNode::Not(ex) => {
                match *ex {
                    // The child of NOT expression is a Leaf Node. In this case we flip the sign
                    // and bail.
                    mut child @ ExpressionNode::Leaf(_) => {
                        child.negate();
                        child
                    }

                    // We've got NOT(AND(left, right)) expression. We can't do anything,
                    // so optimize branches and return.
                    ExpressionNode::And(left, right) => {
                        // if !(a == 1 && b == 37) {}
                        // if a != 1 || b != 37 {}
                        let left = (*left).optimize().boxed();
                        let right = (*right).optimize().boxed();
                        ExpressionNode::Not(
                            ExpressionNode::And(left, right).boxed(),
                        )
                    }

                    // We are handling the NOT(OR(left, right)) case. We can make it AND using the law:
                    // if !(a == 1 || b == 37) {}
                    // if a != 1 && b != 37 {}
                    ExpressionNode::Or(left, right) => {
                        let left = (*left).optimize().boxed();
                        let mut right = (*right).optimize().boxed();
                        // left.negate();
                        right.negate();

                        ExpressionNode::And(
                            ExpressionNode::Not(left).boxed(),
                            right,
                        )
                    }

                    // NOT(NOT(expression)) case. We just return the underlying expression.
                    // if !(!(a == 3 && b == 3)) {}
                    // if !(a != 3 || b != 3) {}
                    // if a == 3 && b == 3 {}
                    ExpressionNode::Not(expression) => (*expression).optimize(),
                }
            }
        }
    }
}
