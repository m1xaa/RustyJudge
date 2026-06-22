use std::collections::HashMap;
use rslint_parser::{AstNode, SyntaxNode};
use rslint_parser::ast;
use crate::cfg::basic_blocks::{Span, Symbol, SymbolId, SymbolKind};

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Default)]
pub struct Scope {
    bindings: HashMap<String, SymbolId>,
}

pub struct Resolver {
    symbol_table: SymbolTable,
    scopes: Vec<Scope>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::default(),
            scopes: vec![Scope::default()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn exit_scope(&mut self) {
        assert!(self.scopes.len() > 1, "cannot exit global scope");
        self.scopes.pop();
    }

    pub fn declare(
        &mut self,
        name: String,
        kind: SymbolKind,
        decl_span: Span,
    ) -> Result<SymbolId, String> {
        let current_scope = self
            .scopes
            .last_mut()
            .expect("resolver must always have at least one scope");

        if current_scope.bindings.contains_key(&name) {
            return Err(format!("symbol '{}' already declared in this scope", name));
        }

        let id = self.symbol_table.symbols.len();

        let symbol = Symbol {
            id,
            name: name.clone(),
            decl_span,
            kind,
        };

        self.symbol_table.symbols.push(symbol);
        current_scope.bindings.insert(name, id);

        Ok(id)
    }

    pub fn resolve(&self, name: &str) -> Option<SymbolId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&symbol_id) = scope.bindings.get(name) {
                return Some(symbol_id);
            }
        }

        None
    }

    pub fn is_declared_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.bindings.contains_key(name))
            .unwrap_or(false)
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn collect_expr_uses(&self, expr: ast::Expr) -> Vec<SymbolId> {
        let mut uses = Vec::new();
        self.collect_expr_uses_into(expr.syntax(), &mut uses);
        uses
    }

    fn collect_expr_uses_into(&self, node: &SyntaxNode, uses: &mut Vec<SymbolId>) {
        if ast::FnExpr::cast(node.clone()).is_some() || ast::ArrowExpr::cast(node.clone()).is_some() {
            return;
        }

        if let Some(name_ref) = ast::NameRef::cast(node.clone()) {
            let name = name_ref.syntax().text().to_string();

            if let Some(symbol_id) = self.resolve(&name) {
                if !uses.contains(&symbol_id) {
                    uses.push(symbol_id);
                }
            }
        }

        for child in node.children() {
            self.collect_expr_uses_into(&child, uses);
        }
    }
}