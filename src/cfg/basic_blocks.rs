use rslint_parser::{ast, AstNode, NodeOrToken, SyntaxNode};

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
    pub fn successors(&self, exit: BlockId) -> Vec<BlockId> {
        match &self.terminator {
            Terminator::Goto(target) => vec![*target],
            Terminator::Branch { then_bb, else_bb, .. } => vec![*then_bb, *else_bb],
            Terminator::Return { .. } => vec![exit],
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

impl StatementInfo {
    pub fn has_initializer(&self) -> bool {
        matches!(self.kind, StatementKind::VarDecl { has_initializer: true })
    }

    pub fn is_var_decl(&self) -> bool {
        matches!(self.kind, StatementKind::VarDecl { .. })
    }

    pub fn is_write_like(&self) -> bool {
        matches!(
            self.kind,
            StatementKind::Assign
                | StatementKind::Update
                | StatementKind::VarDecl { has_initializer: true }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementKind {
    VarDecl { has_initializer: bool },
    Assign,
    Update,
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

impl Span {
    pub fn from_node(node: &SyntaxNode) -> Self {
        let range = node.text_range();
        Span {
            start: range.start().into(),
            end: range.end().into(),
        }
    }
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



impl From<&ast::VarDecl> for SymbolKind {
    fn from(value: &ast::VarDecl) -> Self {
        let first_token = value
            .syntax()
            .children_with_tokens()
            .find_map(|el| match el {
                NodeOrToken::Token(tok) => Some(tok),
                NodeOrToken::Node(_) => None,
            })
            .expect("VarDecl should have a declaration keyword");

        match first_token.text().as_str() {
            "var" => SymbolKind::Var,
            "let" => SymbolKind::Let,
            "const" => SymbolKind::Const,
            other => panic!("unexpected var decl keyword: {}", other),
        }
    }
}