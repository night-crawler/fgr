use std::time::SystemTime;

use lazy_static::lazy_static;

pub mod expression_node_impl;
pub mod comparison_impl;
pub mod traits;
pub mod filter_impl;

lazy_static! {
    pub static ref NOW: SystemTime = SystemTime::now();
}
