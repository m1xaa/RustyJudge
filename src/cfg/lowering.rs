use std::cell::RefCell;
use crate::cfg::basic_blocks::{Cfg, Span, SymbolId, SymbolKind};
use crate::cfg::builder::CfgBuilder;
use rslint_parser::{ast, AstNode, SyntaxKind, SyntaxNode};
use crate::cfg::resolver::Resolver;
use crate::cfg::semantic::SemanticModel;
use crate::compute_liveness;

pub fn build_semantic_model_from_root(root: SyntaxNode) -> Result<SemanticModel, String> {
    let mut resolver = Resolver::new();
    let mut builder = CfgBuilder::new();

    {
        let mut lowerer = AstLowerer {
            resolver: &mut resolver,
            builder: &mut builder,
        };

        lowerer.lower_root(&root)?;
    }

    let cfg = builder.finish();
    let liveness = compute_liveness(&cfg);
    let symbols = resolver.symbol_table().clone();

    Ok(SemanticModel {
        cfg,
        symbols,
        liveness,
    })
}

struct AstLowerer<'a> {
    resolver: &'a mut Resolver,
    builder: &'a mut CfgBuilder,
}

impl<'a> AstLowerer<'a> {
    fn lower_root(&mut self, root: &SyntaxNode) -> Result<(), String> {
        let script = ast::Script::cast(root.clone())
            .ok_or("root is not a Script node")?;

        for child in script.syntax().children() {
            if child.kind() == SyntaxKind::FOR_OF_STMT {
                let for_of_stmt = ast::ForOfStmt::cast(child.clone())
                    .ok_or("failed to cast FOR_OF_STMT")?;
                self.lower_for_of_stmt(&for_of_stmt)?;
                continue;
            }

            if let Some(stmt) = ast::Stmt::cast(child.clone()) {
                self.lower_stmt(stmt)?;
            }
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

                    if self.try_lower_update_expr(expr.clone(), span)? {
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
                    self.lower_var_decl(var_decl)?;
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

            ast::Stmt::ForStmt(for_stmt) => {
                self.lower_for_stmt(&for_stmt)?;
            }

            ast::Stmt::ForInStmt(for_in_stmt) => {
                self.lower_for_in_stmt(&for_in_stmt)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn lower_for_in_stmt(&mut self, for_in_stmt: &ast::ForInStmt) -> Result<(), String> {
        let left_syntax = for_in_stmt
            .left()
            .ok_or("for-in statement missing left side")?
            .syntax()
            .clone();

        let right_expr = for_in_stmt
            .right()
            .ok_or("for-in statement missing right side")?;

        let body_stmt = for_in_stmt
            .cons()
            .ok_or("for-in statement missing body")?;

        self.lower_for_each_loop(
            left_syntax,
            right_expr,
            body_stmt,
        )
    }

    fn lower_for_of_stmt(&mut self, for_of_stmt: &ast::ForOfStmt) -> Result<(), String> {
        let left_syntax = for_of_stmt
            .left()
            .ok_or("for-of statement missing left side")?
            .syntax()
            .clone();

        let right_expr = for_of_stmt
            .right()
            .ok_or("for-of statement missing right side")?;

        let body_stmt = for_of_stmt
            .cons()
            .ok_or("for-of statement missing body")?;

        self.lower_for_each_loop(
            left_syntax,
            right_expr,
            body_stmt,
        )
    }

    fn lower_for_each_loop(
        &mut self,
        left_syntax: SyntaxNode,
        right_expr: ast::Expr,
        body_stmt: ast::Stmt,
    ) -> Result<(), String> {
        self.resolver.enter_scope();

        let target_symbol = self.lower_for_each_left_from_syntax(&left_syntax)?;
        let source_uses = self.resolver.collect_expr_uses(right_expr);
        let assign_span = Span::from_node(&left_syntax);

        let resolver = RefCell::new(&mut *self.resolver);
        let body_result = RefCell::new(Ok(()));

        self.builder.build_for_each(
            source_uses,
            target_symbol,
            assign_span,
            |builder| {
                let mut resolver_ref = resolver.borrow_mut();
                let mut nested = AstLowerer {
                    resolver: &mut *resolver_ref,
                    builder,
                };
                *body_result.borrow_mut() = nested.lower_stmt(body_stmt.clone());
            },
        );

        let out = body_result.into_inner();
        self.resolver.exit_scope();
        out
    }

    fn lower_for_each_left_from_syntax(
        &mut self,
        left_syntax: &SyntaxNode,
    ) -> Result<SymbolId, String> {
        if let Some(var_decl) = left_syntax.descendants().find_map(ast::VarDecl::cast) {
            let decl_kind: SymbolKind = (&var_decl).into();

            let declarator = var_decl
                .syntax()
                .children()
                .find_map(ast::Declarator::cast)
                .ok_or("expected declarator in for-each left side")?;

            let span = Span::from_node(declarator.syntax());

            let name = simple_decl_name(&declarator)
                .ok_or("only simple identifier loop targets are supported for now")?;

            let symbol_id = self.resolver.declare(name, decl_kind, span)?;
            self.builder
                .build_variable_declaration(span, vec![symbol_id], vec![]);

            return Ok(symbol_id);
        }

        if let Some(expr) = left_syntax.descendants().find_map(ast::Expr::cast) {
            let name = simple_expr_name(&expr)
                .ok_or("only simple identifier loop targets are supported for now")?;

            let symbol_id = self
                .resolver
                .resolve(&name)
                .ok_or_else(|| format!("for-each target '{}' is not declared", name))?;

            return Ok(symbol_id);
        }

        Err("unsupported for-each left side".to_string())
    }

    fn try_lower_update_expr(
        &mut self,
        expr: ast::Expr,
        span: Span,
    ) -> Result<bool, String> {
        let Some(name) = update_target_name(&expr) else {
            return Ok(false);
        };

        let symbol_id = self
            .resolver
            .resolve(&name)
            .ok_or_else(|| format!("update of unknown symbol '{}'", name))?;

        self.builder
            .build_assignment(span, vec![symbol_id], vec![symbol_id]);

        Ok(true)
    }
    fn lower_var_decl(&mut self, var_decl: ast::VarDecl) -> Result<(), String> {
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

        Ok(())
    }

    fn lower_header_expr(&mut self, expr: ast::Expr, span: Span) -> Result<(), String> {
        if self.try_lower_assignment(expr.clone(), span)? {
            return Ok(());
        }

        if self.try_lower_update_expr(expr.clone(), span)? {
            return Ok(());
        }

        let uses = self.resolver.collect_expr_uses(expr);
        self.builder.build_expression_statement(span, uses);
        Ok(())
    }

    fn lower_for_init(&mut self, init: &ast::ForStmtInit) -> Result<(), String> {
        if let Some(var_decl) = init.syntax().descendants().find_map(ast::VarDecl::cast) {
            return self.lower_var_decl(var_decl);
        }

        if let Some(expr) = init.syntax().descendants().find_map(ast::Expr::cast) {
            return self.lower_header_expr(expr, Span::from_node(init.syntax()));
        }

        Ok(())
    }

    fn lower_for_update(&mut self, update: &ast::ForStmtUpdate) -> Result<(), String> {
        if let Some(expr) = update.syntax().descendants().find_map(ast::Expr::cast) {
            return self.lower_header_expr(expr, Span::from_node(update.syntax()));
        }

        Ok(())
    }

    fn lower_for_stmt(&mut self, for_stmt: &ast::ForStmt) -> Result<(), String> {
        self.resolver.enter_scope();

        if let Some(init) = for_stmt.init() {
            self.lower_for_init(&init)?;
        }

        let condition_uses = for_stmt
            .test()
            .and_then(|test| test.syntax().descendants().find_map(ast::Expr::cast))
            .map(|expr| self.resolver.collect_expr_uses(expr));

        let body_stmt = for_stmt
            .cons()
            .ok_or("for statement missing body")?;

        let update = for_stmt.update();

        let resolver = RefCell::new(&mut *self.resolver);
        let body_result = RefCell::new(Ok(()));
        let update_result = RefCell::new(Ok(()));

        if let Some(update) = update {
            self.builder.build_for(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                    };
                    *body_result.borrow_mut() = nested.lower_stmt(body_stmt.clone());
                },
                Some(|builder: &mut CfgBuilder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                    };
                    *update_result.borrow_mut() = nested.lower_for_update(&update);
                }),
            );
        } else {
            self.builder.build_for(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                    };
                    *body_result.borrow_mut() = nested.lower_stmt(body_stmt.clone());
                },
                None::<fn(&mut CfgBuilder)>,
            );
        }

        let body_outcome = body_result.into_inner();
        let update_outcome = update_result.into_inner();

        self.resolver.exit_scope();

        body_outcome?;
        update_outcome
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

fn update_target_name(expr: &ast::Expr) -> Option<String> {
    let text = expr.syntax().text().to_string();

    let is_update = text.starts_with("++")
        || text.starts_with("--")
        || text.ends_with("++")
        || text.ends_with("--");

    if !is_update {
        return None;
    }

    expr.syntax()
        .descendants_with_tokens()
        .find_map(|elem| match elem {
            rslint_parser::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
                Some(tok.text().to_string())
            }
            _ => None,
        })
}

fn unary_update_target_name(expr: &ast::Expr) -> Option<String> {
    let unary = match expr {
        ast::Expr::UnaryExpr(unary) => unary,
        _ => return None,
    };

    let text = unary.syntax().text().to_string();
    let is_update = text.starts_with("++")
        || text.starts_with("--")
        || text.ends_with("++")
        || text.ends_with("--");

    if !is_update {
        return None;
    }

    unary
        .syntax()
        .descendants_with_tokens()
        .find_map(|elem| match elem {
            rslint_parser::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
                Some(tok.text().to_string())
            }
            _ => None,
        })
}

fn simple_expr_name(expr: &ast::Expr) -> Option<String> {
    expr.syntax()
        .descendants_with_tokens()
        .find_map(|elem| match elem {
            rslint_parser::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
                Some(tok.text().to_string())
            }
            _ => None,
        })
}