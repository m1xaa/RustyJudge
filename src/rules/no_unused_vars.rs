use std::collections::HashSet;

use rslint_parser::TextRange;

use crate::cfg::basic_blocks::{Cfg, Span, SymbolId, SymbolKind, Terminator};
use crate::cfg::liveness::LivenessResult;
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

pub struct NoUnusedVars;

impl Rule for NoUnusedVars {
    fn name(&self) -> &'static str {
        "no-unused-vars"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let Some(semantic) = ctx.semantic() else {
            return Vec::new();
        };

        let mut used_symbols = HashSet::new();

        collect_used_symbols_from_body(
            semantic.script_cfg(),
            semantic.script_liveness(),
            &mut used_symbols,
        );

        for function in &semantic.functions {
            collect_used_symbols_from_body(
                &function.body.cfg,
                &function.body.liveness,
                &mut used_symbols,
            );
        }

        let mut diagnostics = Vec::new();

        for symbol in &semantic.symbols.symbols {
            if !should_check_symbol(symbol.kind) {
                continue;
            }

            if used_symbols.contains(&symbol.id) {
                continue;
            }

            diagnostics.push(ctx.diagnostic_at(
                span_to_text_range(symbol.decl_span),
                unused_message(symbol.kind, &symbol.name),
            ));
        }

        diagnostics
    }
}

fn collect_used_symbols_from_body(
    cfg: &Cfg,
    liveness: &LivenessResult,
    used_symbols: &mut HashSet<SymbolId>,
) {
    for block in &cfg.blocks {
        if !liveness.reachable.contains(&block.id) {
            continue;
        }

        for stmt in &block.statements {
            used_symbols.extend(stmt.uses.iter().copied());
        }

        match &block.terminator {
            Terminator::Branch { cond_uses, .. } => {
                used_symbols.extend(cond_uses.iter().copied());
            }
            Terminator::Return { value_uses, .. } => {
                used_symbols.extend(value_uses.iter().copied());
            }
            Terminator::Goto(_) | Terminator::Unset | Terminator::Exit => {}
        }
    }
}

fn should_check_symbol(kind: SymbolKind) -> bool {
    matches!(
        kind,
        SymbolKind::Var
            | SymbolKind::Let
            | SymbolKind::Const
            | SymbolKind::Param
            | SymbolKind::Function
    )
}

fn unused_message(kind: SymbolKind, name: &str) -> String {
    match kind {
        SymbolKind::Param => format!("Unused parameter '{}'", name),
        SymbolKind::Function => format!("Unused function '{}'", name),
        SymbolKind::Var | SymbolKind::Let | SymbolKind::Const => {
            format!("Unused variable '{}'", name)
        }
    }
}

fn span_to_text_range(span: Span) -> TextRange {
    TextRange::new(span.start.into(), span.end.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::make_context;

    #[test]
    fn detects_unused_variable() {
        let ctx = make_context(
            r#"
            let x = 5;
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_used_variable() {
        let ctx = make_context(
            r#"
            let x = 5;
            console.log(x);
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_unused_parameter() {
        let ctx = make_context(
            r#"
            function test(a, b) {
                return a;
            }
            test(1,2);
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Unused parameter 'b'"));
    }

    #[test]
    fn handles_shadowing_correctly() {
        let ctx = make_context(
            r#"
            let x = 1;

            function test() {
                let x = 2;
                return x;
            }

            test();

            console.log(x);
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_unused_shadowed_variable() {
        let ctx = make_context(
            r#"
            let x = 1;

            function test() {
                let x = 2;
                return 0;
            }

            console.log(x);
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn reachable_uses_count_but_unreachable_only_uses_do_not() {
        let ctx = make_context(
            r#"
            function test() {
                let x = 1;
                return 0;
                console.log(x);
            }
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn does_not_flag_used_function() {
        let ctx = make_context(
            r#"
            function test() {
                return 1;
            }

            let x = test();
            console.log(x);
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn flags_unused_function() {
        let ctx = make_context(
            r#"
            function test() {
                return 1;
            }
            "#,
            None,
        );

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Unused function 'test'"));
    }
}