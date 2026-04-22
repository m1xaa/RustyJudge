use crate::cfg::basic_blocks::{Cfg, StatementKind};
use crate::cfg::liveness::LivenessResult;
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

pub struct NoDeadStores;

impl Rule for NoDeadStores {
    fn name(&self) -> &'static str {
        "no-dead-stores"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let Some(semantic) = ctx.semantic() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        collect_dead_stores_from_body(
            ctx,
            &semantic.script.cfg,
            &semantic.script.liveness,
            None,
            &mut diagnostics,
        );

        for function in &semantic.functions {
            collect_dead_stores_from_body(
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

fn collect_dead_stores_from_body(
    ctx: &RuleContext,
    cfg: &Cfg,
    liveness: &LivenessResult,
    function_name: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for block in &cfg.blocks {
        if !liveness.reachable.contains(&block.id) {
            continue;
        }

        for (index, stmt) in block.statements.iter().enumerate() {
            if !is_store_statement(&stmt.kind) {
                continue;
            }

            let live_out = &liveness.stmt_live_out[block.id][index];

            for defined in &stmt.defines {
                if live_out.contains(defined) {
                    continue;
                }

                let Some(symbol) = ctx.semantic().and_then(|s| s.symbol(*defined)) else {
                    continue;
                };

                let message = match function_name {
                    Some(name) => format!(
                        "Dead store in function '{}': value assigned to '{}' is never read",
                        name, symbol.name
                    ),
                    None => format!(
                        "Dead store: value assigned to '{}' is never read",
                        symbol.name
                    ),
                };

                diagnostics.push(ctx.diagnostic_at_span(stmt.span, message));
            }
        }
    }
}

fn is_store_statement(kind: &StatementKind) -> bool {
    matches!(
        kind,
        StatementKind::Assign
            | StatementKind::Update
            | StatementKind::VarDecl { has_initializer: true }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::make_context;

    #[test]
    fn detects_overwrite_chain() {
        let ctx = make_context(
            r#"
            let x = 1;
            x = 2;
            x = 3;
            console.log(x);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn does_not_flag_when_values_are_used_between_assignments() {
        let ctx = make_context(
            r#"
            let x = 1;
            console.log(x);
            x = 2;
            console.log(x);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_dead_store_inside_function() {
        let ctx = make_context(
            r#"
            function test() {
                let x = 1;
                x = 2;
                x = 3;
                return x;
            }

            test();
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn does_not_flag_when_if_branch_uses_value_before_overwrite() {
        let ctx = make_context(
            r#"
            function test(flag) {
                let x = 1;

                if (flag) {
                    console.log(x);
                }

                x = 2;
                return x;
            }

            test(true);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_dead_initial_value_when_both_branches_overwrite() {
        let ctx = make_context(
            r#"
            function test(flag) {
                let x = 1;

                if (flag) {
                    x = 2;
                } else {
                    x = 3;
                }

                return x;
            }

            test(true);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_when_one_branch_reads_the_value() {
        let ctx = make_context(
            r#"
            function test(flag) {
                let x = 1;

                if (flag) {
                    console.log(x);
                } else {
                    x = 2;
                }

                return x;
            }

            test(true);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_flag_simple_while_loop() {
        let ctx = make_context(
            r#"
            function test(n) {
                let x = n;

                while (x > 0) {
                    x = x - 1;
                }

                return x;
            }

            test(3);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_dead_store_in_while_body() {
        let ctx = make_context(
            r#"
            function test() {
                let x = 0;

                while (x < 3) {
                    let y = 1;
                    y = 2;
                    x = x + 1;
                }

                return x;
            }

            test();
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn does_not_flag_simple_for_loop() {
        let ctx = make_context(
            r#"
            function test(n) {
                let sum = 0;

                for (let i = 0; i < n; i++) {
                    sum = sum + i;
                }

                return sum;
            }

            test(5);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_dead_store_in_for_body() {
        let ctx = make_context(
            r#"
            function test(n) {
                let sum = 0;

                for (let i = 0; i < n; i++) {
                    let temp = i;
                    temp = i + 1;
                    sum = sum + i;
                }

                return sum;
            }

            test(5);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn detects_dead_store_in_for_of_body() {
        let ctx = make_context(
            r#"
            function test(values) {
                let total = 0;

                for (const value of values) {
                    let tmp = value;
                    tmp = value + 1;
                    total = total + value;
                }

                return total;
            }

            test([1, 2, 3]);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn handles_shadowing_correctly() {
        let ctx = make_context(
            r#"
            let x = 1;

            function test() {
                let x = 2;
                x = 3;
                return x;
            }

            console.log(x);
            test();
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_unreachable_store_but_flags_reachable_dead_store() {
        let ctx = make_context(
            r#"
            function test() {
                let x = 1;
                return 0;
                x = 2;
            }

            test();
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn complex_mixed_case() {
        let ctx = make_context(
            r#"
            function complex(flag, values) {
                let result = 0;
                let temp = 1;

                if (flag) {
                    temp = 2;
                    result = temp;
                } else {
                    temp = 3;
                }

                for (const value of values) {
                    let local = value;
                    local = value + 1;
                    result = result + value;
                }

                return result;
            }

            complex(true, [1, 2, 3]);
            "#,
            None,
        );

        let rule = NoDeadStores;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 4);
    }
}