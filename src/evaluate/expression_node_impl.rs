use crate::errors::GenericError;
use crate::evaluate::traits::Evaluate;
use crate::parse::expression_node::ExpressionNode;
use crate::walk::traits::DirEntryWrapperExt;

impl<E: DirEntryWrapperExt> Evaluate<E> for ExpressionNode {
    fn evaluate(&self, entry: &E) -> Result<bool, GenericError> {
        match self {
            ExpressionNode::Leaf(filter) => filter.evaluate(entry),
            ExpressionNode::And(left, right) => {
                Ok(left.evaluate(entry)? && right.evaluate(entry)?)
            }
            ExpressionNode::Or(left, right) => {
                Ok(left.evaluate(entry)? || right.evaluate(entry)?)
            }
            ExpressionNode::Not(exp) => Ok(!exp.evaluate(entry)?),
        }
    }
}
