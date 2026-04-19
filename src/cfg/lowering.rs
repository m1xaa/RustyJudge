use std::cell::RefCell;

use crate::cfg::basic_blocks::{Span, SymbolId, SymbolKind};
use crate::cfg::builder::CfgBuilder;
use crate::cfg::resolver::Resolver;
use crate::cfg::semantic::{BodyAnalysis, FunctionAnalysis, SemanticModel};
use crate::compute_liveness;
use rslint_parser::{ast, AstNode, NodeOrToken, SyntaxKind, SyntaxNode};

pub fn build_semantic_model_from_root(root: SyntaxNode) -> Result<SemanticModel, String> {
    let mut resolver = Resolver::new();
    let mut builder = CfgBuilder::new();
    let mut functions = Vec::new();

    {
        let mut lowerer = AstLowerer {
            resolver: &mut resolver,
            builder: &mut builder,
            functions: &mut functions,
        };

        lowerer.lower_root(&root)?;
    }

    let script_cfg = builder.finish();
    let script_liveness = compute_liveness(&script_cfg);
    let symbols = resolver.symbol_table().clone();

    Ok(SemanticModel {
        script: BodyAnalysis {
            cfg: script_cfg,
            liveness: script_liveness,
        },
        functions,
        symbols,
    })
}

struct AstLowerer<'a> {
    resolver: &'a mut Resolver,
    builder: &'a mut CfgBuilder,
    functions: &'a mut Vec<FunctionAnalysis>,
}

impl<'a> AstLowerer<'a> {
    fn lower_root(&mut self, root: &SyntaxNode) -> Result<(), String> {
        let script = ast::Script::cast(root.clone())
            .ok_or("root is not a Script node")?;

        for child in script.syntax().children() {
            if let Some(stmt) = ast::Stmt::cast(child.clone()) {
                self.predeclare_function_decl_in_stmt(&stmt)?;
            }
        }

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
        if stmt.syntax().kind() == SyntaxKind::FOR_OF_STMT {
            let for_of_stmt = ast::ForOfStmt::cast(stmt.syntax().clone())
                .ok_or("failed to cast FOR_OF_STMT")?;
            return self.lower_for_of_stmt(&for_of_stmt);
        }

        if self.lower_function_decl_in_stmt(&stmt)? {
            return Ok(());
        }

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

                    self.lower_functions_in_expr(&expr, None, None)?;

                    let uses = self.resolver.collect_expr_uses(expr);
                    self.builder.build_expression_statement(span, uses);
                }
            }

            ast::Stmt::ReturnStmt(ret) => {
                let span = Span::from_node(ret.syntax());

                let uses = if let Some(expr) = ret.value() {
                    self.lower_functions_in_expr(&expr, None, None)?;
                    self.resolver.collect_expr_uses(expr)
                } else {
                    Vec::new()
                };

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
        self.lower_functions_in_expr(&right_expr, None, None)?;
        let source_uses = self.resolver.collect_expr_uses(right_expr);
        let assign_span = Span::from_node(&left_syntax);

        let resolver = RefCell::new(&mut *self.resolver);
        let functions = RefCell::new(&mut *self.functions);
        let body_result = RefCell::new(Ok(()));

        self.builder.build_for_each(
            source_uses,
            target_symbol,
            assign_span,
            |builder| {
                let mut resolver_ref = resolver.borrow_mut();
                let mut functions_ref = functions.borrow_mut();

                let mut nested = AstLowerer {
                    resolver: &mut *resolver_ref,
                    builder,
                    functions: &mut *functions_ref,
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
                .build_variable_declaration(span, vec![symbol_id], vec![], false);

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

        self.builder.build_update(span, symbol_id);

        Ok(true)
    }
    fn lower_var_decl(&mut self, var_decl: ast::VarDecl) -> Result<(), String> {
        let decl_kind: SymbolKind = (&var_decl).into();

        for child in var_decl.syntax().children() {
            if let Some(declarator) = ast::Declarator::cast(child) {
                let span = Span::from_node(declarator.syntax());

                let name = simple_decl_name(&declarator)
                    .ok_or("expected simple identifier declarator")?;

                let symbol_id = self.resolver.declare(name.clone(), decl_kind, span)?;

                let init_expr = declarator_init_expr(&declarator);
                let has_initializer = init_expr.is_some();

                if let Some(expr) = init_expr.clone() {
                    self.lower_functions_in_expr(&expr, Some(name.clone()), Some(symbol_id))?;
                }

                let uses = init_expr
                    .map(|expr| self.resolver.collect_expr_uses(expr))
                    .unwrap_or_default();

                self.builder.build_variable_declaration(
                    span,
                    vec![symbol_id],
                    uses,
                    has_initializer,
                );
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

        self.lower_functions_in_expr(&expr, None, None)?;

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

        let condition_uses = if let Some(test) = for_stmt.test() {
            if let Some(expr) = test.syntax().descendants().find_map(ast::Expr::cast) {
                self.lower_functions_in_expr(&expr, None, None)?;
                Some(self.resolver.collect_expr_uses(expr))
            } else {
                None
            }
        } else {
            None
        };

        let body_stmt = for_stmt
            .cons()
            .ok_or("for statement missing body")?;

        let update = for_stmt.update();

        let resolver = RefCell::new(&mut *self.resolver);
        let functions = RefCell::new(&mut *self.functions);
        let body_result = RefCell::new(Ok(()));
        let update_result = RefCell::new(Ok(()));

        if let Some(update) = update {
            self.builder.build_for(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut functions_ref = functions.borrow_mut();

                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                        functions: &mut *functions_ref,
                    };

                    *body_result.borrow_mut() = nested.lower_stmt(body_stmt.clone());
                },
                Some(|builder: &mut CfgBuilder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut functions_ref = functions.borrow_mut();

                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                        functions: &mut *functions_ref,
                    };

                    *update_result.borrow_mut() = nested.lower_for_update(&update);
                }),
            );
        } else {
            self.builder.build_for(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut functions_ref = functions.borrow_mut();

                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                        functions: &mut *functions_ref,
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

        self.lower_functions_in_expr(&condition_expr, None, None)?;
        let condition_uses = self.resolver.collect_expr_uses(condition_expr);

        let resolver = RefCell::new(&mut *self.resolver);
        let functions = RefCell::new(&mut *self.functions);
        let then_result = RefCell::new(Ok(()));
        let else_result = RefCell::new(Ok(()));

        if let Some(else_stmt) = else_stmt {
            self.builder.build_if(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut functions_ref = functions.borrow_mut();

                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                        functions: &mut *functions_ref,
                    };

                    *then_result.borrow_mut() = nested.lower_stmt(then_stmt.clone());
                },
                Some(|builder: &mut CfgBuilder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut functions_ref = functions.borrow_mut();

                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                        functions: &mut *functions_ref,
                    };

                    *else_result.borrow_mut() = nested.lower_stmt(else_stmt.clone());
                }),
            );
        } else {
            self.builder.build_if(
                condition_uses,
                |builder| {
                    let mut resolver_ref = resolver.borrow_mut();
                    let mut functions_ref = functions.borrow_mut();

                    let mut nested = AstLowerer {
                        resolver: &mut *resolver_ref,
                        builder,
                        functions: &mut *functions_ref,
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

        self.lower_functions_in_expr(&condition_expr, None, None)?;
        let condition_uses = self.resolver.collect_expr_uses(condition_expr);

        let resolver = &mut *self.resolver;
        let functions = &mut *self.functions;
        let mut body_result: Result<(), String> = Ok(());

        self.builder.build_while(condition_uses, |builder| {
            let mut nested = AstLowerer {
                resolver,
                builder,
                functions,
            };
            body_result = nested.lower_stmt(body_stmt.clone());
        });

        body_result
    }

    fn lower_block_stmt(&mut self, block_stmt: &ast::BlockStmt) -> Result<(), String> {
        self.resolver.enter_scope();

        let statements: Vec<ast::Stmt> = block_stmt.stmts().collect();

        for stmt in &statements {
            self.predeclare_function_decl_in_stmt(stmt)?;
        }

        for stmt in statements {
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
        self.lower_functions_in_expr(&rhs, None, None)?;

        let mut uses = self.resolver.collect_expr_uses(rhs);

        if is_compound_assignment(&assign) && !uses.contains(&defined_symbol) {
            uses.push(defined_symbol);
        }

        self.builder
            .build_assignment(span, vec![defined_symbol], uses);

        Ok(true)
    }

    fn predeclare_function_decl_in_stmt(&mut self, stmt: &ast::Stmt) -> Result<(), String> {
        let ast::Stmt::Decl(decl) = stmt.clone() else {
            return Ok(());
        };

        if decl.syntax().kind() != SyntaxKind::FN_DECL {
            return Ok(());
        }

        let name = function_declared_name(decl.syntax())
            .ok_or("function declaration missing name")?;

        let span = Span::from_node(decl.syntax());

        if !self.resolver.is_declared_in_current_scope(&name) {
            self.resolver.declare(name, SymbolKind::Function, span)?;
        }

        Ok(())
    }

    fn lower_function_decl_in_stmt(&mut self, stmt: &ast::Stmt) -> Result<bool, String> {
        let ast::Stmt::Decl(decl) = stmt.clone() else {
            return Ok(false);
        };

        if decl.syntax().kind() != SyntaxKind::FN_DECL {
            return Ok(false);
        }

        let name = function_declared_name(decl.syntax())
            .ok_or("function declaration missing name")?;

        let span = Span::from_node(decl.syntax());

        let symbol_id = if let Some(symbol_id) = self.resolver.resolve(&name) {
            symbol_id
        } else {
            self.resolver.declare(name.clone(), SymbolKind::Function, span)?
        };

        self.lower_function_like_node(
            decl.syntax(),
            Some(name),
            Some(symbol_id),
        )?;

        Ok(true)
    }

    fn lower_functions_in_expr(
        &mut self,
        expr: &ast::Expr,
        fallback_name: Option<String>,
        fallback_symbol_id: Option<SymbolId>,
    ) -> Result<(), String> {
        if is_function_like_node(expr.syntax()) {
            self.lower_function_like_node(expr.syntax(), fallback_name, fallback_symbol_id)
        } else {
            self.lower_direct_nested_functions_in_node(expr.syntax())
        }
    }

    fn lower_direct_nested_functions_in_node(&mut self, node: &SyntaxNode) -> Result<(), String> {
        let mut function_nodes = Vec::new();
        collect_direct_function_like_nodes(node, &mut function_nodes);

        for function_node in function_nodes {
            self.lower_function_like_node(&function_node, None, None)?;
        }

        Ok(())
    }

    fn lower_function_like_node(
        &mut self,
        node: &SyntaxNode,
        fallback_name: Option<String>,
        fallback_symbol_id: Option<SymbolId>,
    ) -> Result<(), String> {
        let span = Span::from_node(node);

        let declared_name = function_declared_name(node);
        let metadata_name = fallback_name.or_else(|| declared_name.clone());

        let metadata_symbol_id = if node.kind() == SyntaxKind::FN_DECL {
            fallback_symbol_id.or_else(|| {
                declared_name
                    .as_ref()
                    .and_then(|name| self.resolver.resolve(name))
            })
        } else {
            fallback_symbol_id
        };

        let param_names = function_param_names(node);
        let mut fn_builder = CfgBuilder::new();

        self.resolver.enter_scope();

        if node.kind() == SyntaxKind::FN_EXPR {
            if let Some(inner_name) = declared_name {
                if !self.resolver.is_declared_in_current_scope(&inner_name) {
                    let _ = self.resolver.declare(inner_name, SymbolKind::Function, span)?;
                }
            }
        }

        let mut param_ids = Vec::new();
        for param_name in param_names {
            let param_id = self.resolver.declare(param_name, SymbolKind::Param, span)?;
            param_ids.push(param_id);
        }

        {
            let mut nested = AstLowerer {
                resolver: self.resolver,
                builder: &mut fn_builder,
                functions: self.functions,
            };

            match function_body(node)? {
                FunctionBody::Block(block_stmt) => {
                    nested.lower_block_stmt(&block_stmt)?;
                }
                FunctionBody::Expr(expr) => {
                    nested.lower_functions_in_expr(&expr, None, None)?;
                    let uses = nested.resolver.collect_expr_uses(expr);
                    nested.builder.build_return(span, uses);
                }
            }
        }

        self.resolver.exit_scope();

        let cfg = fn_builder.finish();
        let liveness = compute_liveness(&cfg);

        self.functions.push(FunctionAnalysis {
            name: metadata_name,
            symbol_id: metadata_symbol_id,
            span,
            params: param_ids,
            body: BodyAnalysis { cfg, liveness },
        });

        Ok(())
    }
}

enum FunctionBody {
    Block(ast::BlockStmt),
    Expr(ast::Expr),
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

fn is_function_like_node(node: &SyntaxNode) -> bool {
    matches!(
        node.kind(),
        SyntaxKind::FN_DECL | SyntaxKind::FN_EXPR | SyntaxKind::ARROW_EXPR
    )
}

fn collect_direct_function_like_nodes(node: &SyntaxNode, out: &mut Vec<SyntaxNode>) {
    for child in node.children() {
        if matches!(child.kind(), SyntaxKind::FN_EXPR | SyntaxKind::ARROW_EXPR) {
            out.push(child);
            continue;
        }

        collect_direct_function_like_nodes(&child, out);
    }
}

fn function_declared_name(node: &SyntaxNode) -> Option<String> {
    let params_start = node
        .descendants()
        .find(|n| n.kind() == SyntaxKind::PARAMETER_LIST)
        .map(|n| n.text_range().start());

    node.descendants_with_tokens().find_map(|elem| match elem {
        NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
            if let Some(start) = params_start {
                if tok.text_range().start() < start {
                    Some(tok.text().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    })
}

fn function_param_names(node: &SyntaxNode) -> Vec<String> {
    if let Some(param_list) = node.descendants().find(|n| n.kind() == SyntaxKind::PARAMETER_LIST) {
        return param_list
            .descendants_with_tokens()
            .filter_map(|elem| match elem {
                NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::IDENT => {
                    Some(tok.text().to_string())
                }
                _ => None,
            })
            .collect();
    }

    if node.kind() == SyntaxKind::ARROW_EXPR {
        let mut params = Vec::new();

        for elem in node.children_with_tokens() {
            match elem {
                NodeOrToken::Token(tok) => {
                    if tok.text().as_str() == "=>" {
                        break;
                    }

                    if tok.kind() == SyntaxKind::IDENT {
                        params.push(tok.text().to_string());
                    }
                }
                NodeOrToken::Node(_) => {}
            }
        }

        return params;
    }

    Vec::new()
}

fn function_body(node: &SyntaxNode) -> Result<FunctionBody, String> {
    match node.kind() {
        SyntaxKind::FN_DECL | SyntaxKind::FN_EXPR => {
            let block = node
                .children()
                .find_map(ast::BlockStmt::cast)
                .or_else(|| node.descendants().find_map(ast::BlockStmt::cast))
                .ok_or("function body block not found")?;

            Ok(FunctionBody::Block(block))
        }

        SyntaxKind::ARROW_EXPR => {
            let mut seen_arrow = false;

            for elem in node.children_with_tokens() {
                match elem {
                    NodeOrToken::Token(tok) => {
                        if tok.text().as_str() == "=>" {
                            seen_arrow = true;
                        }
                    }

                    NodeOrToken::Node(child) if seen_arrow => {
                        if let Some(block) = ast::BlockStmt::cast(child.clone()) {
                            return Ok(FunctionBody::Block(block));
                        }

                        if let Some(expr) = ast::Expr::cast(child) {
                            return Ok(FunctionBody::Expr(expr));
                        }
                    }

                    _ => {}
                }
            }

            Err("arrow function body not found".to_string())
        }

        _ => Err("node is not a function-like node".to_string()),
    }
}