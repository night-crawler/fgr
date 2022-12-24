use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hint::unreachable_unchecked;

use dot_writer::DotWriter;
use itertools::Itertools;
use splr::types::{CNFDescription, Instantiate};
use splr::{Config, SatSolverIF, Solver};

use crate::errors::GenericError;
use crate::evaluate::nnf::macros::var;
use crate::evaluate::nnf::Nnf;
use crate::evaluate::tseitin::TseitinTransform;
use crate::parse::expression_node::ExpressionNode;
use crate::parse::filter::Filter;
use crate::parse::render::Render;

pub struct ExecutionManager {
    pub(crate) filters: Vec<Filter>,
    pub(crate) root: Nnf<FilterVar>,
    counter: usize,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum FilterVar {
    Var { id: usize, weight: usize },
    Aux(usize),
}

impl FilterVar {
    fn id(&self) -> usize {
        *match self {
            FilterVar::Var { id, .. } | FilterVar::Aux(id) => id,
        }
    }

    fn is_aux(&self) -> bool {
        match self {
            FilterVar::Var { .. } => false,
            FilterVar::Aux(_) => true,
        }
    }

    fn is_var(&self) -> bool {
        !self.is_aux()
    }

    fn weight(&self) -> usize {
        match self {
            &FilterVar::Var { weight, .. } => weight,
            FilterVar::Aux(_) => unimplemented!("Must not be called to aux vars"),
        }
    }
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

            (Self::Var { .. }, Self::Aux(_)) => Some(Ordering::Greater),
            (Self::Aux(_), Self::Var { .. }) => Some(Ordering::Less),
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

#[derive(Default)]
struct Graph {
    edges: HashMap<i32, HashSet<i32>>,
    num_vars: usize,
}

impl Graph {
    fn new(num_vars: usize) -> Self {
        Self { edges: HashMap::default(), num_vars }
    }

    fn add_all(&mut self, nodes: Vec<i32>) {
        for left in 0..nodes.len() {
            for right in left + 1..nodes.len() {
                let left_node = nodes[left];
                let right_node = nodes[right];
                self.add_edge(left_node, right_node);
            }
        }
    }

    fn add_edge(&mut self, from: i32, to: i32) {
        self.edges.entry(from).or_default().insert(to);
        self.edges.entry(to).or_default().insert(from);
    }

    fn path_to_bitmask(&self, path: &[i32]) -> usize {
        let mut bits = 0usize;

        let path_vars = path.iter().map(|&var| {
            if var > 0 {
                var as usize
            } else {
                var.unsigned_abs() as usize + self.num_vars
            }
        });

        for var in path_vars {
            bits |= 1 << var;
        }

        bits
    }

    fn dfs(&self, path: &mut Vec<i32>, taken_vars: &mut HashSet<i32>, node: i32) {
        if !taken_vars.insert(node.abs()) {
            return;
        }

        if taken_vars.len() == self.num_vars {
            println!("{:?}, {:?}", path, taken_vars);
        }

        path.push(node);

        if let Some(adjacent) = self.edges.get(&node) {
            for neighbour in adjacent {
                if taken_vars.contains(&neighbour.abs()) {
                    continue;
                }

                let negative = self.edges.get(&(-neighbour));
                let positive = self.edges.get(neighbour);

                match (negative, positive) {
                    // we skip the current node neighbour iff it has its negation, i.e.
                    // if the node = 4 and neighbour = 7, then if we have also neighbour -7
                    // on the same level, we take the intersection of 7 & -7 children and skip
                    // the current neighbour
                    (Some(negative), Some(positive))
                        if adjacent.contains(&(-neighbour)) =>
                    {
                        taken_vars.insert(neighbour.abs());

                        for next_node in negative
                            .iter()
                            .filter(|negative_node| positive.contains(negative_node))
                        {
                            if taken_vars.contains(&next_node.abs()) {
                                continue;
                            }

                            self.dfs(path, taken_vars, *next_node);
                        }

                        taken_vars.remove(&neighbour.abs());
                    }
                    _ => {
                        self.dfs(path, taken_vars, *neighbour);
                    }
                }
            }
        }

        path.pop();
        taken_vars.remove(&node.abs());
    }
}

impl Render for Graph {
    fn render(&self) -> String {
        let mut output_bytes = Vec::new();
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);
        let mut scope = writer.graph();

        let mut visited = HashSet::new();

        for (&current, neighbours) in &self.edges {
            for &adjacent in neighbours {
                let c = scope.node_named(current.to_string()).id();
                let a = scope.node_named(adjacent.to_string()).id();

                if visited.contains(&(current, adjacent))
                    || visited.contains(&(adjacent, current))
                {
                    continue;
                }
                visited.insert((current, adjacent));

                scope.edge(c, a);
            }
        }

        drop(scope);
        unsafe { String::from_utf8_unchecked(output_bytes) }
    }
}

impl ExecutionManager {
    pub fn new(root: ExpressionNode) -> Self {
        let mut filters = vec![];
        let root = Self::map(root, &mut filters);
        let counter = filters.len();
        ExecutionManager { filters, root, counter }
    }

    fn map(root: ExpressionNode, filters: &mut Vec<Filter>) -> Nnf<FilterVar> {
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

    fn extract_var_id_filter_map<'a>(
        root: &'a Nnf<FilterVar>,
        map: &mut [&'a FilterVar],
    ) {
        match root {
            Nnf::Var(filter_var, ..) => {
                map[filter_var.id()] = filter_var;
            }
            Nnf::Or(children) | Nnf::And(children) => {
                for child in children {
                    Self::extract_var_id_filter_map(child, map);
                }
            }
        }
    }

    pub fn tseitin_transform(self) -> Self {
        let mut counter = self.counter;
        let transformer = TseitinTransform::new(|| {
            let v = var!(FilterVar::new_aux(counter));
            counter += 1;
            v
        });

        let root = transformer.transform(self.root);
        assert!(root.is_cnf());

        Self { filters: self.filters, root, counter }
    }

    pub fn prepare_execution_plan(&self) -> Result<(), GenericError> {
        let clauses = if let Nnf::And(children) = &self.root {
            children
        } else {
            unsafe { unreachable_unchecked() }
        };

        let mut solver = self.build_solver();

        let default_filter = FilterVar::default();
        let mut var_id_to_filter_var_map = vec![&default_filter; self.counter];
        Self::extract_var_id_filter_map(&self.root, &mut var_id_to_filter_var_map);

        self.setup_solver(&mut solver, clauses.iter())?;

        let mut graph = Graph::new(self.filters.len());

        for mut arrangement in solver.iter() {
            arrangement.drain(self.filters.len()..);
            arrangement.sort_unstable_by_key(|&var| {
                let index = var.unsigned_abs() as usize - 1;
                let filter_var = var_id_to_filter_var_map[index];
                filter_var.weight()
            });
            arrangement.reverse();

            println!("{:?}", arrangement);
            graph.add_all(arrangement);
        }

        let mut path = vec![];
        let mut visited = HashSet::new();

        graph.dfs(&mut path, &mut visited, 1);

        for (i, filter) in self.filters.iter().enumerate() {
            println!("{}: {}", i + 1, filter);
        }

        Ok(())
    }

    fn map_arrangement_to_filters(
        &self,
        arrangement: Vec<i32>,
        var_id_to_filter_var_map: &[&FilterVar],
    ) -> Vec<(usize, usize, bool)> {
        let a = arrangement
            .into_iter()
            .filter_map(|var_id| {
                let (var_id, value) = (var_id.unsigned_abs() as usize, var_id > 0);
                let filter_var = var_id_to_filter_var_map[var_id - 1];
                match filter_var {
                    FilterVar::Var { id, weight } => Some((*id, *weight, value)),
                    FilterVar::Aux(_) => None,
                }
            })
            .sorted_by_key(|(_, weight, _)| *weight)
            .collect::<Vec<_>>();

        a
    }

    fn setup_solver<'a>(
        &self,
        solver: &mut Solver,
        clauses: impl Iterator<Item = &'a Nnf<FilterVar>>,
    ) -> Result<(), GenericError> {
        for clause in clauses {
            let statement = self.extract_clause_vars(clause);

            if let Err(err) = solver.add_clause(&statement) {
                return Err(GenericError::CustomSolverError(
                    err,
                    format!("{:?}", statement),
                ));
            }
        }

        Ok(())
    }

    fn build_solver(&self) -> Solver {
        Solver::instantiate(
            &Config::default(),
            &CNFDescription {
                num_of_variables: self.counter,
                ..CNFDescription::default()
            },
        )
    }

    fn extract_clause_vars(&self, root: &Nnf<FilterVar>) -> Vec<i32> {
        let mut statement = vec![];

        if let Nnf::Or(vars) = root {
            for var in vars {
                match var {
                    Nnf::Var(filter_var, value) => {
                        let id = *match filter_var {
                            FilterVar::Var { id, .. } => id,
                            FilterVar::Aux(id) => id,
                        } + 1; // this +1 is for solver because 0 is not allowed there

                        let mut id = id as i32;
                        if !value {
                            id *= -1;
                        }

                        statement.push(id);
                    }
                    _ => unsafe { unreachable_unchecked() },
                }
            }
        } else {
            unsafe { unreachable_unchecked() }
        }
        statement
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
    use crate::evaluate::execution_manager::{ExecutionManager, FilterVar};
    use crate::evaluate::nnf::macros::{or, var};
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
        // println!("{}", nnf.render());

        let mapper = ExecutionManager::new(nnf).tseitin_transform();
        // println!("{}", mapper.render());
        mapper.prepare_execution_plan().unwrap();
    }

    #[test]
    fn test_ord() {
        assert!(var!(FilterVar::Aux(0), true) < var!(FilterVar::Aux(2), true));
        assert!(var!(FilterVar::Aux(0), false) < var!(FilterVar::Aux(2), true));
        assert!(var!(FilterVar::Aux(0), false) < var!(FilterVar::Aux(0), true));

        assert!(var!(FilterVar::Aux(0), false) < var!(FilterVar::new_var(0, 1)));
        assert!(var!(FilterVar::Aux(0), true) < var!(FilterVar::new_var(0, 1)));

        assert!(
            or!(var!(FilterVar::Aux(0), true), var!(FilterVar::Aux(2), true))
                < or!(var!(FilterVar::Aux(0), false), var!(FilterVar::new_var(0, 1)))
        );
    }
}
