use dot_writer::{Attributes, DotWriter, NodeId, Scope};

use crate::parse::expression_node::{NnfNode, ExpressionNode};

pub trait Render {
    fn render(&self) -> String;
}

fn traverse_expression_node(scope: &mut Scope, root: &ExpressionNode, counter: &mut usize) -> NodeId {
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
            node.set_label("&");
            let id = node.id();
            drop(node);

            let left_child = traverse_expression_node(scope, left, counter);
            let right_child = traverse_expression_node(scope, right, counter);

            scope.edge(id.clone(), left_child);
            scope.edge(id.clone(), right_child);

            id
        }
        ExpressionNode::Or(left, right) => {
            let mut node = scope.node_named(current.to_string());
            node.set_label("|");
            let id = node.id();
            drop(node);

            let left_child = traverse_expression_node(scope, left, counter);
            let right_child = traverse_expression_node(scope, right, counter);

            scope.edge(id.clone(), left_child);
            scope.edge(id.clone(), right_child);

            id
        }
        ExpressionNode::Not(exp) => {
            let mut node = scope.node_named(current.to_string());
            node.set_label("!");
            let id = node.id();
            drop(node);

            let left_child = traverse_expression_node(scope, exp, counter);

            scope.edge(id.clone(), left_child);

            id
        }
    }
}



impl Render for ExpressionNode {
    fn render(&self) -> String {
        let mut output_bytes = Vec::new();
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);

        let mut scope = writer.digraph();

        traverse_expression_node(&mut scope, self, &mut 0);

        drop(scope);

        unsafe { String::from_utf8_unchecked(output_bytes) }
    }
}


fn traverse_cnf_node(scope: &mut Scope, root: &NnfNode, counter: &mut usize) -> NodeId {
    let current = *counter;
    *counter += 1;

    match root {
        NnfNode::Leaf(filter) => {
            let label = format!("{filter}");
            let mut node = scope.node_named(current.to_string());
            let id = node.id();
            node.set_label(&label);

            id
        }
        NnfNode::And(cnf_nodes) => {
            let mut graph_node = scope.node_named(current.to_string());
            graph_node.set_label("&");
            let node_id = graph_node.id();
            drop(graph_node);

            for cnf_node in cnf_nodes {
                let cnf_node_id = traverse_cnf_node(scope, cnf_node, counter);
                scope.edge(node_id.clone(), cnf_node_id);
            }

            node_id
        }
        NnfNode::Or(cnf_nodes) => {
            let mut graph_node = scope.node_named(current.to_string());
            graph_node.set_label("|");
            let node_id = graph_node.id();
            drop(graph_node);

            for cnf_node in cnf_nodes {
                let cnf_node_id = traverse_cnf_node(scope, cnf_node, counter);
                scope.edge(node_id.clone(), cnf_node_id);
            }

            node_id
        }

    }
}


impl Render for NnfNode {
    fn render(&self) -> String {
        let mut output_bytes = Vec::new();
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);

        let mut scope = writer.digraph();

        traverse_cnf_node(&mut scope, self, &mut 0);

        drop(scope);

        unsafe { String::from_utf8_unchecked(output_bytes) }
    }
}
