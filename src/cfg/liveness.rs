use std::collections::{HashSet, VecDeque};

use crate::cfg::basic_blocks::{BlockId, Cfg, SymbolId, Terminator};

#[derive(Debug, Clone)]
pub struct LivenessResult {
    pub reachable: HashSet<BlockId>,
    pub block_use: Vec<HashSet<SymbolId>>,
    pub block_def: Vec<HashSet<SymbolId>>,
    pub live_in: Vec<HashSet<SymbolId>>,
    pub live_out: Vec<HashSet<SymbolId>>,
}

pub fn compute_liveness(cfg: &Cfg) -> LivenessResult {
    let reachable = compute_reachable_blocks(cfg);
    let (block_use, block_def) = compute_block_use_def(cfg, &reachable);

    let mut live_in = vec![HashSet::new(); cfg.blocks.len()];
    let mut live_out = vec![HashSet::new(); cfg.blocks.len()];

    let mut changed = true;

    while changed {
        changed = false;

        for block in cfg.blocks.iter().rev() {
            let block_id = block.id;

            if !reachable.contains(&block_id) {
                continue;
            }

            let mut new_live_out = HashSet::new();
            for succ in block.successors() {
                if reachable.contains(&succ) {
                    new_live_out.extend(live_in[succ].iter().copied());
                }
            }

            let mut new_live_in = block_use[block_id].clone();
            for sym in &new_live_out {
                if !block_def[block_id].contains(sym) {
                    new_live_in.insert(*sym);
                }
            }

            if new_live_out != live_out[block_id] || new_live_in != live_in[block_id] {
                live_out[block_id] = new_live_out;
                live_in[block_id] = new_live_in;
                changed = true;
            }
        }
    }

    LivenessResult {
        reachable,
        block_use,
        block_def,
        live_in,
        live_out,
    }
}

pub fn compute_reachable_blocks(cfg: &Cfg) -> HashSet<BlockId> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(cfg.entry);

    while let Some(block_id) = queue.pop_front() {
        if !reachable.insert(block_id) {
            continue;
        }

        for succ in cfg.blocks[block_id].successors() {
            queue.push_back(succ);
        }
    }

    reachable
}

pub fn compute_block_use_def(
    cfg: &Cfg,
    reachable: &HashSet<BlockId>,
) -> (Vec<HashSet<SymbolId>>, Vec<HashSet<SymbolId>>) {
    let mut block_use = vec![HashSet::new(); cfg.blocks.len()];
    let mut block_def = vec![HashSet::new(); cfg.blocks.len()];

    for block in &cfg.blocks {
        let block_id = block.id;

        if !reachable.contains(&block_id) {
            continue;
        }

        let mut uses = HashSet::new();
        let mut defs = HashSet::new();

        for stmt in &block.statements {
            for used in &stmt.uses {
                if !defs.contains(used) {
                    uses.insert(*used);
                }
            }

            for defined in &stmt.defines {
                defs.insert(*defined);
            }
        }

        match &block.terminator {
            Terminator::Branch { cond_uses, .. } => {
                for used in cond_uses {
                    if !defs.contains(used) {
                        uses.insert(*used);
                    }
                }
            }
            Terminator::Return { value_uses } => {
                for used in value_uses {
                    if !defs.contains(used) {
                        uses.insert(*used);
                    }
                }
            }
            Terminator::Goto(_) | Terminator::Unset | Terminator::Exit => {}
        }

        block_use[block_id] = uses;
        block_def[block_id] = defs;
    }

    (block_use, block_def)
}

pub fn sorted_symbols(set: &HashSet<SymbolId>) -> Vec<SymbolId> {
    let mut v: Vec<_> = set.iter().copied().collect();
    v.sort_unstable();
    v
}

pub fn print_liveness(cfg: &Cfg, result: &LivenessResult) {
    for block in &cfg.blocks {
        let id = block.id;

        println!("block {}", id);
        println!("  reachable: {}", result.reachable.contains(&id));
        println!("  use: {:?}", sorted_symbols(&result.block_use[id]));
        println!("  def: {:?}", sorted_symbols(&result.block_def[id]));
        println!("  live_in: {:?}", sorted_symbols(&result.live_in[id]));
        println!("  live_out: {:?}", sorted_symbols(&result.live_out[id]));
    }
}