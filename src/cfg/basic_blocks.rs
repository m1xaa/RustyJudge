pub type BlockId = usize;
pub type StatementId = usize;
pub type SymbolId = usize;

#[derive(Debug, Clone)]
pub struct Cfg {
    pub entry: BlockId,
    pub exit: BlockId,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub statements: Vec<StatementInfo>,
    pub terminator: Terminator,
    pub predecessors: Vec<BlockId>,
}

impl BasicBlock {
    pub fn successors(&self) -> Vec<BlockId> {
        match &self.terminator {
            Terminator::Goto(target) => vec![*target],
            Terminator::Branch { then_bb, else_bb, .. } => vec![*then_bb, *else_bb],
            Terminator::Return { .. } => vec![],
            Terminator::Unset => vec![],
            Terminator::Exit => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatementInfo {
    pub id: StatementId,
    pub kind: StatementKind,
    pub span: Span,
    pub defines: Vec<SymbolId>,
    pub uses: Vec<SymbolId>,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    VarDecl,
    Assign,
    Expr,
    Return,
    Break,
    Continue,
    Empty,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(BlockId),
    Branch {
        cond_uses: Vec<SymbolId>,
        then_bb: BlockId,
        else_bb: BlockId,
    },
    Return {
        value_uses: Vec<SymbolId>,
    },
    Unset,
    Exit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub decl_span: Span,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Var,
    Let,
    Const,
    Param,
    Function,
}