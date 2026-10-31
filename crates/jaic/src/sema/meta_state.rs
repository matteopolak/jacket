//! What a metaprogram's view of this compile needs: the workspace it builds, the records already
//! handed to it, and the counts for its performance report.
use crate::sema::body_probe::{Probed, W};
use crate::sema::{code_export, perf};

pub struct MetaState {
    /// Id of the workspace this compiler builds (1: the top-level program).
    pub workspace: i64,
    /// This compiler is the host twin of a cross build: it runs compile-time code on behalf of
    /// the target compiler and neither takes workspace sources nor settles top-level code.
    pub twin: bool,
    /// Syntax trees and types already handed to metaprograms as records.
    pub export: Probed<code_export::ExportState, { W::MetaExport as usize }>,
    /// Counts for the workspace's `PERFORMANCE_REPORT` message (`perf.rs`).
    pub perf: Probed<perf::PerfStats, { W::MetaPerf as usize }>,
}

impl Default for MetaState {
    fn default() -> Self {
        Self {
            workspace: crate::build::TOP_LEVEL_WORKSPACE,
            twin: false,
            export: Probed::default(),
            perf: Probed::default(),
        }
    }
}
