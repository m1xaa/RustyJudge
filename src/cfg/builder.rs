use crate::cfg::basic_blocks::{BasicBlock, BlockId, Cfg, Span, StatementId, StatementInfo, StatementKind, SymbolId, Terminator};

pub struct CfgBuilder {
    cfg: Cfg,
    current: BlockId,
    next_statement_id: StatementId,
    loop_stack: Vec<LoopContext>,
}

#[derive(Debug, Clone)]
struct LoopContext {
    break_target: BlockId,
    continue_target: BlockId,
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
            },
            BasicBlock {
                id: exit,
                statements: vec![],
                terminator: Terminator::Exit,
                predecessors: vec![],
            },
        ];

        Self {
            cfg: Cfg { entry, exit, blocks },
            current: entry,
            next_statement_id: 0,
            loop_stack: vec![],
        }
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.cfg.blocks.len();
        self.cfg.blocks.push(BasicBlock {
            id,
            statements: vec![],
            terminator: Terminator::Unset,
            predecessors: vec![],
        });
        id
    }

    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        if !self.cfg.blocks[to].predecessors.contains(&from) {
            self.cfg.blocks[to].predecessors.push(from);
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

        self.cfg
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
    }

    pub fn build_return(&mut self, span: Span, uses: Vec<SymbolId>) {
        self.push_statement(
            StatementKind::Return,
            span,
            vec![],
            uses.clone(),
        );

        self.terminate_current(Terminator::Return {
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
    ) {
        self.push_statement(
            StatementKind::VarDecl,
            span,
            defines,
            uses,
        );
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
        let join_block = self.new_block();

        self.terminate_current(Terminator::Branch {
            cond_uses: condition_uses,
            then_bb: then_block,
            else_bb: else_block,
        });

        self.switch_to_block(then_block);
        then_builder(self);
        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(join_block));
        }

        self.switch_to_block(else_block);
        if let Some(build_else_branch) = else_builder {
            build_else_branch(self);
        }
        if self.current_block_is_open() {
            self.terminate_current(Terminator::Goto(join_block));
        }

        self.switch_to_block(join_block);
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

    pub fn build_while<F>(
        &mut self,
        condition_uses: Vec<SymbolId>,
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
        self.terminate_current(Terminator::Branch {
            cond_uses: condition_uses,
            then_bb: body_block,
            else_bb: exit_block,
        });

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
}