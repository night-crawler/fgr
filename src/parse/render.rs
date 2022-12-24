use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

use dot_writer::{Attributes, DotWriter, NodeId, Scope};

use crate::evaluate::nnf::Nnf;
use crate::evaluate::execution_manager::{FilterVar, ExecutionManager};
use crate::parse::expression_node::ExpressionNode;

pub trait Render {
    fn render(&self) -> String;
}

fn traverse_expression_node(
    scope: &mut Scope,
    root: &ExpressionNode,
    counter: &mut usize,
) -> NodeId {
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

            let left_child = traverse_expression_node(scope, left, counter);
            let right_child = traverse_expression_node(scope, right, counter);

            scope.edge(id.clone(), left_child);
            scope.edge(id.clone(), right_child);

            id
        }
        ExpressionNode::Or(left, right) => {
            let mut node = scope.node_named(current.to_string());
            node.set_label("OR");
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

fn traverse_nnf_node<V: Ord+ Debug>(
    scope: &mut Scope,
    root: &Nnf<V>,
    var_label_mapper: impl Fn(&Nnf<V>) -> String,
) {
    let mut counter = 0;
    let mut map = BTreeMap::new();
    let mut stack: Vec<(&Nnf<V>, Option<NodeId>)> = vec![(root, None)];

    while let Some((root, parent)) = stack.pop() {
        let current = counter;

        let node_id = match root {
            var @ Nnf::Var(_, _) => {
                let var_node_id = map.entry(var).or_insert_with(|| {
                    let label = var_label_mapper(var);
                    let mut node = scope.node_named(current.to_string());
                    node.set_label(&label);
                    counter += 1;
                    node.id()
                });
                var_node_id.clone()
            }
            Nnf::And(children) => {
                let mut and_node = scope.node_named(current.to_string());
                and_node.set_label("AND");
                let node_id = and_node.id();
                drop(and_node);

                for child in children {
                    stack.push((child, Some(node_id.clone())));
                }

                counter += 1;
                node_id
            }
            Nnf::Or(children) => {
                let mut or_node = scope.node_named(current.to_string());
                or_node.set_label("OR");
                let node_id = or_node.id();

                for child in children {
                    stack.push((child, Some(node_id.clone())));
                }
                counter += 1;
                node_id
            }
        };

        if let Some(parent_node_id) = parent {
            scope.edge(parent_node_id, node_id);
        }
    }
}

impl<T: Ord + Display+ Debug> Render for Nnf<T> {
    fn render(&self) -> String {
        let mut output_bytes = Vec::new();
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);
        let mut scope = writer.digraph();

        traverse_nnf_node(&mut scope, self,  |node| {
            if let Nnf::Var(name, value) = node {
                let prefix = if !*value { "¬" } else { "" };
                format!("{prefix}{name}")
            } else {
                unimplemented!()
            }
        });

        drop(scope);

        unsafe { String::from_utf8_unchecked(output_bytes) }
    }
}

impl Render for ExecutionManager {
    fn render(&self) -> String {
        let mut output_bytes = Vec::new();
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);
        let mut scope = writer.digraph();

        traverse_nnf_node(&mut scope, &self.root, |node| {
            if let Nnf::Var(filter_var, value) = node {
                let prefix = if !value { "¬" } else { "" };

                match filter_var {
                    &FilterVar::Var { id, weight } => {
                        let filter = &self.filters[id];
                        format!("{prefix}{filter} [{weight}]")
                    }
                    FilterVar::Aux(id) => {
                        format!("{prefix}aux_{id}")
                    }
                }

            } else {
                unimplemented!()
            }
        });

        drop(scope);

        unsafe { String::from_utf8_unchecked(output_bytes) }
    }
}
