use crate::cfg::basic_blocks::{Cfg, Span, Symbol, SymbolId};
use crate::cfg::liveness::LivenessResult;
use crate::cfg::resolver::SymbolTable;

#[derive(Debug, Clone)]
pub struct BodyAnalysis {
    pub cfg: Cfg,
    pub liveness: LivenessResult,
}

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub name: Option<String>,
    pub symbol_id: Option<SymbolId>,
    pub span: Span,
    pub params: Vec<SymbolId>,
    pub body: BodyAnalysis,
}

#[derive(Debug, Clone)]
pub struct SemanticModel {
    pub script: BodyAnalysis,
    pub functions: Vec<FunctionAnalysis>,
    pub symbols: SymbolTable,
}

impl SemanticModel {
    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.symbols.get(id)
    }

    pub fn script_cfg(&self) -> &Cfg {
        &self.script.cfg
    }

    pub fn script_liveness(&self) -> &LivenessResult {
        &self.script.liveness
    }
}