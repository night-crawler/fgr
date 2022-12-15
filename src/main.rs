use std::sync::{Arc, Mutex};

use ignore::WalkBuilder;

use crate::config::Config;
use crate::errors::GenericError;
use crate::evaluate::traits::Evaluate;
use crate::parse::expression_node::ExpressionNode;
use crate::parse::parse_root;
use crate::parse::render::Render;
use crate::run::{set_int_handler, spawn_receiver, spawn_senders, ProcessStatus};

pub mod config;
pub mod errors;
pub mod evaluate;
pub mod parse;
pub mod run;
pub mod walk;

#[cfg(test)]
pub mod test_utils;
pub mod r#macro;

fn main() {
    let config = match Config::build() {
        Ok(config) => config,
        Err(error) => {
            println!("Failed to build configuration: {:?}", error);
            std::process::exit(1);
        }
    };

    if config.print_expression_tree {
        println!("{}", config.root.render());
        std::process::exit(0);
    }

    let mut dir_iter = config.start_dirs.iter();
    let first_path = dir_iter.next().unwrap();

    let root_node = Arc::new(config.root);

    let mut builder = WalkBuilder::new(first_path);
    builder.standard_filters(config.standard_filters);
    config.hidden.map(|yes| builder.hidden(yes));
    config.parents.map(|yes| builder.parents(yes));
    config.ignore.map(|yes| builder.ignore(yes));
    config.git_ignore.map(|yes| builder.git_ignore(yes));
    config.git_global.map(|yes| builder.git_global(yes));
    config.git_exclude.map(|yes| builder.git_exclude(yes));
    config.same_filesystem.map(|yes| builder.same_file_system(yes));

    builder.threads(config.threads);

    let walk = builder.build_parallel();

    let (sender, receiver) = kanal::unbounded();
    let status = Arc::new(Mutex::new(ProcessStatus::InProgress));

    set_int_handler(&status);

    spawn_senders(&status, &root_node, sender, walk);

    let handle = spawn_receiver(&status, receiver);

    let status = handle.join().unwrap();
    std::process::exit(status);
}
