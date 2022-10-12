use std::hint::unreachable_unchecked;

use crate::parse::expression_node::ExpressionNode;
use crate::parse::filter::Filter;

#[derive(Debug, Eq, PartialEq)]
pub enum AstNode {
    And { nodes: Vec<AstNode> },
    Or { nodes: Vec<AstNode> },
    Not { node: Box<AstNode> },
    Leaf { filter: Filter },
}

impl AstNode {
    pub fn flatten(self) -> Self {
        match self {
            Self::And { .. } => self,
            Self::Or { .. } => unsafe { unreachable_unchecked() },
            Self::Not { node } => {
                // if !(a == 1 && b == 3 && a == 11 && b == 13) {}
                // if a != 1 || b != 3 || a != 11 || b != 13 {}
                match *node {
                    AstNode::And { mut nodes } => {
                        nodes.iter_mut().for_each(|n| n.negate());
                        Self::Or { nodes }
                    }
                    f => f,
                }
            }
            Self::Leaf { .. } => self,
        }
    }

    pub fn negate(&mut self) {
        match self {
            Self::And { nodes } | Self::Or { nodes } => {
                nodes.iter_mut().for_each(|node| node.negate())
            }
            Self::Not { node } => node.negate(),
            Self::Leaf { filter } => filter.negate(),
        }
    }
}

fn flatten_and(node: AstNode) -> Vec<AstNode> {
    let mut res = vec![];
    match node {
        AstNode::And { nodes } => res.extend(nodes),
        f => res.push(f),
    }
    res
}

impl From<ExpressionNode> for AstNode {
    fn from(expression: ExpressionNode) -> Self {
        match expression {
            ExpressionNode::Leaf(filter) => AstNode::Leaf { filter },
            ExpressionNode::And(left, right) => {
                let nodes = flatten_and((*left).into())
                    .into_iter()
                    .chain(flatten_and((*right).into()))
                    .collect();

                AstNode::And { nodes }
            }

            // SAFETY: there must be no remaining OR expressions
            ExpressionNode::Or(_, _) => unsafe { unreachable_unchecked() },
            ExpressionNode::Not(child_expression) => {
                AstNode::Not { node: Box::new((*child_expression).into()) }
            }
        }
    }
}
