//! What compile-time code costs: a code lens over each evaluation (`#run`, a constant that needed
//! the interpreter...) that ran at least `compileTime.costLensBlocks` basic blocks, and the command
//! `jai.showCompileTimeCosts` that lists every evaluation of the program, costliest first.
//!
//! The costs are the interpreter's own counts, recorded by the compile that checked the program
//! (`jaic::sema::perf::RunStat`), so asking for them costs nothing beyond that compile.
use crate::semantic::{EvalCost, LimitKind};
use crate::session::Session;
use crate::{CodeLens, Command, DocumentUri, Error, Location};
use serde_json::{Value, json};
use std::time::Duration;

pub(crate) const COMMAND: &str = "jai.showCompileTimeCosts";

/// What the server sends when a limit stopped compile-time code.
pub(crate) const NOTIFICATION: &str = "jai/compileTimeLimit";

/// `1.2M blocks · 3 ms`.
pub fn cost_text(blocks: u64, elapsed: Duration) -> String {
    let count = match blocks {
        0..1_000 => format!("{blocks}"),
        1_000..1_000_000 => format!("{:.1}k", blocks as f64 / 1e3),
        1_000_000..1_000_000_000 => format!("{:.1}M", blocks as f64 / 1e6),
        _ => format!("{:.1}G", blocks as f64 / 1e9),
    };
    let time = match elapsed.as_secs_f64() {
        s if s >= 1.0 => format!("{s:.1} s"),
        s if s >= 0.001 => format!("{:.0} ms", s * 1e3),
        _ => "<1 ms".to_string(),
    };
    format!("{count} blocks · {time}")
}

/// One evaluation of the program, for the command's answer.
pub struct CostRow {
    pub location: Location,
    pub blocks: u64,
    pub elapsed: Duration,
}

impl CostRow {
    /// The row as the command answers it.
    pub fn wire(&self) -> Value {
        json!({
            "uri": self.location.uri,
            "range": self.location.range,
            "blocks": self.blocks,
            "millis": self.elapsed.as_secs_f64() * 1e3,
            "title": cost_text(self.blocks, self.elapsed),
        })
    }
}

impl Session {
    /// A lens over each costly evaluation written in the document.
    pub(crate) fn cost_code_lenses(&self, uri: &DocumentUri) -> Result<Vec<CodeLens>, Error> {
        self.document(uri)?;
        let found = self.checked(uri, |a, file| {
            let path = a.compiler.src.sources.get(file).path.to_string();
            let hint = self.check_limits.hint_blocks;
            let own: Vec<EvalCost> = a
                .evaluation_costs()
                .into_iter()
                .filter(|c| c.path == path && c.blocks >= hint)
                .collect();
            own
        });
        let mut lenses = Vec::new();
        for cost in found.unwrap_or_default() {
            let Some(range) =
                self.range_of(uri, crate::analysis::Span::new(cost.start, cost.start))
            else {
                continue;
            };
            lenses.push(CodeLens {
                range,
                command: Command::at(
                    cost_text(cost.blocks, cost.elapsed),
                    COMMAND,
                    uri.as_str(),
                    range.start,
                ),
            });
        }
        Ok(lenses)
    }

    /// Every compile-time evaluation of the program the document is in, costliest first
    /// (`jai.showCompileTimeCosts`).
    pub fn compile_time_costs(&self, uri: &DocumentUri) -> Vec<CostRow> {
        let costs = self
            .checked(uri, |a, _| a.evaluation_costs())
            .unwrap_or_default();
        costs
            .into_iter()
            .filter_map(|c| {
                Some(CostRow {
                    location: self.location_in(&c.path, &c.text, c.start, c.end)?,
                    blocks: c.blocks,
                    elapsed: c.elapsed,
                })
            })
            .collect()
    }
}

impl Session {
    /// The limits the compiles since the last call ran into, as `jai/compileTimeLimit`
    /// notifications for a client that asked for them (`compileTime.limitNotices`).
    pub(crate) fn take_limit_notices(&self) -> Vec<Value> {
        let hits = self.semantic.borrow_mut().take_limit_hits();
        if !self.limit_notices {
            return Vec::new();
        }
        hits.into_iter()
            .filter_map(|hit| {
                let location = self.location_in(&hit.path, &hit.text, hit.start, hit.end)?;
                Some(json!({
                    "jsonrpc": "2.0",
                    "method": NOTIFICATION,
                    "params": {
                        "kind": match hit.kind {
                            LimitKind::Budget => "budget",
                            LimitKind::Time => "time",
                        },
                        "uri": location.uri,
                        "range": location.range,
                        "message": hit.message,
                    },
                }))
            })
            .collect()
    }
}
