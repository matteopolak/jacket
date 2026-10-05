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

/// Counts of one interpreter, indexed by `FuncId`.
#[derive(Default)]
pub(crate) struct Counts {
    rows: Vec<(Row, Option<String>)>,
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

    pub(crate) fn flush(&mut self) {
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
    Some(out)
}
