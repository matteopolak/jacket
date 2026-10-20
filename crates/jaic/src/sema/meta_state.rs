//! What a metaprogram's view of this compile needs: the workspace it builds, the records already
//! handed to it, and the counts for its performance report.
use crate::sema::{code_export, perf};

pub struct MetaState {
    /// Id of the workspace this compiler builds (1: the top-level program).
    pub workspace: i64,
    /// Syntax trees and types already handed to metaprograms as records.
    pub export: code_export::ExportState,
    /// Counts for the workspace's `PERFORMANCE_REPORT` message (`perf.rs`).
    pub perf: perf::PerfStats,
}

impl Default for MetaState {
    fn default() -> Self {
        Self {
            workspace: crate::build::TOP_LEVEL_WORKSPACE,
            export: code_export::ExportState::default(),
            perf: perf::PerfStats::default(),
        }
    }
}
