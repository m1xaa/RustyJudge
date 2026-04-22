use std::fmt;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) message: String
}

impl Diagnostic {
    pub fn new(row: usize, col: usize, message: String) -> Diagnostic {
        Diagnostic {
            row,
            col,
            message
        }
    }
    
    pub fn message(&self) -> &str {
        &self.message
    }
}



impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "row:col  --> {}:{}  {}", self.row, self.col, self.message)
    }
}
