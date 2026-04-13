use crate::cfg::basic_blocks::{Cfg, Symbol, SymbolId};
use crate::cfg::liveness::LivenessResult;
use crate::cfg::resolver::SymbolTable;

#[derive(Debug, Clone)]
pub struct SemanticModel {
    pub cfg: Cfg,
    pub symbols: SymbolTable,
    pub liveness: LivenessResult,
}

impl SemanticModel {
    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.symbols.get(id)
    }
}