mod error;
mod command;
pub(crate) mod cli;
#[cfg(test)]
mod tests;


pub use cli::parse_cli;