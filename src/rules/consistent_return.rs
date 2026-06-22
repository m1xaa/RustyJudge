use crate::cfg::basic_blocks::Terminator;
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

pub struct ConsistentReturn;

impl Rule for ConsistentReturn {
    fn name(&self) -> &'static str {
        "consistent-return"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let Some(semantic) = ctx.semantic() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        for function in &semantic.functions {
            let exit = function.body.cfg.exit;

            let reachable_exit_predecessors: Vec<_> = function
                .body
                .cfg
                .blocks[exit]
                .predecessors
                .iter()
                .copied()
                .filter(|block_id| function.body.liveness.reachable.contains(block_id))
                .collect();

            if reachable_exit_predecessors.is_empty() {
                continue;
            }

            let mut has_return_with_value = false;
            let mut has_return_without_value_or_fallthrough = false;

            for pred in reachable_exit_predecessors {
                match &function.body.cfg.blocks[pred].terminator {
                    Terminator::Return { has_value, .. } => {
                        if *has_value {
                            has_return_with_value = true;
                        } else {
                            has_return_without_value_or_fallthrough = true;
                        }
                    }

                    Terminator::Goto(target) if *target == exit => {
                        has_return_without_value_or_fallthrough = true;
                    }

                    Terminator::Branch { then_bb, else_bb, .. } => {
                        if *then_bb == exit || *else_bb == exit {
                            has_return_without_value_or_fallthrough = true;
                        }
                    }

                    _ => {}
                }
            }

            if has_return_with_value && has_return_without_value_or_fallthrough {
                let message = match &function.name {
                    Some(name) => format!(
                        "Inconsistent return in function '{}': some code paths return a value, others do not",
                        name
                    ),
                    None => "Inconsistent return: some code paths return a value, others do not"
                        .to_string(),
                };

                diagnostics.push(ctx.diagnostic_at_span(function.span, message));
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::make_context;

    #[test]
    fn flags_return_value_vs_fallthrough() {
        let ctx = make_context(
            r#"
            function getValue(x) {
                if (x > 0) {
                    return x;
                }
            }
            "#,
            None,
        );

        let rule = ConsistentReturn;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn flags_return_value_vs_empty_return() {
        let ctx = make_context(
            r#"
            function test(x) {
                if (x > 0) {
                    return 1;
                }
                return;
            }
            "#,
            None,
        );

        let rule = ConsistentReturn;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_when_all_paths_return_value() {
        let ctx = make_context(
            r#"
            function getValue(x) {
                if (x > 0) {
                    return x;
                }
                return 0;
            }
            "#,
            None,
        );

        let rule = ConsistentReturn;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_flag_when_all_paths_return_no_value() {
        let ctx = make_context(
            r#"
            function test(x) {
                if (x > 0) {
                    return;
                }
                return;
            }
            "#,
            None,
        );

        let rule = ConsistentReturn;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_flag_when_function_only_falls_through() {
        let ctx = make_context(
            r#"
            function logValue(x) {
                let y = x + 1;
                console.log(y);
            }
            "#,
            None,
        );

        let rule = ConsistentReturn;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_flag_when_both_branches_return_value() {
        let ctx = make_context(
            r#"
            function test(x) {
                if (x > 0) {
                    return 1;
                } else {
                    return 2;
                }
            }
            "#,
            None,
        );

        let rule = ConsistentReturn;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }
}