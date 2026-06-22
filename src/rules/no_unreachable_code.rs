use crate::cfg::basic_blocks::Cfg;
use crate::cfg::liveness::LivenessResult;
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

pub struct NoUnreachableCode;

impl Rule for NoUnreachableCode {
    fn name(&self) -> &'static str {
        "no-unreachable-code"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let Some(semantic) = ctx.semantic() else {
            return diagnostics;
        };

        collect_unreachable_from_body(
            ctx,
            semantic.script_cfg(),
            semantic.script_liveness(),
            None,
            &mut diagnostics,
        );

        for function in &semantic.functions {
            collect_unreachable_from_body(
                ctx,
                &function.body.cfg,
                &function.body.liveness,
                function.name.as_deref(),
                &mut diagnostics,
            );
        }

        diagnostics
    }
}

fn collect_unreachable_from_body(
    ctx: &RuleContext,
    cfg: &Cfg,
    liveness: &LivenessResult,
    function_name: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for block in &cfg.blocks {
        if liveness.reachable.contains(&block.id) {
            continue;
        }

        for stmt in &block.statements {
            let message = match function_name {
                Some(name) => format!("Unreachable code in function '{}'", name),
                None => "Unreachable code".to_string(),
            };

            diagnostics.push(ctx.diagnostic_at_span(stmt.span, message));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::make_context;

    #[test]
    fn detects_unreachable_after_return() {
        let ctx = make_context(
            r#"
            function test() {
                return 1;
                let x = 2;
            }
            "#,
            None,
        );

        let rule = NoUnreachableCode;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_multiple_unreachable_statements() {
        let ctx = make_context(
            r#"
            function test() {
                return 1;
                let x = 2;
                x = 3;
            }
            "#,
            None,
        );

        let rule = NoUnreachableCode;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn does_not_flag_reachable_if_return_paths() {
        let ctx = make_context(
            r#"
            function test(x) {
                if (x > 0) {
                    return x;
                }
                return 0;
            }
            "#,
            None,
        );

        let rule = NoUnreachableCode;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_unreachable_after_if_both_branches_return() {
        let ctx = make_context(
            r#"
            function test(x) {
                if (x > 0) {
                    return 1;
                } else {
                    return 2;
                }

                let y = 10;
            }
            "#,
            None,
        );

        let rule = NoUnreachableCode;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_unreachable_after_continue() {
        let ctx = make_context(
            r#"
            function test() {
                while (true) {
                    continue;
                    let x = 1;
                }
            }
            "#,
            None,
        );

        let rule = NoUnreachableCode;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_unreachable_while_false_body() {
        let ctx = make_context(
            r#"
        function test() {
            while (false) {
                let x = 1;
            }

            return 0;
        }

        test();
        "#,
            None,
        );

        let rule = NoUnreachableCode;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }
}