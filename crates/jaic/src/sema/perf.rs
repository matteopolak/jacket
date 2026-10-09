//! What a workspace's `PERFORMANCE_REPORT` message reports, and the text of `DEBUG_DUMP` messages.
//!
//! The compiler counts as it goes: how long each compile-time run took and how often it had to
//! start over, how long lowering procedure bodies took, and how often a call asked for an
//! instance of a polymorphic procedure. `Compiler::export_performance_report` (in
//! `code_export.rs`) turns the counts into the message's records when the workspace finishes.
use super::procs::{BodyState, ProcTarget};
use super::*;
use std::time::Duration;

/// A point in time that reads as zero where the platform has no clock (the browser build).
#[derive(Clone, Copy)]
pub struct Stopwatch(#[cfg(not(target_arch = "wasm32"))] std::time::Instant);

impl Stopwatch {
    pub fn start() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        return Stopwatch(std::time::Instant::now());
        #[cfg(target_arch = "wasm32")]
        return Stopwatch();
    }

    pub fn elapsed(self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        return self.0.elapsed();
        #[cfg(target_arch = "wasm32")]
        return Duration::ZERO;
    }
}

/// One execution of compile-time code (a `#run`, or a constant that needed the interpreter).
#[derive(Clone)]
pub struct RunStat {
    pub span: Span,
    /// The first procedure the code calls directly, which is the one `#run f()` runs.
    pub callee: Option<ir::FuncId>,
    /// How many times the run stopped for something that was not compiled yet and started over.
    pub stalls: u64,
    pub elapsed: Duration,
}

/// The requests for instances of one polymorphic procedure.
#[derive(Clone, Default)]
pub struct PolyStat {
    /// Where the requests came from, once each: a body lowered again after a failed attempt asks
    /// again from the same place.
    sites: HashSet<Span>,
    /// How many places asked.
    pub call_sites: u64,
    /// Requests answered by an instance made earlier for the same constants.
    pub by_constants: u64,
    /// The instances made, with where the request that made each came from.
    pub instances: Vec<(ProcId, Span)>,
}

#[derive(Default)]
pub struct PerfStats {
    pub runs: Vec<RunStat>,
    /// Time the interpreter spent in compile-time code. Runs inside runs count once.
    pub run_time: Duration,
    run_depth: u32,
    /// Time spent lowering procedure bodies to IR, without the compile-time code they ran.
    pub lowering: Duration,
    pub polys: HashMap<ProcId, PolyStat>,
    /// The polymorphic procedures in the order they were first asked for.
    pub poly_order: Vec<ProcId>,
    pub solves_used: u64,
}

impl PerfStats {
    /// The interpreter is about to execute compile-time code.
    pub fn start_run(&mut self) -> Stopwatch {
        self.run_depth += 1;
        Stopwatch::start()
    }

    /// The execution that began at `since` is over.
    pub fn finish_run(&mut self, since: Stopwatch) -> Duration {
        self.run_depth -= 1;
        let elapsed = since.elapsed();
        if self.run_depth == 0 {
            self.run_time += elapsed;
        }
        elapsed
    }

    /// A procedure body at depth 0 starts lowering; returns the mark `finish_lowering` takes.
    pub fn start_lowering(&mut self) -> (Stopwatch, Duration) {
        (Stopwatch::start(), self.run_time)
    }

    /// The outermost body that began at `mark` is lowered: its time counts as lowering, less the
    /// compile-time code it ran, which counts as running.
    pub fn finish_lowering(&mut self, mark: (Stopwatch, Duration)) {
        let ran = self.run_time.saturating_sub(mark.1);
        self.lowering += mark.0.elapsed().saturating_sub(ran);
    }

    /// A use of polymorphic procedure `source` at `span` asked for an instance.
    pub fn poly_request(&mut self, source: ProcId, existing: bool, span: Span) {
        let stat = self.polys.entry(source).or_insert_with(|| {
            self.poly_order.push(source);
            PolyStat::default()
        });
        if !stat.sites.insert(span) {
            return;
        }
        stat.call_sites += 1;
        if existing {
            stat.by_constants += 1;
        }
    }

    /// The request made `instance`.
    pub fn poly_instance(&mut self, source: ProcId, instance: ProcId, span: Span) {
        self.solves_used += 1;
        if let Some(stat) = self.polys.get_mut(&source) {
            stat.instances.push((instance, span));
        }
    }

    /// How many requests there were in all.
    pub fn solves_invoked(&self) -> u64 {
        self.polys.values().map(|s| s.call_sites).sum()
    }
}

impl Compiler {
    /// The `DEBUG_DUMP` texts: one for each lowered procedure declared `#dump`, in the order the
    /// compiler made the procedures. Each is a heading and the listing of the procedure's IR.
    pub fn debug_dumps(&self) -> Vec<String> {
        let mut dumps = Vec::new();
        for info in &self.procs {
            let dump = info
                .lit
                .header
                .flags
                .other
                .iter()
                .any(|flag| flag.name.as_str() == "dump");
            if !dump || info.is_poly || info.is_macro || info.body_state != BodyState::Done {
                continue;
            }
            let Some(ProcTarget::Func(func)) = info.target else {
                continue;
            };
            let Some(Some(body)) = self.program.funcs.get(func.0 as usize) else {
                continue;
            };
            let (line, _) = self
                .sources
                .try_get(info.span.file)
                .map_or((0, 0), |f| f.line_col(info.span.start));
            let path = self
                .sources
                .try_get(info.span.file)
                .map_or("", |f| f.path.as_str());
            dumps.push(format!(
                "Dump of {} ({path}:{line}), jaic IR:\n{}",
                info.name.as_str(),
                body.listing()
            ));
        }
        dumps
    }

    /// The instructions and direct calls in the program lowered so far, for `Bytecode_Report`.
    pub fn instruction_counts(&self) -> (u64, u64) {
        let (mut calls, mut instructions) = (0, 0);
        for func in self.program.funcs.iter().flatten() {
            for block in &func.blocks {
                instructions += block.insts.len() as u64 + 1;
                calls += block
                    .insts
                    .iter()
                    .filter(|i| matches!(i, ir::Inst::Call(_)))
                    .count() as u64;
            }
        }
        (calls, instructions)
    }
}
