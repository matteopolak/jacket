//! Line coverage for programs run by `-interp -coverage` (docs/tools/coverage.md), in the
//! format native `-coverage` programs write, so `jacket test --coverage` reads both.
//!
//! The interpreter notes each basic block it enters. A block's lines are those of the `Loc`
//! markers in it; a procedure's blocks are registered, with zero runs, the first time it is
//! entered, so a line of an entered procedure that never ran is known. Each interpreter (the main
//! compile and every metaprogram workspace) counts for itself and merges into one process-wide
//! table when it is flushed. The table is written at exit as
//!
//! ```text
//! jaic-coverage 1
//! interp
//! file /abs/path/a.jai
//! 12 3
//! 13 0
//! ```
//!
//! (a `file` line, then a `line runs` line per line of that file), to `$JAIC_COVERAGE_FILE`, or
//! to `$JAIC_COVERAGE_DIR/<program>.<pid>.cov` (default folder: the working directory), like a
//! native program.
use crate::fxhash::HashMap;
use crate::ir::{self, Inst};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};

/// Whether the lines the program runs are being recorded (`-interp -coverage`).
pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let on = crate::knobs::get().coverage;
        if on {
            // The program may change the working directory before it ends.
            let _ = DESTINATION.set(destination());
        }
        // A program that calls `exit` under `jaic -interp` ends the process from inside the
        // interpreter, so the lines it ran are also written from the C runtime's exit hook.
        #[cfg(not(target_arch = "wasm32"))]
        if on {
            unsafe extern "C" {
                fn atexit(hook: extern "C" fn()) -> i32;
            }
            extern "C" fn at_exit() {
                write();
            }
            // SAFETY: `at_exit` is an `extern "C"` function with no captured state, valid for the
            // life of the process.
            unsafe { atexit(at_exit) };
        }
        on
    })
}

static DESTINATION: OnceLock<std::path::PathBuf> = OnceLock::new();

/// How often each line of each file ran, merged from every interpreter of the process.
static TOTALS: LazyLock<Mutex<BTreeMap<String, BTreeMap<u32, u64>>>> =
    LazyLock::new(Mutex::default);

fn totals() -> MutexGuard<'static, BTreeMap<String, BTreeMap<u32, u64>>> {
    TOTALS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// One interpreter's counts.
#[derive(Default)]
pub(crate) struct Lines {
    /// For each procedure seen (by the address of its IR), the lines of each of its blocks as
    /// indexes into `lines`.
    procs: HashMap<usize, Vec<Vec<u32>>>,
    /// The `(file, line)` of each index.
    sites: Vec<(u32, u32)>,
    index: HashMap<(u32, u32), u32>,
    runs: Vec<u64>,
    /// The path of each file id, as the program names it.
    paths: HashMap<u32, String>,
}

impl Lines {
    /// A procedure is about to run: registers its lines the first time.
    pub(crate) fn enter(&mut self, program: &ir::Program, func: &ir::Func) {
        let key = std::ptr::from_ref(func) as usize;
        if self.procs.contains_key(&key) {
            return;
        }
        let mut blocks = Vec::with_capacity(func.blocks.len());
        for (i, block) in func.blocks.iter().enumerate() {
            let mut ids = Vec::new();
            if i == 0
                && let Some((file, line)) = declaration(func)
            {
                ids.push(self.line(program, file, line));
            }
            for inst in &block.insts {
                if let Inst::Loc {
                    file,
                    line,
                    ..
                } = *inst
                {
                    ids.push(self.line(program, file, line));
                }
            }
            ids.sort_unstable();
            ids.dedup();
            blocks.push(ids);
        }
        self.procs.insert(key, blocks);
    }

    fn line(&mut self, program: &ir::Program, file: u32, line: u32) -> u32 {
        if let Some(&id) = self.index.get(&(file, line)) {
            return id;
        }
        self.paths.entry(file).or_insert_with(|| {
            let path = program
                .file_paths
                .get(file as usize)
                .map_or("", String::as_str);
            std::path::absolute(path)
                .map_or_else(|_| path.to_string(), |p| p.to_string_lossy().into_owned())
        });
        let id = self.sites.len() as u32;
        self.sites.push((file, line));
        self.runs.push(0);
        self.index.insert((file, line), id);
        id
    }

    /// Block `block` of `func` is entered.
    #[inline]
    pub(crate) fn hit(&mut self, func: &ir::Func, block: usize) {
        let key = std::ptr::from_ref(func) as usize;
        if let Some(ids) = self.procs.get(&key).and_then(|blocks| blocks.get(block)) {
            for &id in ids {
                self.runs[id as usize] += 1;
            }
        }
    }

    /// Adds the counts to the process-wide table and starts over.
    pub(crate) fn flush(&mut self) {
        let mut totals = totals();
        for (&(file, line), runs) in self.sites.iter().zip(self.runs.iter_mut()) {
            let path = self.paths.get(&file).map_or("", String::as_str);
            if path.is_empty() || line == 0 {
                continue;
            }
            *totals
                .entry(path.to_string())
                .or_default()
                .entry(line)
                .or_default() += std::mem::take(runs);
        }
    }
}

/// The file and line a procedure is declared at, when it says.
fn declaration(func: &ir::Func) -> Option<(u32, u32)> {
    match (&func.trace, &func.debug) {
        (Some(t), _) => Some((t.file, t.line)),
        (None, Some(d)) => Some((d.file, d.line)),
        (None, None) => None,
    }
}

/// The report text for a table.
fn render(table: &BTreeMap<String, BTreeMap<u32, u64>>) -> String {
    let mut text = String::from("jaic-coverage 1\ninterp\n");
    for (path, lines) in table {
        let _ = writeln!(text, "file {path}");
        for (line, runs) in lines {
            let _ = writeln!(text, "{line} {runs}");
        }
    }
    text
}

/// Where the report goes: `$JAIC_COVERAGE_FILE`, or a file named for the program in
/// `$JAIC_COVERAGE_DIR` (default: the working directory).
fn destination() -> std::path::PathBuf {
    if let Some(file) = jacket_config::env::COVERAGE_FILE
        .os()
        .filter(|f| !f.is_empty())
    {
        return file.into();
    }
    let dir = jacket_config::env::COVERAGE_DIR
        .os()
        .filter(|d| !d.is_empty())
        .map_or_else(
            || std::path::PathBuf::from("."),
            |d| std::path::absolute(&d).unwrap_or_else(|_| d.into()),
        );
    let name = std::env::args_os()
        .nth(1)
        .and_then(|p| {
            std::path::Path::new(&p)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "interp".into());
    dir.join(format!("{name}.{}.cov", std::process::id()))
}

/// Writes every line counted so far (nothing when coverage is off). Called when the program
/// ends and from the exit hook; the second write replaces the first with the same table.
pub fn write() {
    if !enabled() {
        return;
    }
    let text = render(&totals());
    let Some(path) = DESTINATION.get() else {
        return;
    };
    if let Err(e) = std::fs::write(path, text) {
        eprintln!(
            "warning: could not write the coverage file {}: {e}",
            path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_lists_files_and_how_often_each_line_ran() {
        let mut table = BTreeMap::new();
        table.insert("/p/a.jai".to_string(), BTreeMap::from([(3, 2), (4, 0)]));
        table.insert("/p/b.jai".to_string(), BTreeMap::from([(1, 7)]));
        assert_eq!(
            render(&table),
            "jaic-coverage 1\ninterp\nfile /p/a.jai\n3 2\n4 0\nfile /p/b.jai\n1 7\n"
        );
    }
}
