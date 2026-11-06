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
        let _query = self.discovery_query();
        let fs = self.config_filesystem();
        let (uri, _) = match &fs {
            Some(fs) => self.projected_document_entry(fs, uri)?,
            None => self.document_entry(uri)?,
        };
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
        let _query = self.discovery_query();
        if !self.begin_cost_inputs(uri) {
            return Vec::new();
        }
        let Some(fs) = self.config_filesystem() else {
            return Vec::new();
        };
        let Ok((uri, _)) = self.projected_document_entry(&fs, uri) else {
            return Vec::new();
        };
        if !self.begin_cost_inputs(uri) {
            return Vec::new();
        }
        let Some(costs) = self.checked(uri, |analysis, _| {
            analysis
                .evaluation_costs()
                .into_iter()
                .filter_map(|cost| {
                    Some(CostRow {
                        location: self.projected_location(
                            &fs, &cost.path, &cost.text, cost.start, cost.end,
                        )?,
                        blocks: cost.blocks,
                        elapsed: cost.elapsed,
                    })
                })
                .collect()
        }) else {
            return Vec::new();
        };
        if let Some(doc) = self.documents.get(uri) {
            doc.initial_costs_accepted.set(true);
        }
        costs
    }

    /// Admit additional program locations before the first cost-command projection.
    fn begin_cost_inputs(&self, uri: &DocumentUri) -> bool {
        self.documents
            .get(uri)
            .is_none_or(|doc| doc.initial_costs_accepted.get() || self.begin_cold_input_capture())
    }
}

impl Session {
    /// Queued historical events, accounted queue/string bytes and pressure retirements.
    /// Example: inspect `session.limit_notice_counts()` independently of compiler retention.
    pub fn limit_notice_counts(&self) -> (usize, usize, usize) {
        self.semantic.borrow().limit_notice_counts()
    }

    /// The limits the compiles since the last call ran into, as `jai/compileTimeLimit`
    /// notifications for a client that asked for them (`compileTime.limitNotices`).
    pub(crate) fn take_limit_notices(&self) -> Vec<Value> {
        let hits = self.semantic.borrow_mut().take_limit_hits();
        if !self.limit_notices {
            return Vec::new();
        }
        hits.into_iter()
            .map(|hit| {
                json!({
                    "jsonrpc": "2.0",
                    "method": NOTIFICATION,
                    "params": {
                        "kind": match hit.kind {
                            LimitKind::Budget => "budget",
                            LimitKind::Time => "time",
                        },
                        "uri": hit.location.uri,
                        "range": hit.location.range,
                        "message": hit.message,
                    },
                })
            })
            .collect()
    }
}


#[cfg(test)]
mod notice_tests {
    use super::*;
    use jaic::sema::{FileSystem, NativeFs};
    use std::rc::Rc;
    #[cfg(unix)]
    use std::{
        cell::Cell,
        path::{Path, PathBuf},
    };

    #[cfg(unix)]
    struct CountFs(Cell<usize>);

    #[cfg(unix)]
    impl CountFs {
        fn bump(&self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[cfg(unix)]
    impl FileSystem for CountFs {
        fn is_native(&self) -> bool {
            true
        }

        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            self.bump();
            NativeFs.read(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.bump();
            NativeFs.is_file(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.bump();
            NativeFs.is_dir(path)
        }

        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            self.bump();
            NativeFs.list_dir(path)
        }

        fn stamp(&self, path: &Path) -> Option<u64> {
            self.bump();
            NativeFs.stamp(path)
        }

        fn canonical(&self, path: &Path) -> PathBuf {
            self.bump();
            NativeFs.canonical(path)
        }

        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            self.bump();
            NativeFs.canonical_exact(path)
        }
    }

    fn session(fs: Rc<dyn FileSystem>) -> Session {
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                fs,
                services: None,
                options: Box::new(|_, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options
                }),
            },
        );
        session.set_evaluation_budget(16);
        session.set_check_time_limit(0.0);
        session.set_limit_notices(true);
        session
    }

    #[cfg(unix)]
    #[test]
    fn historical_limit_notice_freezes_original_utf16_frame_without_drain_io() {
        let dir = test_support::TempDir::new("historical-limit-frame");
        let physical = dir.path().join("physical");
        let alias = dir.path().join("alias");
        std::fs::create_dir(&physical).unwrap();
        std::os::unix::fs::symlink(&physical, &alias).unwrap();
        let source = "spin :: () { while true {} }\n/* é🚀 */ #run spin();\nmain :: () {}\n";
        std::fs::write(physical.join("main.jai"), source).unwrap();
        let fs = Rc::new(CountFs(Cell::new(0)));
        let mut session = session(fs.clone());
        let uri = DocumentUri::from_path(&alias.join("main.jai")).unwrap();
        session.open(uri, 1, source.into()).unwrap();
        let diagnostics = session
            .diagnostics(&DocumentUri::from_path(&alias.join("main.jai")).unwrap())
            .unwrap();
        let budget = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("execution budget exhausted"))
            .expect("budget diagnostic");
        fs.0.set(0);
        let notices = session.take_limit_notices();
        assert_eq!(notices.len(), 1);
        assert_eq!(
            fs.0.get(),
            0,
            "historical frames never reconsult native aliases at drain"
        );
        assert_eq!(
            notices[0]["params"]["uri"],
            DocumentUri::from_path(&physical.join("main.jai"))
                .unwrap()
                .as_str()
        );
        assert_eq!(
            notices[0]["params"]["range"],
            serde_json::to_value(budget.range).unwrap()
        );
    }

    #[test]
    fn pending_limit_notices_do_not_retain_whole_source_versions() {
        let dir = test_support::TempDir::new("limit-notice-memory");
        let path = dir.path().join("main.jai");
        let body = "spin :: () { while true {} }\n#run spin();\nmain :: () {}\n";
        std::fs::write(&path, body).unwrap();
        let uri = DocumentUri::from_path(&path).unwrap();
        let mut session = session(Rc::new(NativeFs));
        let padding = "x".repeat(1024 * 1024);
        for version in 1..=17 {
            let source = format!("{body}// {version} {padding}\n");
            if version == 1 {
                session.open(uri.clone(), version, source).unwrap();
            } else {
                session
                    .change(
                        &uri,
                        version,
                        &[crate::TextChange {
                            range: None,
                            range_length: None,
                            text: source,
                        }],
                    )
                    .unwrap();
            }
            assert!(
                session
                    .diagnostics(&uri)
                    .unwrap()
                    .iter()
                    .any(|diagnostic| diagnostic.message.contains("execution budget exhausted"))
            );
        }
        let (count, bytes, _) = session.semantic.borrow().limit_notice_counts();
        assert!(count > 0);
        println!("historical_events={count} accounted_queue_bytes={bytes}");
        assert!(
            bytes < 64 * 1024,
            "tiny historical frames must not retain source-sized allocations"
        );
        assert!(
            bytes <= 16 * 1024 * 1024,
            "pending notices retained {bytes} bytes for {count} tiny events"
        );
    }

    #[test]
    fn configured_compile_budget_survives_trust_transitions() {
        let dir = test_support::TempDir::new("notice-trust-budget");
        let path = dir.path().join("main.jai");
        let source = "count :: (n: int) -> int { total := 0; for 0..n total += it; return total; }\nvalue :: #run count(100);\nmain :: () {}\n";
        std::fs::write(&path, source).unwrap();
        let uri = DocumentUri::from_path(&path).unwrap();
        let mut session = session(Rc::new(NativeFs));
        session.set_trusted(false);
        session.set_trusted(true);
        session.open(uri.clone(), 1, source.into()).unwrap();
        let diagnostics = session.diagnostics(&uri).unwrap();
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("execution budget exhausted (16 steps)")),
            "trust reset must preserve configured budget: {diagnostics:?}"
        );
    }

    #[test]
    fn unaccepted_interrupted_build_does_not_queue_a_historical_event() {
        let dir = test_support::TempDir::new("cancelled-limit-notice");
        let path = dir.path().join("main.jai");
        let source = "spin :: () { while true {} }\n#run spin();\nmain :: () {}\n";
        std::fs::write(&path, source).unwrap();
        let uri = DocumentUri::from_path(&path).unwrap();
        let mut session = session(Rc::new(NativeFs));
        session.open(uri.clone(), 1, source.into()).unwrap();
        jaic::interrupt::stop_after(1);
        let publication = session.publications_unless_interrupted();
        jaic::interrupt::stop_after(0);
        jaic::interrupt::request(false);
        assert!(publication.is_none());
        assert_eq!(session.limit_notice_counts(), (0, 0, 0));
        assert!(
            session
                .diagnostics(&uri)
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic.message.contains("execution budget exhausted"))
        );
        assert_eq!(session.limit_notice_counts().0, 1);
    }

    #[test]
    fn historical_event_is_not_rewritten_by_later_source_or_budget_settings() {
        let dir = test_support::TempDir::new("limit-notice-old-settings");
        let path = dir.path().join("main.jai");
        let source = "spin :: () { while true {} }\n#run spin();\nmain :: () {}\n";
        std::fs::write(&path, source).unwrap();
        let uri = DocumentUri::from_path(&path).unwrap();
        let mut session = session(Rc::new(NativeFs));
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.diagnostics(&uri).unwrap();
        session.set_evaluation_budget(32);
        session
            .change(
                &uri,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: "main :: () {}".into(),
                }],
            )
            .unwrap();
        assert!(session.diagnostics(&uri).unwrap().is_empty());
        let notices = session.take_limit_notices();
        assert_eq!(notices.len(), 1);
        assert!(
            notices[0]["params"]["message"]
                .as_str()
                .unwrap()
                .contains("(16 steps)")
        );
        assert_eq!(notices[0]["params"]["range"]["start"]["line"], 1);
        assert_eq!(session.limit_notice_counts(), (0, 0, 0));
    }

    #[test]
    fn disabled_subscription_never_collects_or_replays_earlier_events() {
        let dir = test_support::TempDir::new("limit-notice-disabled");
        let path = dir.path().join("main.jai");
        let source = "spin :: () { while true {} }\n#run spin();\nmain :: () {}\n";
        std::fs::write(&path, source).unwrap();
        let uri = DocumentUri::from_path(&path).unwrap();
        let mut session = session(Rc::new(NativeFs));
        session.set_limit_notices(false);
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.diagnostics(&uri).unwrap();
        assert_eq!(session.limit_notice_counts(), (0, 0, 0));
        session.set_limit_notices(true);
        assert!(session.take_limit_notices().is_empty());
        session.set_evaluation_budget(32);
        session.diagnostics(&uri).unwrap();
        assert_eq!(session.limit_notice_counts().0, 1);
        session.reset_caches();
        assert_eq!(session.limit_notice_counts(), (0, 0, 0));
        session.diagnostics(&uri).unwrap();
        assert_eq!(
            session.limit_notice_counts().0,
            1,
            "cache reset preserves enabled subscription"
        );
    }
}
