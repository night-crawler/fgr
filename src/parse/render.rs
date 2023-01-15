use dot_writer::DotWriter;
use nnf::nnf::Nnf;
use nnf::render_impls::traverse_nnf_node;
use nnf::traits::Render;

use crate::evaluate::execution_manager::{ExecutionManager, FilterVar};

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
