use crate::cfg::basic_blocks::{BasicBlock, BlockId, Cfg, Span, StatementId, StatementInfo, StatementKind, SymbolId, Terminator};
use crate::cfg::liveness::compute_reachable_blocks;

pub struct CfgBuilder {
    cfg: Cfg,
    current: BlockId,
    next_statement_id: StatementId,
    loop_stack: Vec<LoopContext>,
    current_exception_target: Option<BlockId>,
}

#[derive(Debug, Clone)]
struct LoopContext {
    break_target: BlockId,
    continue_target: BlockId,
}

#[derive(Debug, Clone)]
pub enum LoopCondition {
    AlwaysTrue,
    AlwaysFalse,
    Unknown(Vec<SymbolId>),
}

impl CfgBuilder {
    pub fn new() -> Self {
        let entry = 0;
        let exit = 1;

        let blocks = vec![
            BasicBlock {
                id: entry,
                statements: vec![],
                terminator: Terminator::Unset,
                predecessors: vec![],
                exception_successors: vec![],
            },
            BasicBlock {
                id: exit,
                statements: vec![],
                terminator: Terminator::Exit,
                predecessors: vec![],
                exception_successors: vec![],
            },
        ];

        Self {
            cfg: Cfg { entry, exit, blocks },
            current: entry,
            next_statement_id: 0,
            loop_stack: vec![],
            current_exception_target: None
        }
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.cfg.blocks.len();
        self.cfg.blocks.push(BasicBlock {
            id,
            statements: vec![],
            terminator: Terminator::Unset,
            predecessors: vec![],
            exception_successors: vec![],
        });
        id
    }

    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        if !self.cfg.blocks[to].predecessors.contains(&from) {
            self.cfg.blocks[to].predecessors.push(from);
        }
    }

    fn add_exception_edge(&mut self, from: BlockId, to: BlockId) {
        if !self.cfg.blocks[from].exception_successors.contains(&to) {
            self.cfg.blocks[from].exception_successors.push(to);
        }

        self.add_edge(from, to);
    }

    fn register_possible_exception(&mut self) {
        if let Some(catch_target) = self.current_exception_target {
            self.add_exception_edge(self.current, catch_target);
        }
    }

    fn switch_to_block(&mut self, block_id: BlockId) {
        debug_assert!(block_id < self.cfg.blocks.len());
        self.current = block_id;
    }

    fn terminate_current(&mut self, term: Terminator) {
        let from = self.current;
        debug_assert!(matches!(self.cfg.blocks[from].terminator, Terminator::Unset));
        match &term {
            Terminator::Goto(to) => self.add_edge(from, *to),
            Terminator::Branch { then_bb, else_bb, .. } => {
                self.register_possible_exception();
                self.add_edge(from, *then_bb);
                self.add_edge(from, *else_bb);
            }
            Terminator::Return { .. } => {
                self.add_edge(from, self.cfg.exit);
            }
            Terminator::Unset => {}
            Terminator::Exit => {}
        }
        self.cfg.blocks[from].terminator = term;
    }

    fn current_block_is_open(&self) -> bool {
        matches!(self.cfg.blocks[self.current].terminator, Terminator::Unset)
    }

    pub fn finish(mut self) -> Cfg {
        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(self.cfg.exit));
        }

        prune_unreachable_empty_blocks(self.cfg)
    }

    fn push_statement(&mut self, kind: StatementKind, span: Span, defines: Vec<SymbolId>, uses: Vec<SymbolId>) {
        let id = self.next_statement_id;
        self.next_statement_id += 1;
        self.cfg.blocks[self.current].statements.push(StatementInfo {
            id,
            kind,
            span,
            defines,
            uses,
        });

        self.register_possible_exception();
    }

    pub fn build_return(&mut self, span: Span, uses: Vec<SymbolId>, has_value: bool) {
        self.push_statement(
            StatementKind::Return { has_value },
            span,
            vec![],
            uses.clone(),
        );

        self.terminate_current(Terminator::Return {
            has_value,
            value_uses: uses,
        });

        let next_block = self.new_block();
        self.current = next_block;
    }

    pub fn build_expression_statement(&mut self, span: Span, uses: Vec<SymbolId>) {
        self.push_statement(
            StatementKind::Expr,
            span,
            vec![],
            uses,
        );
    }

    pub fn build_variable_declaration(
        &mut self,
        span: Span,
        defines: Vec<SymbolId>,
        uses: Vec<SymbolId>,
        has_initializer: bool,
    ) {
        let stmt = StatementInfo {
            id: self.next_statement_id,
            kind: StatementKind::VarDecl { has_initializer },
            span,
            defines,
            uses,
        };

        self.next_statement_id += 1;
        self.cfg.blocks[self.current].statements.push(stmt);
        self.register_possible_exception();
    }

    pub fn build_assignment(
        &mut self,
        span: Span,
        defines: Vec<SymbolId>,
        uses: Vec<SymbolId>,
    ) {
        self.push_statement(
            StatementKind::Assign,
            span,
            defines,
            uses,
        );
    }

    pub fn build_update(&mut self, span: Span, symbol_id: SymbolId) {
        let stmt = StatementInfo {
            id: self.next_statement_id,
            kind: StatementKind::Update,
            span,
            defines: vec![symbol_id],
            uses: vec![symbol_id],
        };

        self.next_statement_id += 1;
        self.cfg.blocks[self.current].statements.push(stmt);

        self.register_possible_exception();
    }

    pub fn build_if<F, G>(
        &mut self,
        condition_uses: Vec<SymbolId>,
        then_builder: F,
        else_builder: Option<G>,
    ) where
        F: FnOnce(&mut CfgBuilder),
        G: FnOnce(&mut CfgBuilder),
    {
        let then_block = self.new_block();
        let else_block = self.new_block();

        self.terminate_current(Terminator::Branch {
            cond_uses: condition_uses,
            then_bb: then_block,
            else_bb: else_block,
        });

        self.switch_to_block(then_block);
        then_builder(self);
        let then_end = self.current;
        let then_open = self.current_block_is_open();

        self.switch_to_block(else_block);
        if let Some(build_else_branch) = else_builder {
            build_else_branch(self);
        }
        let else_end = self.current;
        let else_open = self.current_block_is_open();

        if then_open || else_open {
            let join_block = self.new_block();

            if then_open {
                self.switch_to_block(then_end);
                self.terminate_current(Terminator::Goto(join_block));
            }

            if else_open {
                self.switch_to_block(else_end);
                self.terminate_current(Terminator::Goto(join_block));
            }

            self.switch_to_block(join_block);
        } else {
            let dead_continuation = self.new_block();
            self.switch_to_block(dead_continuation);
        }
    }

    pub fn build_break(&mut self, span: Span) {
        self.push_statement(
            StatementKind::Break,
            span,
            vec![],
            vec![],
        );

        let loop_context = self
            .loop_stack
            .last()
            .expect("break used outside of a loop");

        self.terminate_current(Terminator::Goto(loop_context.break_target));

        let next_block = self.new_block();
        self.switch_to_block(next_block);
    }

    pub fn build_continue(&mut self, span: Span) {
        self.push_statement(
            StatementKind::Continue,
            span,
            vec![],
            vec![],
        );

        let loop_context = self
            .loop_stack
            .last()
            .expect("continue used outside of a loop");

        self.terminate_current(Terminator::Goto(loop_context.continue_target));

        let next_block = self.new_block();
        self.switch_to_block(next_block);
    }

    pub fn build_try_catch<F, G>(
        &mut self,
        try_builder: F,
        catch_builder: G,
    )
    where
        F: FnOnce(&mut CfgBuilder),
        G: FnOnce(&mut CfgBuilder),
    {
        let try_block = self.new_block();
        let catch_block = self.new_block();

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(try_block));
        }

        let old_exception_target = self.current_exception_target;
        self.current_exception_target = Some(catch_block);

        self.switch_to_block(try_block);
        try_builder(self);
        let try_end = self.current;
        let try_open = self.current_block_is_open();

        self.current_exception_target = old_exception_target;

        self.switch_to_block(catch_block);
        catch_builder(self);
        let catch_end = self.current;
        let catch_open = self.current_block_is_open();

        if try_open || catch_open {
            let join_block = self.new_block();

            if try_open {
                self.switch_to_block(try_end);
                self.terminate_current(Terminator::Goto(join_block));
            }

            if catch_open {
                self.switch_to_block(catch_end);
                self.terminate_current(Terminator::Goto(join_block));
            }

            self.switch_to_block(join_block);
        } else {
            let dead_continuation = self.new_block();
            self.switch_to_block(dead_continuation);
        }
    }

    pub fn build_while<F>(
        &mut self,
        condition: LoopCondition,
        body_builder: F,
    ) where
        F: FnOnce(&mut CfgBuilder),
    {
        let condition_block = self.new_block();
        let body_block = self.new_block();
        let exit_block = self.new_block();

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(condition_block));
        }

        self.switch_to_block(condition_block);

        match condition {
            LoopCondition::AlwaysTrue => {
                self.terminate_current(Terminator::Goto(body_block));
            }

            LoopCondition::AlwaysFalse => {
                self.terminate_current(Terminator::Goto(exit_block));
            }

            LoopCondition::Unknown(condition_uses) => {
                self.terminate_current(Terminator::Branch {
                    cond_uses: condition_uses,
                    then_bb: body_block,
                    else_bb: exit_block,
                });
            }
        }

        self.loop_stack.push(LoopContext {
            break_target: exit_block,
            continue_target: condition_block,
        });

        self.switch_to_block(body_block);
        body_builder(self);

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(condition_block));
        }

        self.loop_stack.pop();

        self.switch_to_block(exit_block);
    }


    pub fn build_for<F, G>(
        &mut self,
        condition_uses: Option<Vec<SymbolId>>,
        body_builder: F,
        update_builder: Option<G>,
    ) where
        F: FnOnce(&mut CfgBuilder),
        G: FnOnce(&mut CfgBuilder),
    {
        let condition_block = self.new_block();
        let body_block = self.new_block();
        let has_update = update_builder.is_some();
        let update_block = if has_update {
            self.new_block()
        } else {
            condition_block
        };
        let exit_block = self.new_block();

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(condition_block));
        }

        self.switch_to_block(condition_block);

        if let Some(cond_uses) = condition_uses {
            self.terminate_current(Terminator::Branch {
                cond_uses,
                then_bb: body_block,
                else_bb: exit_block,
            });
        } else {
            self.terminate_current(Terminator::Goto(body_block));
        }

        self.loop_stack.push(LoopContext {
            break_target: exit_block,
            continue_target: update_block,
        });

        self.switch_to_block(body_block);
        body_builder(self);

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(update_block));
        }

        if let Some(build_update) = update_builder {
            self.switch_to_block(update_block);
            build_update(self);

            if self.current_block_is_open() {
                self.terminate_current(Terminator::Goto(condition_block));
            }
        }

        self.loop_stack.pop();

        self.switch_to_block(exit_block);
    }

    pub fn build_for_each<F>(
        &mut self,
        source_uses: Vec<SymbolId>,
        target_symbol: SymbolId,
        assign_span: Span,
        body_builder: F,
    ) where
        F: FnOnce(&mut CfgBuilder),
    {
        let header_block = self.new_block();
        let body_block = self.new_block();
        let exit_block = self.new_block();

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(header_block));
        }

        self.switch_to_block(header_block);
        self.terminate_current(Terminator::Branch {
            cond_uses: source_uses,
            then_bb: body_block,
            else_bb: exit_block,
        });

        self.loop_stack.push(LoopContext {
            break_target: exit_block,
            continue_target: header_block,
        });

        self.switch_to_block(body_block);

        self.build_assignment(assign_span, vec![target_symbol], vec![]);

        body_builder(self);

        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(header_block));
        }

        self.loop_stack.pop();

        self.switch_to_block(exit_block);
    }
}

fn prune_unreachable_empty_blocks(cfg: Cfg) -> Cfg {
    let reachable = compute_reachable_blocks(&cfg);

    let mut keep = vec![true; cfg.blocks.len()];

    for block in &cfg.blocks {
        if block.id == cfg.entry || block.id == cfg.exit {
            continue;
        }

        if !reachable.contains(&block.id) && block.statements.is_empty() {
            keep[block.id] = false;
        }
    }

    if keep.iter().all(|&k| k) {
        return cfg;
    }

    let mut remap = vec![usize::MAX; cfg.blocks.len()];
    let mut new_blocks = Vec::new();

    for block in &cfg.blocks {
        if keep[block.id] {
            let new_id = new_blocks.len();
            remap[block.id] = new_id;

            new_blocks.push(BasicBlock {
                id: new_id,
                statements: block.statements.clone(),
                terminator: block.terminator.clone(),
                predecessors: Vec::new(),
                exception_successors: Vec::new()
            });
        }
    }

    for block in &mut new_blocks {
        block.terminator = remap_terminator(&block.terminator, &remap);
    }

    for old_block in &cfg.blocks {
        if !keep[old_block.id] {
            continue;
        }

        let new_id = remap[old_block.id];

        new_blocks[new_id].exception_successors = old_block
            .exception_successors
            .iter()
            .filter_map(|&target| {
                if keep[target] {
                    Some(remap[target])
                } else {
                    None
                }
            })
            .collect();
    }

    for old_block in &cfg.blocks {
        if !keep[old_block.id] {
            continue;
        }

        let new_id = remap[old_block.id];

        new_blocks[new_id].predecessors = old_block
            .predecessors
            .iter()
            .filter_map(|&pred| {
                if keep[pred] {
                    Some(remap[pred])
                } else {
                    None
                }
            })
            .collect();
    }

    Cfg {
        entry: remap[cfg.entry],
        exit: remap[cfg.exit],
        blocks: new_blocks,
    }
}

fn remap_terminator(term: &Terminator, remap: &[BlockId]) -> Terminator {
    match term {
        Terminator::Goto(target) => Terminator::Goto(remap[*target]),
        Terminator::Branch {
            cond_uses,
            then_bb,
            else_bb,
        } => Terminator::Branch {
            cond_uses: cond_uses.clone(),
            then_bb: remap[*then_bb],
            else_bb: remap[*else_bb],
        },
        Terminator::Return { has_value, value_uses } => Terminator::Return {
            has_value: *has_value,
            value_uses: value_uses.clone(),
        },
        Terminator::Unset => Terminator::Unset,
        Terminator::Exit => Terminator::Exit,
    }
}