pub struct Diagnostic {
    row: usize,
    col: usize,
    message: String
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