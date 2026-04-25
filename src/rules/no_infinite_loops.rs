use std::collections::{HashSet, VecDeque};

use crate::cfg::basic_blocks::{BlockId, Cfg, Span};
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

pub struct NoInfiniteLoops;

impl Rule for NoInfiniteLoops {
    fn name(&self) -> &'static str {
        "no-infinite-loops"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let Some(semantic) = ctx.semantic() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        collect_infinite_loops_from_body(
            ctx,
            &semantic.script.cfg,
            &semantic.script.liveness.reachable,
            None,
            Span { start: 0, end: 0 },
            &mut diagnostics,
        );

        for function in &semantic.functions {
            collect_infinite_loops_from_body(
                ctx,
                &function.body.cfg,
                &function.body.liveness.reachable,
                function.name.as_deref(),
                function.span,
                &mut diagnostics,
            );
        }

        diagnostics
    }
}

fn collect_infinite_loops_from_body(
    ctx: &RuleContext,
    cfg: &Cfg,
    reachable: &HashSet<BlockId>,
    function_name: Option<&str>,
    fallback_span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let can_reach_exit = compute_can_reach_exit(cfg, reachable);
    let sccs = compute_sccs(cfg, reachable);

    for scc in sccs {
        if !is_cyclic_scc(cfg, &scc) {
            continue;
        }

        // Ako ijedan blok iz SCC moze da dodje do exit-a,
        // onda petlja ima neku izlaznu putanju i ne prijavljujemo je.
        if scc.iter().any(|block_id| can_reach_exit.contains(block_id)) {
            continue;
        }

        let span = pick_span_for_scc(cfg, &scc).unwrap_or(fallback_span);

        let message = match function_name {
            Some(name) => format!(
                "Potential infinite loop in function '{}': loop has no reachable exit path",
                name
            ),
            None => "Potential infinite loop: loop has no reachable exit path".to_string(),
        };

        diagnostics.push(ctx.diagnostic_at_span(span, message));
    }
}

fn compute_can_reach_exit(cfg: &Cfg, reachable: &HashSet<BlockId>) -> HashSet<BlockId> {
    let mut can_reach_exit = HashSet::new();
    let mut queue = VecDeque::new();

    can_reach_exit.insert(cfg.exit);
    queue.push_back(cfg.exit);

    while let Some(block_id) = queue.pop_front() {
        for pred in &cfg.blocks[block_id].predecessors {
            if !reachable.contains(pred) {
                continue;
            }

            if can_reach_exit.insert(*pred) {
                queue.push_back(*pred);
            }
        }
    }

    can_reach_exit
}

fn compute_sccs(cfg: &Cfg, reachable: &HashSet<BlockId>) -> Vec<Vec<BlockId>> {
    let mut visited = HashSet::new();
    let mut order = Vec::new();

    for block in &cfg.blocks {
        if reachable.contains(&block.id) && !visited.contains(&block.id) {
            dfs_order(cfg, reachable, block.id, &mut visited, &mut order);
        }
    }

    let mut visited_rev = HashSet::new();
    let mut sccs = Vec::new();

    while let Some(block_id) = order.pop() {
        if visited_rev.contains(&block_id) {
            continue;
        }

        let mut component = Vec::new();
        dfs_reverse(cfg, reachable, block_id, &mut visited_rev, &mut component);
        sccs.push(component);
    }

    sccs
}

fn dfs_order(
    cfg: &Cfg,
    reachable: &HashSet<BlockId>,
    block_id: BlockId,
    visited: &mut HashSet<BlockId>,
    order: &mut Vec<BlockId>,
) {
    if !visited.insert(block_id) {
        return;
    }

    for succ in cfg.blocks[block_id].successors(cfg.exit) {
        if reachable.contains(&succ) {
            dfs_order(cfg, reachable, succ, visited, order);
        }
    }

    order.push(block_id);
}

fn dfs_reverse(
    cfg: &Cfg,
    reachable: &HashSet<BlockId>,
    block_id: BlockId,
    visited: &mut HashSet<BlockId>,
    component: &mut Vec<BlockId>,
) {
    if !visited.insert(block_id) {
        return;
    }

    component.push(block_id);

    for pred in &cfg.blocks[block_id].predecessors {
        if reachable.contains(pred) {
            dfs_reverse(cfg, reachable, *pred, visited, component);
        }
    }
}

fn is_cyclic_scc(cfg: &Cfg, scc: &[BlockId]) -> bool {
    if scc.len() > 1 {
        return true;
    }

    let block_id = scc[0];
    cfg.blocks[block_id]
        .successors(cfg.exit)
        .into_iter()
        .any(|succ| succ == block_id)
}

fn pick_span_for_scc(cfg: &Cfg, scc: &[BlockId]) -> Option<Span> {
    let mut best: Option<Span> = None;

    for block_id in scc {
        for stmt in &cfg.blocks[*block_id].statements {
            match best {
                Some(current) if stmt.span.start >= current.start => {}
                _ => best = Some(stmt.span),
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::make_context;

    #[test]
    fn flags_obvious_infinite_for_loop_in_function() {
        let ctx = make_context(
            r#"
            function test() {
                let x = 0;
                for (;;) {
                    x = x + 1;
                }
            }

            test();
            "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_for_loop_with_break() {
        let ctx = make_context(
            r#"
            function test() {
                let x = 0;
                for (;;) {
                    if (x > 10) {
                        break;
                    }
                    x = x + 1;
                }

                return x;
            }

            test();
            "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_flag_normal_for_loop() {
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

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn flags_top_level_infinite_for_loop() {
        let ctx = make_context(
            r#"
            let x = 0;

            for (;;) {
                x = x + 1;
            }
            "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
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

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn flags_while_true_without_exit() {
        let ctx = make_context(
            r#"
        function test() {
            let x = 0;

            while (true) {
                x = x + 1;
            }
        }

        test();
        "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_while_true_with_break() {
        let ctx = make_context(
            r#"
        function test(x) {
            while (true) {
                if (x > 10) {
                    break;
                }

                x = x + 1;
            }

            return x;
        }

        test(1);
        "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn while_false_body_is_unreachable_not_infinite() {
        let ctx = make_context(
            r#"
        function test() {
            while (false) {
                console.log("never");
            }

            return 1;
        }

        test();
        "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn flags_while_one_without_exit() {
        let ctx = make_context(
            r#"
        function test() {
            while (1) {
                console.log("loop");
            }
        }

        test();
        "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn flags_while_non_empty_string_without_exit() {
        let ctx = make_context(
            r#"
        function test() {
            while ("yes") {
                console.log("loop");
            }
        }

        test();
        "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_while_zero() {
        let ctx = make_context(
            r#"
        function test() {
            while (0) {
                console.log("never");
            }

            return 1;
        }

        test();
        "#,
            None,
        );

        let rule = NoInfiniteLoops;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }
}