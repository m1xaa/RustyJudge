use std::{fs, path::Path};
use rslint_parser::{parse_text, SyntaxNode};

use crate::parser::error::ParseError;

pub fn parse_js_file(path: &Path) -> Result<(SyntaxNode, String), ParseError> {
    let source = fs::read_to_string(path)?;
    let parse = parse_text(&source, 0).to_syntax();

    if let Some(err) = parse.errors().first() {
        return Err(ParseError::Syntax(err.clone()));
    }

    Ok((parse.syntax(), source))
}
