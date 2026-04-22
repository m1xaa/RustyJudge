pub(crate) mod basic_blocks;
mod builder;
mod resolver;
mod lowering;
pub(crate) mod liveness;
pub(crate) mod semantic;

pub use lowering::build_semantic_model_from_root;
pub use liveness::{compute_liveness, print_liveness, print_statement_liveness};
pub use semantic::SemanticModel;