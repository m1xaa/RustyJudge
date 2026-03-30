mod basic_blocks;
mod builder;
mod resolver;
mod lowering;
mod liveness;

pub use lowering::build_cfg_from_root;
pub use liveness::{compute_liveness, print_liveness};