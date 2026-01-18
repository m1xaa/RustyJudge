use std::io;
use rslint_parser::ParserError;

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Syntax(rslint_parser::ParserError),
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}

impl From<rslint_parser::ParserError> for ParseError {
    fn from(value: ParserError) -> Self {
        ParseError::Syntax(value)
    }
}
