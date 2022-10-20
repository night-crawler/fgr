use dot_writer::{Attributes, DotWriter, NodeId, Scope};

use crate::parse::expression_node::ExpressionNode;

fn traverse(scope: &mut Scope, root: &ExpressionNode, counter: &mut usize) -> NodeId {
    let current = *counter;
    *counter += 1;

    match root {
        ExpressionNode::Leaf(filter) => {
            let label = format!("{filter}");
            let mut node = scope.node_named(current.to_string());
            let id = node.id();
            node.set_label(&label);

            id
        }
        ExpressionNode::And(left, right) => {
            let mut node = scope.node_named(current.to_string());
            node.set_label("AND");
            let id = node.id();
            drop(node);

            let left_child = traverse(scope, left, counter);
            let right_child = traverse(scope, right, counter);

            scope.edge(id.clone(), left_child);
            scope.edge(id.clone(), right_child);

            id
        }
        ExpressionNode::Or(left, right) => {
            let mut node = scope.node_named(current.to_string());
            node.set_label("OR");
            let id = node.id();
            drop(node);

            let left_child = traverse(scope, left, counter);
            let right_child = traverse(scope, right, counter);

            scope.edge(id.clone(), left_child);
            scope.edge(id.clone(), right_child);

            id
        }
        ExpressionNode::Not(exp) => {
            let mut node = scope.node_named(current.to_string());
            node.set_label("NOT");
            let id = node.id();
            drop(node);

            let left_child = traverse(scope, exp, counter);

            scope.edge(id.clone(), left_child);

            id
        }
    }
}

pub fn render_expression_tree(node: &ExpressionNode) -> String {
    let mut output_bytes = Vec::new();
    let mut writer = DotWriter::from(&mut output_bytes);
    writer.set_pretty_print(true);

    let mut scope = writer.digraph();

    traverse(&mut scope, node, &mut 0);

    drop(scope);

    unsafe { String::from_utf8_unchecked(output_bytes) }
}
