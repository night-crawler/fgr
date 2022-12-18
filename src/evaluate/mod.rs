use std::time::SystemTime;

use lazy_static::lazy_static;

pub mod comparison_impl;
pub mod expression_node_impl;
pub mod filter_impl;
pub mod traits;
pub mod nnf;

lazy_static! {
    pub static ref NOW: SystemTime = SystemTime::now();
}
