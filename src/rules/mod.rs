mod rule;
mod no_var;
mod no_unused_vars;
mod strict_equality;
mod no_console_log;
mod max_line_length;
mod no_duplicate_params;
mod no_empty_block;

pub use rule::*;
pub use no_unused_vars::*;
pub use strict_equality::*;
pub use no_var::*;
pub use no_console_log::*;
pub use max_line_length::*;
pub use no_duplicate_params::*;
pub use no_empty_block::*;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(MaxLineLength),
        Box::new(NoConsoleLog),
        Box::new(NoDuplicateParams),
        Box::new(NoEmptyBlock),
        Box::new(StrictEquality),
        Box::new(NoUnusedVars),
        Box::new(NoVar),
    ]
}