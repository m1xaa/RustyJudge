use std::{fs, path::Path};
use rslint_parser::{parse_text, SyntaxNode};

use crate::parser::error::ParseError;

pub fn read_js_file(path: &Path) -> Result<String, ParseError> {
    Ok(fs::read_to_string(path)?)
}


pub fn parse_source(source: &String) -> Result<SyntaxNode, ParseError> {
    let parse = parse_text(source, 0).to_syntax();

    if let Some(err) = parse.errors().first() {
        return Err(ParseError::Syntax(err.clone()));
    }

    Ok(parse.syntax())
}
