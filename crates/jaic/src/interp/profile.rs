//! Opt-in execution counts per procedure (`-profile`), to find slow compile-time code.
//!
//! Every interpreter (the main compile and each metaprogram workspace) counts calls, basic blocks
//! and instructions executed in each procedure's own frames (callees not included), and merges them
//! into a process-wide table when it is dropped or flushed. `report` formats the table.
//!
//! `-coverage_file <file>` records which procedures ran at all: `write_coverage` appends one
//! `path:line name` line per procedure (polymorph instances share their declaration's line).
//! `tools/stdlib/stdlib_coverage.py` reads it.
use crate::fxhash::HashMap;
use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| crate::knobs::get().profile != crate::knobs::Profile::Off)
}

/// Whether the counts are of the unoptimized control flow (the default): `-profile_ops` runs
/// the straightened code instead, so the op counts show what really runs, but blocks and IR
/// instructions are then counted only where a block is entered.
pub(crate) fn exact() -> bool {
    static EXACT: OnceLock<bool> = OnceLock::new();
    *EXACT.get_or_init(|| crate::knobs::get().profile != crate::knobs::Profile::Ops)
}

/// The `-coverage_file`, when coverage is being recorded.
pub(crate) fn coverage_file() -> Option<&'static std::path::Path> {
    static FILE: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
    FILE.get_or_init(|| {
        let file = crate::knobs::get().coverage_file.clone();
        // A program that calls `exit` under `jaic -interp` ends the process from inside the
        // interpreter, so the procedures it ran are also written from the C runtime's exit hook.
        #[cfg(not(target_arch = "wasm32"))]
        if file.is_some() {
            unsafe extern "C" {
                fn atexit(hook: extern "C" fn()) -> i32;
            }
            extern "C" fn at_exit() {
                write_coverage();
            }
            // SAFETY: `at_exit` is an `extern "C"` function with no captured state, valid for the
            // life of the process.
            unsafe { atexit(at_exit) };
        }
        file
    })
    .as_deref()
}

/// Every procedure any interpreter of this process ran, as `path:line name`.
static COVERED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// Record that the procedure declared at `path:line` ran.
pub(crate) fn cover(path: &str, line: u32, name: &str) {
    let mut covered = COVERED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    covered.insert(format!("{path}:{line} {name}"));
}

/// Append the procedures that ran since the last call to the `-coverage_file` (nothing when
/// it is unset).
pub fn write_coverage() {
    use std::io::Write;
    let Some(path) = coverage_file() else {
        return;
    };
    let covered = std::mem::take(
        &mut *COVERED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    if covered.is_empty() {
        return;
    }
    let text: String = covered.iter().map(|line| format!("{line}\n")).collect();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path);
    if let Err(e) = file.and_then(|mut f| f.write_all(text.as_bytes())) {
        eprintln!(
            "warning: could not write the coverage file {}: {e}",
            path.display()
        );
    }
}

#[derive(Clone, Copy, Default)]
struct Row {
    calls: u64,
    blocks: u64,
    insts: u64,
}

static TOTALS: Mutex<Option<HashMap<String, Row>>> = Mutex::new(None);

/// IR instructions executed by kind (`IConst`, `Load`, ...).
static OPS: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

/// The interpreter's own ops executed by kind (`code.rs`: folded loads, immediates...).
static CODE_OPS: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

/// Adjacent op pairs executed within a block, by kind (`code.rs` ops), to find what to fuse.
static CODE_PAIRS: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

/// Counts of one interpreter, indexed by `FuncId`.
#[derive(Default)]
pub(crate) struct Counts {
    rows: Vec<(Row, Option<String>)>,
    ir: Kinds,
    ops: Kinds,
    pairs: HashMap<String, u64>,
}

/// Executions by kind name, with each block's kinds cached by the block's address.
#[derive(Default)]
struct Kinds {
    blocks: HashMap<(usize, usize), Vec<usize>>,
    names: Vec<(String, u64)>,
}

impl Kinds {
    fn count<T: std::fmt::Debug>(&mut self, items: &[T]) {
        let key = (items.as_ptr() as usize, items.len());
        if !self.blocks.contains_key(&key) {
            let mut list = Vec::with_capacity(items.len());
            for item in items {
                let text = format!("{item:?}");
                let name = text
                    .split(|c: char| !c.is_alphanumeric())
                    .next()
                    .unwrap_or("");
                let index = if let Some(i) = self.names.iter().position(|(n, _)| n == name) {
                    i
                } else {
                    self.names.push((name.to_string(), 0));
                    self.names.len() - 1
                };
                list.push(index);
            }
            self.blocks.insert(key, list);
        }
        for &k in &self.blocks[&key] {
            self.names[k].1 += 1;
        }
    }

    fn flush(&mut self, into: &Mutex<Option<HashMap<String, u64>>>) {
        let mut table = into
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let table = table.get_or_insert_with(HashMap::default);
        for (name, n) in &mut self.names {
            *table.entry(name.clone()).or_default() += std::mem::take(n);
        }
    }
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

    /// Count the instructions of a block about to run, by kind: its IR and the ops it
    /// became.
    pub(crate) fn block<T: std::fmt::Debug>(&mut self, block: &crate::ir::Block, ops: &[T]) {
        self.ir.count(&block.insts);
        self.ops.count(ops);
        let key = (ops.as_ptr() as usize, ops.len());
        let kinds = self.ops.blocks.get(&key).cloned().unwrap_or_default();
        for w in kinds.windows(2) {
            let name = format!("{} {}", self.ops.names[w[0]].0, self.ops.names[w[1]].0);
            *self.pairs.entry(name).or_default() += 1;
        }
    }

    pub(crate) fn flush(&mut self) {
        self.ir.flush(&OPS);
        self.ops.flush(&CODE_OPS);
        {
            let mut table = CODE_PAIRS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let table = table.get_or_insert_with(HashMap::default);
            for (name, n) in self.pairs.drain() {
                *table.entry(name).or_default() += n;
            }
        }
        let mut totals = TOTALS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let totals = totals.get_or_insert_with(HashMap::default);
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
    let totals = TOTALS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
    out.push_str(&kind_line("instructions by kind:", &OPS, all));
    let ran: u64 = {
        let table = CODE_OPS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        table.iter().flat_map(|t| t.values()).sum()
    };
    out.push_str(&kind_line(
        &format!("{ran} interpreter ops by kind:"),
        &CODE_OPS,
        ran,
    ));
    {
        let table = CODE_PAIRS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut pairs: Vec<(&String, &u64)> = table.iter().flat_map(|t| t.iter()).collect();
        pairs.sort_by_key(|a| std::cmp::Reverse(*a.1));
        out.push_str("op pairs within blocks:");
        for (name, n) in pairs.into_iter().take(24) {
            out.push_str(&format!(" [{name}] {n}"));
        }
        out.push('\n');
    }
    Some(out)
}

/// One line of kind shares, as percentages of `all`.
fn kind_line(title: &str, table: &Mutex<Option<HashMap<String, u64>>>, all: u64) -> String {
    let table = table
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut kinds: Vec<(&String, &u64)> = table.iter().flat_map(|t| t.iter()).collect();
    kinds.sort_by_key(|a| std::cmp::Reverse(*a.1));
    let mut out = title.to_string();
    for (name, n) in kinds.into_iter().take(40) {
        out.push_str(&format!(
            " {name} {:.1}%",
            100.0 * *n as f64 / all.max(1) as f64
        ));
    }
    out.push('\n');
    out
}
