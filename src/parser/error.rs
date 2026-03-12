use std::fmt;
use std::io;
use std::error::Error as StdError;
use rslint_parser::ParserError;

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Syntax(ParserError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {}", e),
            ParseError::Syntax(e) => write!(f, "Syntax error: {:?}", e),
        }
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ParseError::Io(e) => Some(e),
            ParseError::Syntax(_) => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}

impl From<ParserError> for ParseError {
    fn from(value: ParserError) -> Self {
        ParseError::Syntax(value)
    }
}
