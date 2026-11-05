//! Untimed native-context filesystem work accounting. The optional collector is test-only.
//! Run `cargo test -p jai-language-server --lib native_protocol_fs_work -- --ignored --nocapture --test-threads=1`.
use super::*;
use crate::{Environment, JsonSession, Limits};
use jaic::sema::{FileSystem, NativeFs, Options};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
struct Counts {
    calls: usize,
    bytes: usize,
    missing: usize,
    paths: BTreeMap<PathBuf, usize>,
}

struct Pass {
    kind: &'static str,
    entries: usize,
    parent: Option<usize>,
    counts: BTreeMap<(&'static str, &'static str), Counts>,
}

struct Work {
    current: usize,
    passes: Vec<Pass>,
}

thread_local! {
    static WORK: RefCell<Option<Work>> = const { RefCell::new(None) };
    static OVERLAY: RefCell<Option<OverlayWork>> = const { RefCell::new(None) };
    static COST_PATHS: RefCell<Option<std::collections::BTreeSet<PathBuf>>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct OverlayWork {
    current: &'static str,
    uses: BTreeMap<PathBuf, std::collections::BTreeSet<&'static str>>,
    uncached_files: std::collections::BTreeSet<PathBuf>,
}

pub(crate) struct OverlayScope(&'static str, std::marker::PhantomData<Rc<()>>);

impl Drop for OverlayScope {
    fn drop(&mut self) {
        OVERLAY.with_borrow_mut(|slot| {
            if let Some(work) = slot {
                work.current = self.0;
            }
        });
    }
}

/// Attribute nested overlay identity work to its actual caller; dropping restores the parent.
/// For example, a file probe includes only the ancestor bookkeeping it actually requested.
pub(crate) fn overlay_scope(kind: &'static str) -> Option<OverlayScope> {
    OVERLAY.with_borrow_mut(|slot| {
        let work = slot.as_mut()?;
        let previous = std::mem::replace(&mut work.current, kind);
        Some(OverlayScope(previous, std::marker::PhantomData))
    })
}

/// Record an actual identity use without evaluating paths or cache lookups when disabled.
/// Callers pass a closure such as `|| (raw_key, false)` rather than computing it first.
pub(crate) fn overlay_identity(observe: impl FnOnce() -> (PathBuf, bool)) {
    OVERLAY.with_borrow_mut(|slot| {
        if let Some(work) = slot {
            let (path, first_file) = observe();
            work.uses
                .entry(path.clone())
                .or_default()
                .insert(work.current);
            if first_file {
                work.uncached_files.insert(path);
            }
        }
    });
}

/// Preserve canonical requirements from producers outside the overlay before rows coalesce.
/// A config discovery lookup, for example, cannot be removed with an unrelated file probe.
pub(crate) fn outside_identity(path: &Path) {
    OVERLAY.with_borrow_mut(|slot| {
        if let Some(work) = slot {
            work.uses
                .entry(path.to_path_buf())
                .or_default()
                .insert("outside");
        }
    });
}

struct OverlayRecording;

impl Drop for OverlayRecording {
    fn drop(&mut self) {
        OVERLAY.with_borrow_mut(|slot| *slot = None);
        COST_PATHS.with_borrow_mut(|slot| *slot = None);
    }
}

pub(crate) struct Scope {
    previous: usize,
    _same_thread: std::marker::PhantomData<Rc<()>>,
}

impl Drop for Scope {
    fn drop(&mut self) {
        WORK.with_borrow_mut(|work| {
            if let Some(work) = work {
                work.current = self.previous;
            }
        });
    }
}

/// Keep nested replay operations exclusively attributed to their own ordered pass.
pub(crate) fn scope(kind: &'static str, entries: usize) -> Option<Scope> {
    WORK.with_borrow_mut(|slot| {
        let work = slot.as_mut()?;
        let previous = work.current;
        work.current = work.passes.len();
        work.passes.push(Pass {
            kind,
            entries,
            parent: Some(previous),
            counts: BTreeMap::new(),
        });
        Some(Scope {
            previous,
            _same_thread: std::marker::PhantomData,
        })
    })
}

struct Recording(bool);

impl Recording {
    fn start() -> Self {
        WORK.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = Some(Work {
                current: 0,
                passes: vec![Pass {
                    kind: "other",
                    entries: 0,
                    parent: None,
                    counts: BTreeMap::new(),
                }],
            });
        });
        Self(true)
    }

    fn finish(mut self) -> Value {
        self.0 = false;
        let work = WORK.with_borrow_mut(Option::take).unwrap();
        if let Some(pass) = work
            .passes
            .iter()
            .rev()
            .find(|pass| pass.kind == "history_replay")
        {
            COST_PATHS.with_borrow_mut(|slot| {
                if let Some(paths) = slot {
                    for ((operation, _), counts) in &pass.counts {
                        if *operation == "canonical" {
                            paths.extend(counts.paths.keys().cloned());
                        }
                    }
                }
            });
        }
        Value::Array(work.passes.into_iter().enumerate().map(|(ordinal, pass)| {
            let mut canonical_classes = BTreeMap::new();
            for ((operation, category), counts) in &pass.counts {
                if *operation != "canonical" { continue; }
                for path in counts.paths.keys() {
                    // Classification happens after the request and is diagnostic only.
                    let state = match std::fs::metadata(path) {
                        Ok(meta) if meta.is_file() => "file",
                        Ok(meta) if meta.is_dir() => "directory",
                        Ok(_) => "other_existing",
                        Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => "definitely_missing",
                        Err(_) => "unknown_error",
                    };
                    let used = |kind| pass.counts.get(&(kind, *category)).is_some_and(|row| row.paths.contains_key(path));
                    let origin = OVERLAY.with_borrow(|slot| {
                        let Some(work) = slot else { return "not_collected"; };
                        match work.uses.get(path) {
                            Some(kinds) if kinds.len() == 1 && kinds.contains("file") && work.uncached_files.contains(path) => "uncached_file_only",
                            Some(kinds) if kinds.len() == 1 && kinds.contains("file") => "file_only_ancestor",
                            Some(_) => "other_overlay_use",
                            None => "outside_overlay",
                        }
                    });
                    let shape = if used("read") || used("directory") || used("listing") || used("stamp") { "other_operation" } else if used("file") { "file_probe_only" } else { "identity_only" };
                    *canonical_classes.entry(format!("{category}/{state}/{shape}/{origin}")).or_insert(0usize) += 1;
                }
            }
            let rows: Vec<_> = pass.counts.into_iter().map(|((operation, category), count)| {
                let mut multiplicity = BTreeMap::new();
                for calls in count.paths.values() { *multiplicity.entry(*calls).or_insert(0usize) += 1; }
                json!({"operation":operation,"category":category,"calls":count.calls,"read_bytes":count.bytes,"missing_reads":count.missing,"distinct_paths":count.paths.len(),"path_call_multiplicity":multiplicity})
            }).collect();
            json!({"ordinal":ordinal,"parent":pass.parent,"phase":pass.kind,"proof_entries":pass.entries,"canonical_post_request_classification":canonical_classes,"operations":rows})
        }).collect())
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        if self.0 {
            WORK.with_borrow_mut(|slot| *slot = None);
        }
    }
}

struct CountingFs {
    stdlib: PathBuf,
    project: PathBuf,
}

impl CountingFs {
    fn record(&self, operation: &'static str, path: &Path, bytes: Option<usize>) {
        WORK.with_borrow_mut(|slot| {
            let Some(work) = slot else {
                return;
            };
            let category = if path.starts_with(&self.stdlib) {
                "stdlib"
            } else if path.starts_with(&self.project) {
                "project"
            } else {
                "other"
            };
            let counts = work.passes[work.current]
                .counts
                .entry((operation, category))
                .or_default();
            counts.calls += 1;
            counts.bytes += bytes.unwrap_or(0);
            counts.missing += usize::from(operation == "read" && bytes.is_none());
            *counts.paths.entry(path.to_path_buf()).or_default() += 1;
        });
    }
}

impl FileSystem for CountingFs {
    fn is_native(&self) -> bool {
        NativeFs.is_native()
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        let value = NativeFs.read(path);
        self.record("read", path, value.as_ref().map(Vec::len));
        value
    }

    fn is_file(&self, path: &Path) -> bool {
        let value = NativeFs.is_file(path);
        self.record("file", path, None);
        value
    }

    fn is_dir(&self, path: &Path) -> bool {
        let value = NativeFs.is_dir(path);
        self.record("directory", path, None);
        value
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        let value = NativeFs.canonical(path);
        self.record("canonical", path, None);
        value
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        let value = NativeFs.missing_file_key(path);
        self.record("absence_probe", path, None);
        value
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        let value = NativeFs.canonical_exact(path);
        self.record("canonical_exact", path, None);
        value
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let value = NativeFs.list_dir(path);
        self.record("listing", path, None);
        value
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        let value = NativeFs.stamp(path);
        self.record("stamp", path, None);
        value
    }
}

fn send(session: &mut JsonSession, body: Value) -> Vec<Value> {
    let message = serde_json::from_value(body).unwrap();
    let mut output = Vec::new();
    session
        .dispatch_messages(vec![message], |messages| {
            output.extend(
                messages
                    .into_iter()
                    .map(|message| serde_json::to_value(message).unwrap()),
            );
            Ok(())
        })
        .unwrap();
    output
}

fn request(session: &mut JsonSession, id: i32, method: &str, params: Value) -> Value {
    let output = send(
        session,
        json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}),
    );
    let reply = output
        .iter()
        .find(|reply| reply["id"] == id)
        .expect("response");
    assert!(reply.get("error").is_none(), "{method}: {}", reply["error"]);
    reply["result"].clone()
}

fn position(text: &str, snippet: &str) -> Value {
    let at = text
        .match_indices(snippet)
        .map(|(at, _)| at)
        .find(|at| {
            *at == 0
                || !(text.as_bytes()[at - 1].is_ascii_alphanumeric()
                    || text.as_bytes()[at - 1] == b'_')
        })
        .expect("fixture selection");
    let before = &text[..at];
    json!({"line":before.bytes().filter(|byte| *byte==b'\n').count(),"character":before.rsplit('\n').next().unwrap().encode_utf16().count()})
}

fn record_index(session: &Session, uri: &DocumentUri) -> Option<Rc<ReferenceIndex>> {
    session
        .reference_records
        .borrow()
        .records
        .iter()
        .rev()
        .find(|record| &record.uri == uri)
        .map(|record| record.snapshot.index.clone())
}

fn eligible(session: &Session, uri: &DocumentUri) -> Option<bool> {
    let memo = session.inputs.borrow();
    let (_, last, _, (files, root, dirs)) = memo.as_ref()?;
    if last != uri {
        return Some(false);
    }
    let records = session.reference_records.borrow();
    let record = records
        .records
        .iter()
        .rev()
        .find(|record| &record.uri == uri)?;
    Some(
        session
            .semantic
            .borrow()
            .published_reference_inputs(
                root,
                dirs,
                files,
                &record.snapshot.owner,
                &record.snapshot.index,
            )
            .is_some(),
    )
}

#[test]
#[ignore = "manual untimed native protocol filesystem operation accounting"]
fn native_protocol_fs_work() {
    protocol_fs_work(
        &[
            "textDocument/hover",
            "textDocument/prepareRename",
            "textDocument/rename",
            "textDocument/references",
            "textDocument/linkedEditingRange",
            "textDocument/diagnostic",
        ],
        &[false, true],
        3,
    );
}

#[test]
#[ignore = "manual post-request classification, not accepted filesystem authority or timing"]
fn native_missing_canonical_fraction() {
    let _cleanup = OverlayRecording;
    COST_PATHS.with_borrow_mut(|slot| *slot = Some(std::collections::BTreeSet::default()));
    OVERLAY.with_borrow_mut(|slot| *slot = Some(OverlayWork::default()));
    protocol_fs_work(&["textDocument/references"], &[true], 1);
    OVERLAY.with_borrow_mut(|slot| *slot = None);
    let paths = COST_PATHS.with_borrow_mut(Option::take).unwrap();
    // Private diagnostic handoff, never a source artifact or accepted input proof.
    let body = paths
        .iter()
        .map(|path| path.to_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(
        std::env::temp_dir().join("lsp-canonical-cost-inputs.txt"),
        body,
    )
    .unwrap();
}

fn protocol_fs_work(methods: &[&str], modes: &[bool], samples: usize) {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let stdlib = repository.join("stdlib").canonicalize().unwrap();
    let capabilities: Value =
        serde_json::from_str(include_str!("../../../tests/lsp/client-capabilities.json")).unwrap();
    let mut id = 1;
    for (label, relative, binding, declaration, references, edits) in [
        (
            "small",
            "tools/bench/fixtures/lsp-small/main.jai",
            "result := value",
            "twice ::",
            3,
            3,
        ),
        (
            "medium",
            "examples/kvdb/main.jai",
            "store: Store;",
            "run_local ::",
            13,
            2,
        ),
        (
            "editor",
            "examples/editor/editor.jai",
            "doc: *Document",
            "status_line ::",
            3,
            2,
        ),
    ] {
        let path = repository.join(relative).canonicalize().unwrap();
        let uri = DocumentUri::from_path(&path).unwrap();
        let root_uri = DocumentUri::from_path(path.parent().unwrap()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let mut expected = BTreeMap::new();
        for &native_options in modes {
            OVERLAY.with_borrow_mut(|slot| {
                if slot.is_some() {
                    *slot = Some(OverlayWork::default());
                }
            });
            let fs = Rc::new(CountingFs {
                stdlib: stdlib.clone(),
                project: path.parent().unwrap().to_path_buf(),
            });
            let library = stdlib.clone();
            let services = jaic::service::Services::new(jaic::service::ServiceConfig::default());
            let environment = Environment {
                fs,
                services: Some(Rc::new(crate::services::EditorServices::new(services))),
                options: Box::new(move |fs, main| {
                    if native_options {
                        let inputs = jaic::project::ProgramInputs::for_file(fs, main, &library);
                        Options::for_program(fs, main, &inputs)
                    } else {
                        let mut options = Options::host();
                        options.import_paths = vec![library.clone()];
                        options.set_stdlib(fs, &library);
                        options
                    }
                }),
            };
            let mut session = JsonSession::with_environment(Limits::default(), environment);
            session.defer_publications(true);
            session.defer_compiles(true);
            request(
                &mut session,
                id,
                "initialize",
                json!({"processId":null,"rootUri":root_uri.as_str(),"capabilities":capabilities}),
            );
            id += 1;
            send(
                &mut session,
                json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
            );
            send(
                &mut session,
                json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri.as_str(),"languageId":"jai","version":1,"text":text}}}),
            );
            let document = json!({"uri":uri.as_str()});
            let diagnostics = request(
                &mut session,
                id,
                "textDocument/diagnostic",
                json!({"textDocument":document}),
            );
            id += 1;
            assert_eq!(diagnostics["items"], json!([]));
            for &method in methods {
                for sample in 0..samples {
                    let params = match method {
                        "textDocument/diagnostic" => json!({"textDocument":document}),
                        "textDocument/references" => {
                            json!({"textDocument":document,"position":position(&text,binding),"context":{"includeDeclaration":true}})
                        }
                        "textDocument/rename" => {
                            json!({"textDocument":document,"position":position(&text,declaration),"newName":"renamed_work_probe"})
                        }
                        "textDocument/prepareRename" => {
                            json!({"textDocument":document,"position":position(&text,declaration)})
                        }
                        _ => json!({"textDocument":document,"position":position(&text,binding)}),
                    };
                    let before_index = record_index(session.session(), &uri);
                    let before_memo = session
                        .session()
                        .inputs
                        .borrow()
                        .as_ref()
                        .map(|memo| memo.3.0.clone());
                    let view_eligible = eligible(session.session(), &uri);
                    let before_compiles = session.compile_counts();
                    let recording = Recording::start();
                    let result = request(&mut session, id, method, params);
                    id += 1;
                    let passes = recording.finish();
                    assert_eq!(
                        session.compile_counts(),
                        before_compiles,
                        "warm request must not compile"
                    );
                    match method {
                        "textDocument/references" => {
                            assert_eq!(result.as_array().unwrap().len(), references);
                        }
                        "textDocument/linkedEditingRange" => {
                            assert_eq!(result["ranges"].as_array().unwrap().len(), references);
                        }
                        "textDocument/rename" => assert_eq!(
                            result["changes"]
                                .as_object()
                                .unwrap()
                                .values()
                                .map(|rows| rows.as_array().unwrap().len())
                                .sum::<usize>(),
                            edits
                        ),
                        "textDocument/diagnostic" => assert_eq!(result["items"], json!([])),
                        _ => assert!(!result.is_null()),
                    }
                    if let Some(previous) = expected.get(method) {
                        assert_eq!(
                            &result, previous,
                            "fixed/native and repeated results must agree"
                        );
                    } else {
                        expected.insert(method, result.clone());
                    }
                    let after_index = record_index(session.session(), &uri);
                    let same_index = before_index
                        .as_ref()
                        .zip(after_index.as_ref())
                        .is_some_and(|(a, b)| Rc::ptr_eq(a, b));
                    let after_memo = session
                        .session()
                        .inputs
                        .borrow()
                        .as_ref()
                        .map(|memo| memo.3.0.clone());
                    let same_memo = before_memo
                        .as_ref()
                        .zip(after_memo.as_ref())
                        .is_some_and(|(a, b)| Rc::ptr_eq(a, b));
                    println!(
                        "{}",
                        json!({"file":label,"native_options":native_options,"method":method,"sample":sample,"published_view_eligible":view_eligible,"reference_index_same_rc":same_index,"input_memo_same_rc":same_memo,"passes":passes})
                    );
                }
            }
        }
    }
}

#[test]
fn rename_preparation_and_edits_use_one_checked_transaction() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let stdlib = repository.join("stdlib");
    let path = repository.join("tools/bench/fixtures/lsp-small/main.jai");
    let uri = DocumentUri::from_path(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let mut session = JsonSession::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(move |fs, main| {
                let inputs = jaic::project::ProgramInputs::for_file(fs, main, &stdlib);
                Options::for_program(fs, main, &inputs)
            }),
        },
    );
    session.defer_publications(true);
    session.defer_compiles(true);
    request(
        &mut session,
        1,
        "initialize",
        json!({"capabilities":serde_json::from_str::<Value>(include_str!("../../../tests/lsp/client-capabilities.json")).unwrap()}),
    );
    send(
        &mut session,
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
    );
    send(
        &mut session,
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri.as_str(),"languageId":"jai","version":1,"text":text}}}),
    );
    let document = json!({"uri":uri.as_str()});
    assert_eq!(
        request(
            &mut session,
            2,
            "textDocument/diagnostic",
            json!({"textDocument":document})
        )["items"],
        json!([])
    );
    let compile_counts = session.compile_counts();
    let mut replays = Vec::new();
    for (id, method) in [
        (3, "textDocument/prepareRename"),
        (4, "textDocument/rename"),
    ] {
        let recording = Recording::start();
        let result = request(
            &mut session,
            id,
            method,
            json!({"textDocument":document,"position":position(&text,"twice ::"),"newName":"renamed_work_probe"}),
        );
        let work = recording.finish();
        if method == "textDocument/rename" {
            assert_eq!(result["changes"][uri.as_str()].as_array().unwrap().len(), 3);
        } else {
            assert!(!result.is_null());
        }
        assert_eq!(session.compile_counts(), compile_counts);
        replays.push((
            method,
            work.as_array()
                .unwrap()
                .iter()
                .filter(|pass| {
                    pass["phase"] == "history_replay" && pass["proof_entries"].as_u64().unwrap() > 0
                })
                .count(),
        ));
    }
    assert_eq!(
        replays,
        vec![
            ("textDocument/prepareRename", 2),
            ("textDocument/rename", 2)
        ],
        "one selected preguard and one final semantic guard per accepted plan"
    );
}
