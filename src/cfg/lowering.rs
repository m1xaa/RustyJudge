use crate::cfg::basic_blocks::{Cfg, Span, SymbolKind};
use crate::cfg::builder::CfgBuilder;
use rslint_parser::{ast, AstNode, SyntaxKind, SyntaxNode};
use crate::cfg::resolver::Resolver;

pub fn build_cfg_from_root(root: SyntaxNode) -> Result<Cfg, String> {
    let mut resolver = Resolver::new();
    let builder = CfgBuilder::new();

    let mut lowerer = AstLowerer {
        resolver: &mut resolver,
        builder,
    };

    lowerer.lower_root(&root)?;

    Ok(lowerer.builder.finish())
}

struct AstLowerer<'a> {
    resolver: &'a mut Resolver,
    builder: CfgBuilder,
}

impl<'a> AstLowerer<'a> {
    fn lower_root(&mut self, root: &SyntaxNode) -> Result<(), String> {
        let script = ast::Script::cast(root.clone())
            .ok_or("root is not a Script node")?;

        for stmt in script.items() {
            self.lower_stmt(stmt)?;
        }

        Ok(())
    }

    fn lower_stmt(&mut self, stmt: ast::Stmt) -> Result<(), String> {
        match stmt {
            ast::Stmt::ExprStmt(expr_stmt) => {
                if let Some(expr) = expr_stmt.expr() {
                    let uses = self.resolver.collect_expr_uses(expr);
                    let span = Span::from_node(expr_stmt.syntax());
                    self.builder.build_expression_statement(span, uses);
                }
            }

            ast::Stmt::ReturnStmt(ret) => {
                let span = Span::from_node(ret.syntax());

                let uses = ret
                    .value()
                    .map(|expr| self.resolver.collect_expr_uses(expr))
                    .unwrap_or_default();

                self.builder.build_return(span, uses);
            }

            ast::Stmt::Decl(decl) => {
                if let ast::Decl::VarDecl(var_decl) = decl {
                    let decl_kind: SymbolKind = (&var_decl).into();
                    for child in var_decl.syntax().children() {
                        if let Some(declarator) = ast::Declarator::cast(child) {
                            let span = Span::from_node(declarator.syntax());

                            let name = simple_decl_name(&declarator)
                                .ok_or("expected simple identifier declarator")?;

                            let symbol_id = self
                                .resolver
                                .declare(name, decl_kind, span)?;

                            let uses = declarator_init_expr(&declarator)
                                .map(|expr| {
                                    println!("{:#?}", expr);
                                    self.resolver.collect_expr_uses(expr)
                                })
                                .unwrap_or_default();

                            self.builder
                                .build_variable_declaration(span, vec![symbol_id], uses);
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }
}

fn simple_decl_name(declarator: &ast::Declarator) -> Option<String> {
    declarator
        .syntax()
        .descendants_with_tokens()
        .find_map(|elem| match elem {
            rslint_parser::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
                Some(tok.text().to_string())
            }
            _ => None,
        })
}

fn declarator_init_expr(declarator: &ast::Declarator) -> Option<ast::Expr> {
    let mut seen_eq = false;

    for elem in declarator.syntax().children_with_tokens() {
        match elem {
            rslint_parser::NodeOrToken::Token(tok) => {
                if tok.kind() == SyntaxKind::EQ {
                    seen_eq = true;
                }
            }
            rslint_parser::NodeOrToken::Node(node) => {
                if seen_eq {
                    if let Some(expr) = ast::Expr::cast(node) {
                        return Some(expr);
                    }
                }
            }
        }
    }

    if seen_eq {
        declarator
            .syntax()
            .descendants()
            .find_map(ast::Expr::cast)
    } else {
        None
    }
}