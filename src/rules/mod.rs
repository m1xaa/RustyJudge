mod rule;
mod no_var;
mod no_unused_vars;
mod strict_equality;
mod no_console_log;
mod max_line_length;
mod no_duplicate_params;
mod no_empty_block;
mod no_unreachable_code;
mod consistent_return;
mod no_dead_stores;
mod no_infinite_loops;

pub use rule::*;
pub use no_unused_vars::*;
pub use strict_equality::*;
pub use no_var::*;
pub use no_console_log::*;
pub use max_line_length::*;
pub use no_duplicate_params::*;
pub use no_empty_block::*;
pub use no_unreachable_code::*;
pub use consistent_return::*;
pub use no_dead_stores::*;
pub use no_infinite_loops::*;

pub fn default_rules() -> Vec<Box<dyn Rule + Send + Sync>> {
    vec![
        Box::new(MaxLineLength),
        Box::new(NoConsoleLog),
        Box::new(NoDuplicateParams),
        Box::new(NoEmptyBlock),
        Box::new(StrictEquality),
        Box::new(NoUnusedVars),
        Box::new(NoVar),
        Box::new(NoUnreachableCode),
        Box::new(ConsistentReturn),
        Box::new(NoDeadStores),
        Box::new(NoInfiniteLoops),
    ]
}