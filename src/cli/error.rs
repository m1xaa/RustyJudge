use std::{fmt, io};
use std::fmt::Formatter;

#[derive(Debug)]
pub enum CliError {
    InvalidFlag(String),
    InvalidFlagValue(String),
    InsufficientArgs,
    Io(io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InvalidFlag(cmd) => {
                write!(f, "Unknown command: {}", cmd)
            }
            CliError::InvalidFlagValue(cmd) => {
                write!(f, "Invalid value for flag: {}", cmd)
            }
            CliError::InsufficientArgs => {
                write!(f, "Must provide at least 1 file to lint")
            }
            CliError::Io(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::Io(e)
    }
}

impl std::error::Error for CliError {}
