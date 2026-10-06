//! Opt-in execution counts per procedure (`JAIC_PROFILE=1`), to find slow compile-time code.
//!
//! Every interpreter (the main compile and each metaprogram workspace) counts calls, basic blocks
//! and instructions executed in each procedure's own frames (callees not included), and merges them
//! into a process-wide table when it is dropped or flushed. `report` formats the table.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("JAIC_PROFILE").is_some_and(|v| v != "0"))
}

#[derive(Clone, Copy, Default)]
struct Row {
    calls: u64,
    blocks: u64,
    insts: u64,
}

static TOTALS: Mutex<Option<HashMap<String, Row>>> = Mutex::new(None);

/// Instructions executed by kind (`IConst`, `Load`, ...).
static OPS: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

/// Counts of one interpreter, indexed by `FuncId`.
#[derive(Default)]
pub(crate) struct Counts {
    rows: Vec<(Row, Option<String>)>,
    /// Instruction kinds of each block seen, by the block's address: indexes into `kinds`.
    block_kinds: HashMap<usize, Vec<usize>>,
    kinds: Vec<(String, u64)>,
}

impl Counts {
    pub(crate) fn add(&mut self, func: usize, name: &str, blocks: u64, insts: u64) {
        if self.rows.len() <= func {
            self.rows.resize_with(func + 1, Default::default);
        }
        let (row, known) = &mut self.rows[func];
        known.get_or_insert_with(|| name.to_string());
        row.calls += 1;
        row.blocks += blocks;
        row.insts += insts;
    }

    /// Count the instructions of a block about to run, by kind.
    pub(crate) fn block(&mut self, block: &crate::ir::Block) {
        let key = block as *const _ as usize;
        if !self.block_kinds.contains_key(&key) {
            let mut list = Vec::with_capacity(block.insts.len());
            for inst in &block.insts {
                let text = format!("{inst:?}");
                let name = text
                    .split(|c: char| !c.is_alphanumeric())
                    .next()
                    .unwrap_or("");
                let index = match self.kinds.iter().position(|(n, _)| n == name) {
                    Some(i) => i,
                    None => {
                        self.kinds.push((name.to_string(), 0));
                        self.kinds.len() - 1
                    }
                };
                list.push(index);
            }
            self.block_kinds.insert(key, list);
        }
        for &k in &self.block_kinds[&key] {
            self.kinds[k].1 += 1;
        }
    }

    pub(crate) fn flush(&mut self) {
        {
            let mut ops = OPS.lock().unwrap_or_else(|e| e.into_inner());
            let ops = ops.get_or_insert_with(HashMap::new);
            for (name, n) in &mut self.kinds {
                *ops.entry(name.clone()).or_default() += std::mem::take(n);
            }
        }
        let mut totals = TOTALS.lock().unwrap_or_else(|e| e.into_inner());
        let totals = totals.get_or_insert_with(HashMap::new);
        for (row, name) in self.rows.drain(..) {
            let Some(name) = name else {
                continue;
            };
            let total = totals.entry(name).or_default();
            total.calls += row.calls;
            total.blocks += row.blocks;
            total.insts += row.insts;
        }
    }
}

/// The `top` procedures by instructions executed, or `None` when profiling is off.
pub fn report(top: usize) -> Option<String> {
    if !enabled() {
        return None;
    }
    let totals = TOTALS.lock().unwrap_or_else(|e| e.into_inner());
    let mut rows: Vec<(&String, &Row)> = totals.iter().flat_map(|t| t.iter()).collect();
    rows.sort_by_key(|a| std::cmp::Reverse(a.1.insts));
    let all: u64 = rows.iter().map(|(_, r)| r.insts).sum();
    let mut out = format!(
        "interpreter profile: {all} instructions\n{:>6} {:>14} {:>12} {:>10}  procedure\n",
        "%", "instructions", "blocks", "calls"
    );
    for (name, r) in rows.into_iter().take(top) {
        out.push_str(&format!(
            "{:>5.1}% {:>14} {:>12} {:>10}  {name}\n",
            100.0 * r.insts as f64 / all.max(1) as f64,
            r.insts,
            r.blocks,
            r.calls
        ));
    }
    let ops = OPS.lock().unwrap_or_else(|e| e.into_inner());
    let mut ops: Vec<(&String, &u64)> = ops.iter().flat_map(|t| t.iter()).collect();
    ops.sort_by_key(|a| std::cmp::Reverse(*a.1));
    out.push_str("instructions by kind:");
    for (name, n) in ops.into_iter().take(16) {
        out.push_str(&format!(
            " {name} {:.1}%",
            100.0 * *n as f64 / all.max(1) as f64
        ));
    }
    out.push('\n');
    Some(out)
}
