use std::cell::RefCell;
use crate::cfg::basic_blocks::{Cfg, Span, SymbolKind};
use crate::cfg::builder::CfgBuilder;
use rslint_parser::{ast, AstNode, SyntaxKind, SyntaxNode};
use crate::cfg::resolver::Resolver;

pub fn build_cfg_from_root(root: SyntaxNode) -> Result<Cfg, String> {
    let mut resolver = Resolver::new();
    let mut builder = CfgBuilder::new();

    {
        let mut lowerer = AstLowerer {
            resolver: &mut resolver,
            builder: &mut builder,
        };

        lowerer.lower_root(&root)?;
    }

    Ok(builder.finish())
}

struct AstLowerer<'a> {
    resolver: &'a mut Resolver,
    builder: &'a mut CfgBuilder,
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
                    let span = Span::from_node(expr_stmt.syntax());

                    if self.try_lower_assignment(expr.clone(), span)? {
                        return Ok(());
                    }

                    let uses = self.resolver.collect_expr_uses(expr);
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

                            let symbol_id = self.resolver.declare(name, decl_kind, span)?;

                            let uses = declarator_init_expr(&declarator)
                                .map(|expr| self.resolver.collect_expr_uses(expr))
                                .unwrap_or_default();

                            self.builder
                                .build_variable_declaration(span, vec![symbol_id], uses);
                        }
                    }
                }
            }

            ast::Stmt::BlockStmt(block_stmt) => {
                self.lower_block_stmt(&block_stmt)?;
            }

            ast::Stmt::BreakStmt(break_stmt) => {
                let span = Span::from_node(break_stmt.syntax());
                self.builder.build_break(span);
            }

            ast::Stmt::ContinueStmt(continue_stmt) => {
                let span = Span::from_node(continue_stmt.syntax());
                self.builder.build_continue(span);
            }

            ast::Stmt::WhileStmt(while_stmt) => {
                self.lower_while_stmt(&while_stmt)?;
            }

            ast::Stmt::IfStmt(if_stmt) => {
                self.lower_if_stmt(&if_stmt)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn lower_if_stmt(&mut self, if_stmt: &ast::IfStmt) -> Result<(), String> {
        let condition_expr = if_stmt
            .condition()
            .and_then(|cond| cond.condition())
            .ok_or("if statement missing condition")?;

        let then_stmt = if_stmt
            .cons()
            .ok_or("if statement missing then branch")?;

        let else_stmt = if_stmt.alt();

        let condition_uses = self.resolver.collect_expr_uses(condition_expr);

        let resolver = RefCell::new(&mut *self.resolver);
        let then_result = RefCell::new(Ok(()));
        let else_result = RefCell::new(Ok(()));

        if let Some(else_stmt) = else_stmt {
            self.builder.build_if(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                    };
                    *then_result.borrow_mut() = nested.lower_stmt(then_stmt.clone());
                },
                Some(|builder: &mut CfgBuilder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                    };
                    *else_result.borrow_mut() = nested.lower_stmt(else_stmt.clone());
                }),
            );
        } else {
            self.builder.build_if(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                    };
                    *then_result.borrow_mut() = nested.lower_stmt(then_stmt.clone());
                },
                None::<fn(&mut CfgBuilder)>,
            );
        }

        then_result.into_inner()?;
        else_result.into_inner()
    }

    fn lower_while_stmt(&mut self, while_stmt: &ast::WhileStmt) -> Result<(), String> {
        let condition_expr = while_stmt
            .condition()
            .and_then(|cond| cond.condition())
            .ok_or("while statement missing condition")?;

        let body_stmt = while_stmt
            .cons()
            .ok_or("while statement missing body")?;

        let condition_uses = self.resolver.collect_expr_uses(condition_expr);

        let resolver = &mut *self.resolver;
        let mut body_result: Result<(), String> = Ok(());

        self.builder.build_while(condition_uses, |builder| {
            let mut nested = AstLowerer { resolver, builder };
            body_result = nested.lower_stmt(body_stmt.clone());
        });

        body_result
    }

    fn lower_block_stmt(&mut self, block_stmt: &ast::BlockStmt) -> Result<(), String> {
        self.resolver.enter_scope();

        for stmt in block_stmt.stmts() {
            self.lower_stmt(stmt)?;
        }

        self.resolver.exit_scope();
        Ok(())
    }

    fn try_lower_assignment(
        &mut self,
        expr: ast::Expr,
        span: Span,
    ) -> Result<bool, String> {
        let assign = match expr {
            ast::Expr::AssignExpr(assign) => assign,
            _ => return Ok(false),
        };

        let lhs_name = simple_assign_lhs_name(&assign)
            .ok_or("only simple identifier assignments are supported for now")?;

        let defined_symbol = self
            .resolver
            .resolve(&lhs_name)
            .ok_or_else(|| format!("assignment to unknown symbol '{}'", lhs_name))?;

        let rhs = assign.rhs().ok_or("assignment missing rhs")?;
        let mut uses = self.resolver.collect_expr_uses(rhs);

        if is_compound_assignment(&assign) && !uses.contains(&defined_symbol) {
            uses.push(defined_symbol);
        }

        self.builder
            .build_assignment(span, vec![defined_symbol], uses);

        Ok(true)
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

fn simple_assign_lhs_name(assign: &ast::AssignExpr) -> Option<String> {
    let lhs = assign.lhs()?;

    lhs.syntax()
        .descendants_with_tokens()
        .find_map(|elem| match elem {
            rslint_parser::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
                Some(tok.text().to_string())
            }
            _ => None,
        })
}

fn is_compound_assignment(assign: &ast::AssignExpr) -> bool {
    assign
        .syntax()
        .children_with_tokens()
        .find_map(|elem| match elem {
            rslint_parser::NodeOrToken::Token(tok) => {
                let text = tok.text();
                if text.ends_with('=') {
                    Some(text != "=")
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap_or(false)
}