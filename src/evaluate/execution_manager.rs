use std::cmp::Ordering;
use std::collections::BTreeMap;

use nnf::nnf::Nnf;
use nnf::parse_tree::ExpressionNode;
use nnf::tseitin::TseitinTransform;
use nnf::var;

use crate::errors::GenericError;
use crate::parse::filter::Filter;

pub struct ExecutionManager {
    pub(crate) filters: Vec<Filter>,
    pub(crate) root: Nnf<FilterVar>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum FilterVar {
    Var { id: usize, weight: usize },
    Aux(usize),
}

impl Default for FilterVar {
    fn default() -> Self {
        FilterVar::Aux(0)
    }
}

impl PartialOrd<Self> for FilterVar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (
                Self::Var { id: l_id, weight: l_weight },
                Self::Var { id: r_id, weight: r_weight },
            ) => (l_weight, l_id).partial_cmp(&(r_weight, r_id)),

            (Self::Var { .. }, Self::Aux(_)) => Some(Ordering::Less),
            (Self::Aux(_), Self::Var { .. }) => Some(Ordering::Greater),
            (Self::Aux(l_id), Self::Aux(r_id)) => l_id.partial_cmp(r_id),
        }
    }
}

impl Ord for FilterVar {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl FilterVar {
    fn new_var(id: usize, weight: usize) -> Self {
        Self::Var { id, weight }
    }

    fn new_aux(id: usize) -> Self {
        Self::Aux(id)
    }
}

impl ExecutionManager {
    pub fn new(root: ExpressionNode<Filter>) -> Self {
        let mut filters = vec![];
        let root = Self::map(root, &mut filters);
        let counter = filters.len();

        let (root, _) = Self::tseitin_transform(root, counter);

        ExecutionManager { filters, root }
    }

    fn map(root: ExpressionNode<Filter>, filters: &mut Vec<Filter>) -> Nnf<FilterVar> {
        match root {
            ExpressionNode::Leaf(filter) => {
                // todo: optimize negated filters & uniqueness
                let var = var!(FilterVar::new_var(filters.len(), filter.weight()), true);
                filters.push(filter);
                var
            }
            ExpressionNode::And(left, right) => {
                Self::map(*left, filters) & Self::map(*right, filters)
            }
            ExpressionNode::Or(left, right) => {
                Self::map(*left, filters) | Self::map(*right, filters)
            }
            ExpressionNode::Not(_) => unimplemented!("Must never happen"),
        }
    }

    fn tseitin_transform(root: Nnf<FilterVar>, mut counter: usize) -> (Nnf<FilterVar>, BTreeMap<Nnf<FilterVar>, Nnf<FilterVar>>) {
        let mut aux_var_map = BTreeMap::new();
        let transformer = TseitinTransform::new(|node| {
            let var = var!(FilterVar::new_aux(counter));
            aux_var_map.insert(var.clone(), node.clone());

            counter += 1;
            var
        });

        let root = transformer.transform(root);

        (root, aux_var_map)
    }

    pub fn prepare_execution_plan(&self) -> Result<(), GenericError> {
        Ok(())
    }
}

trait ComputationWeight {
    fn compute_weight(&self) -> usize;
}

impl ComputationWeight for Nnf<FilterVar> {
    fn compute_weight(&self) -> usize {
        match self {
            Nnf::Var(filter_var, _) => match filter_var {
                FilterVar::Var { weight, .. } => *weight,
                FilterVar::Aux(_) => 0,
            },
            Nnf::And(children) => {
                children.iter().map(|child| child.compute_weight()).sum()
            }
            Nnf::Or(children) => {
                children.iter().map(|child| child.compute_weight()).max().unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nnf::{or, var};
    use nnf::traits::Render;

    use crate::evaluate::execution_manager::ExecutionManager;
    use crate::evaluate::execution_manager::FilterVar;
    use crate::parse::parse_root;

    #[test]
    fn test_simple_expression() {
        let expression = r#"
        name = lol* and (
            name = *lol and
            size >= 100K
        ) or not(
            contains != *penguins* or
            type = vid or not (
                perms >= 777 or
                user > 0
            )
        )
        "#;

        // let expression = "bool=true and (bool=false or (bool=true and bool=false))";
        let expression_node = parse_root(expression).unwrap();
        let nnf = expression_node.to_nnf();
        println!("{}", nnf.render());

        let mapper = ExecutionManager::new(nnf);
        println!("{}", mapper.render());
        mapper.prepare_execution_plan().unwrap();
    }

    #[test]
    fn test_ord() {
        assert!(var!(FilterVar::Aux(0), true) < var!(FilterVar::Aux(2), true));
        assert!(var!(FilterVar::Aux(0), false) < var!(FilterVar::Aux(2), true));
        assert!(var!(FilterVar::Aux(0), false) < var!(FilterVar::Aux(0), true));

        assert!(var!(FilterVar::Aux(0), false) > var!(FilterVar::new_var(0, 1)));
        assert!(var!(FilterVar::Aux(0), true) > var!(FilterVar::new_var(0, 1)));

        assert!(
            or!(var!(FilterVar::Aux(0), true), var!(FilterVar::Aux(2), true))
                > or!(var!(FilterVar::Aux(0), false), var!(FilterVar::new_var(0, 1)))
        );
    }
}
