mod basic_blocks;
mod builder;
mod resolver;
mod lowering;

pub use lowering::build_cfg_from_root;