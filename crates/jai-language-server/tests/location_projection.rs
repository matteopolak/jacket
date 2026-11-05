//! Real alias retargeting during URI projection must never mix accepted identities.
#![cfg(unix)]
use jai_language_server as projection_api;
#[path = "support/location_projection.rs"]
mod support;

#[test]
fn alias_retarget_during_location_mapping_cannot_change_an_accepted_answer_uri() {
    support::exercise_query(
        |_| {},
        false,
        support::REFERENCES,
        0,
        3,
        |session, uri, position| session.references(uri, position, true),
    );
}

#[test]
fn alias_retarget_after_input_mapping_rejects_the_mixed_context() {
    input_mapping();
}

#[test]
fn alias_retarget_during_definition_mapping_cannot_change_the_answer_uri() {
    support::exercise_query(
        |_| {},
        false,
        SOURCE,
        2,
        1,
        jai_language_server::Session::definition,
    );
}

#[test]
fn alias_retarget_during_type_mapping_cannot_change_the_answer_uri() {
    support::exercise_query(
        |_| {},
        false,
        "Thing :: struct { value: int; }\nconsume :: (value: Thing) {}\nmain :: () { local: Thing; local.value = 2; consume(local); }\n",
        0,
        1,
        jai_language_server::Session::type_definition,
    );
}

#[test]
fn alias_retarget_during_related_diagnostic_mapping_cannot_change_the_answer_uri() {
    support::exercise_query(
        |_| {},
        false,
        "main :: () { local := 1; local := 2; }\n",
        0,
        1,
        |session, uri, _| {
            session.diagnostics(uri).map(|rows| {
                rows.into_iter()
                    .flat_map(|row| row.related)
                    .map(|related| projection_api::Location {
                        uri: related.uri,
                        range: related.range,
                    })
                    .collect()
            })
        },
    );
}

const SOURCE: &str = support::REFERENCES;

use projection_api as api;
use std::{cell::Cell, rc::Rc};
use support::RetargetFs;
use test_support::lsp::at_nth;

/// The request's physical spelling must not be mapped to a buffer using an unguarded alias.
fn input_mapping() {
    input_mapping_consumer(0);
}

#[test]
fn highlights_reject_retargeted_input_identity() {
    input_mapping_consumer(1);
}

#[test]
fn prepare_rename_rejects_retargeted_input_identity() {
    input_mapping_consumer(2);
}

#[test]
fn rename_rejects_retargeted_input_identity() {
    input_mapping_consumer(3);
}

#[test]
fn linked_ranges_reject_retargeted_input_identity() {
    input_mapping_consumer(4);
}

fn input_mapping_consumer(consumer: usize) {
    let temp = test_support::TempDir::new("input-uri-projection");
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let alias = temp.path().join("alias");
    let source = "main :: () { local := 1; local += 2; local += 3; }\n";
    for dir in [&first, &second] {
        std::fs::create_dir(dir).unwrap();
        std::fs::write(dir.join("main.jai"), source).unwrap();
    }
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let fs = Rc::new(RetargetFs {
        alias: alias.clone(),
        second,
        canonical_calls: Cell::new(0),
        retarget_at: Cell::new(None),
        changed: Cell::new(false),
        retarget_after: true,
    });
    let mut session = api::Session::with_environment(
        api::Limits::default(),
        api::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = api::DocumentUri::from_path(&alias.join("main.jai")).unwrap();
    let physical = api::DocumentUri::from_path(&first.join("main.jai")).unwrap();
    let position = at_nth(source, "local", 0, 1);
    session.open(uri, 1, source.into()).unwrap();
    session.diagnostics(&physical).unwrap();
    let query = || -> Result<usize, api::Error> {
        Ok(match consumer {
            0 => session.references(&physical, position, true)?.len(),
            1 => session.document_highlights(&physical, position)?.len(),
            2 => usize::from(session.prepare_rename(&physical, position)?.is_some()),
            3 => session
                .rename(&physical, position, "renamed")?
                .map_or(0, |groups| {
                    groups.into_iter().map(|(_, edits)| edits.len()).sum()
                }),
            4 => session.linked_editing_ranges(&physical, position)?.len(),
            _ => unreachable!(),
        })
    };
    assert_eq!(
        query().unwrap(),
        if consumer == 2 {
            1
        } else {
            3
        }
    );
    let compiled = session.compile_counts().0;
    fs.canonical_calls.set(0);
    fs.retarget_at.set(Some(2));
    let found = query().unwrap_or_default();
    assert!(
        fs.changed.get(),
        "actual input alias response must trigger retarget"
    );
    assert!(
        found == 0,
        "request alias projection changed before acceptance: {found:?}"
    );
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "mixed input identity must not authorize a new check"
    );
}

#[test]
fn alias_retarget_during_call_target_mapping_cannot_change_the_answer_uri() {
    support::exercise_query(
        |_| {},
        false,
        SOURCE,
        0,
        1,
        |session, uri, _| {
            let item = session
                .prepare_call_hierarchy(uri, at_nth(SOURCE, "main", 0, 1))?
                .into_iter()
                .next()
                .expect("known main declaration");
            session.outgoing_calls(&item).map(|calls| {
                calls
                    .into_iter()
                    .map(|call| api::Location {
                        uri: call.item.uri,
                        range: call.item.selection_range,
                    })
                    .collect()
            })
        },
    );
}

#[test]
fn preparing_an_external_call_never_combines_an_old_definition_with_new_source() {
    let (_dir, session, fs, uri, source) = external_rewrite_fixture();
    let position = at_nth(source, "leaf", 0, 1);
    let query = || session.prepare_call_hierarchy(&uri, position).unwrap();
    assert_eq!(query()[0].name, "leaf");
    fs.reads.set(0);
    let expected = query();
    let reads = fs.reads.get();
    assert!(reads > 0);
    fs.reads.set(0);
    assert_eq!(query(), expected);
    assert_eq!(fs.reads.get(), reads, "stable native control");
    let compiled = session.compile_counts().0;
    fs.reads.set(0);
    fs.rewrite_at.set(Some(reads));
    let found = query();
    assert!(fs.changed.get());
    assert!(
        found.is_empty() || found == expected,
        "old definition interpreted using later source: {found:?}"
    );
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "unstable derivation must not retry a check"
    );
    let settled = query();
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].name, "leaf");
    assert_eq!(settled[0].selection_range.start.line, 1);
    assert_eq!(session.compile_counts().0, compiled + 1);
}

#[test]
fn alias_retarget_during_doc_link_definition_mapping_cannot_change_the_answer_uri() {
    const SOURCE: &str =
        "target :: () {}\n/// See [target].\nmain :: () { local := 1; local += 2; target(); }\n";
    support::exercise_query(
        |_| {},
        false,
        SOURCE,
        0,
        1,
        |session, uri, _| session.definition(uri, at_nth(SOURCE, "[target]", 0, 3)),
    );
}

use jaic::sema::{FileSystem, NativeFs};
use std::path::{Path, PathBuf};

struct RewriteFs {
    path: PathBuf,
    reads: Cell<usize>,
    rewrite_at: Cell<Option<usize>>,
    interrupt_at: Cell<Option<usize>>,
    changed: Cell<bool>,
}

impl FileSystem for RewriteFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        if path == self.path {
            let count = self.reads.get() + 1;
            self.reads.set(count);
            if self.rewrite_at.get() == Some(count) {
                self.rewrite_at.set(None);
                self.changed.set(true);
                std::fs::write(&self.path, "evil :: () {}\nleaf :: () {}\n").unwrap();
            }
        }
        let bytes = NativeFs.read(path);
        if path == self.path && self.interrupt_at.get() == Some(self.reads.get()) {
            self.interrupt_at.set(None);
            jaic::interrupt::request(true);
            assert!(jaic::interrupt::should_stop());
        }
        bytes
    }

    fn is_file(&self, path: &Path) -> bool {
        NativeFs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        NativeFs.is_dir(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        NativeFs.list_dir(path)
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        NativeFs.stamp(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        NativeFs.canonical(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        NativeFs.canonical_exact(path)
    }
}

/// A closed dependency can change between selection and the final native replay.
fn external_rewrite_fixture() -> (
    test_support::TempDir,
    api::Session,
    Rc<RewriteFs>,
    api::DocumentUri,
    &'static str,
) {
    let temp = test_support::TempDir::new("prepare-call-source-epoch");
    let main = temp.path().join("main.jai");
    let callee = temp.path().join("callee.jai");
    let source = "#load \"callee.jai\"; main :: () { leaf(); }\n";
    std::fs::write(&main, source).unwrap();
    std::fs::write(&callee, "leaf :: () {}\n").unwrap();
    let fs = Rc::new(RewriteFs {
        path: callee,
        reads: Cell::new(0),
        rewrite_at: Cell::new(None),
        interrupt_at: Cell::new(None),
        changed: Cell::new(false),
    });
    let mut session = api::Session::with_environment(
        api::Limits::default(),
        api::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = api::DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    (temp, session, fs, uri, source)
}

#[test]
fn failed_live_reference_guard_does_not_start_a_same_request_recheck() {
    let (_dir, session, fs, uri, source) = external_rewrite_fixture();
    let position = at_nth(source, "leaf", 0, 1);
    let query = || session.references(&uri, position, true).unwrap();
    assert_eq!(query().len(), 2);
    fs.reads.set(0);
    let expected = query();
    let reads = fs.reads.get();
    assert!(reads > 0);
    fs.reads.set(0);
    assert_eq!(query(), expected);
    assert_eq!(fs.reads.get(), reads);
    let compiled = session.compile_counts().0;
    fs.reads.set(0);
    fs.rewrite_at.set(Some(reads));
    let found = query();
    assert!(fs.changed.get());
    assert!(found.is_empty() || found == expected);
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "a selected failed live transaction must not fall through to a new compiler"
    );
    let settled = query();
    assert_eq!(settled.len(), 2);
    assert!(
        settled
            .iter()
            .any(|location| location.uri.ends_with("callee.jai") && location.range.start.line == 1)
    );
    assert_eq!(session.compile_counts().0, compiled + 1);
}

#[test]
fn alias_retarget_during_cost_mapping_cannot_change_the_answer_uri() {
    const SOURCE: &str = "count :: (n: int) -> int { total := 0; for 0..n total += it; return total; }\nvalue :: #run count(5);\nmain :: () { local := value; local += 2; }\n";
    support::exercise_query(
        |_| {},
        false,
        SOURCE,
        0,
        1,
        |session, uri, _| {
            Ok(session
                .compile_time_costs(uri)
                .into_iter()
                .map(|row| row.location)
                .collect())
        },
    );
}

#[test]
fn a_self_load_link_cannot_mix_resolution_with_a_later_alias_mapping() {
    let temp = test_support::TempDir::new("load-link-projection");
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let alias = temp.path().join("alias");
    let source = "#load \"main.jai\"; main :: () {}";
    for dir in [&first, &second] {
        std::fs::create_dir(dir).unwrap();
        std::fs::write(dir.join("main.jai"), source).unwrap();
    }
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let fs = Rc::new(RetargetFs {
        alias: alias.clone(),
        second,
        canonical_calls: Cell::new(0),
        retarget_at: Cell::new(None),
        changed: Cell::new(false),
        retarget_after: false,
    });
    let mut session = api::Session::with_environment(
        api::Limits::default(),
        api::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = api::DocumentUri::from_path(&alias.join("main.jai")).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    fs.canonical_calls.set(0);
    let expected = session.document_links(&uri).unwrap();
    assert_eq!(expected.len(), 1);
    assert_eq!(expected[0].1, uri.as_str());
    let calls = fs.canonical_calls.get();
    assert!(calls > 0);
    fs.canonical_calls.set(0);
    assert_eq!(session.document_links(&uri).unwrap(), expected);
    assert_eq!(fs.canonical_calls.get(), calls);
    let compiled = session.compile_counts().0;
    fs.canonical_calls.set(0);
    fs.retarget_at.set(Some(calls));
    let found = session.document_links(&uri).unwrap();
    assert!(fs.changed.get());
    assert!(
        found.is_empty() || found == expected,
        "mixed load resolution and later identity: {found:?}"
    );
    assert_eq!(session.compile_counts().0, compiled);
    assert_eq!(session.document_links(&uri).unwrap(), expected);
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "literal load links do not require a compiler"
    );
}

fn physical_navigation(consumer: usize) {
    let dir = test_support::lsp::project(
        "navigation-client-spelling",
        &[("first/main.jai", support::REFERENCES)],
    );
    let alias = dir.join("alias");
    std::os::unix::fs::symlink(dir.join("first"), &alias).unwrap();
    let (mut session, _, text) = test_support::lsp::open_file(&dir, "first/main.jai");
    let physical = api::DocumentUri::from_path(&dir.join("first/main.jai")).unwrap();
    session.close(&physical).unwrap();
    let stored = api::DocumentUri::from_path(&alias.join("main.jai")).unwrap();
    session.open(stored.clone(), 1, text.clone()).unwrap();
    let position = at_nth(&text, "local", 0, 1);
    assert!(session.diagnostics(&physical).unwrap().is_empty());
    match consumer {
        0 => assert_eq!(
            session
                .document_highlights(&physical, position)
                .unwrap()
                .len(),
            3
        ),
        1 => assert!(
            session
                .prepare_rename(&physical, position)
                .unwrap()
                .is_some()
        ),
        2 => {
            let edits = session
                .rename(&physical, position, "renamed")
                .unwrap()
                .unwrap();
            assert_eq!(edits.len(), 1);
            assert_eq!(edits[0].0, stored.as_str());
            assert_eq!(edits[0].1.len(), 3);
        }
        3 => assert_eq!(
            session
                .linked_editing_ranges(&physical, position)
                .unwrap()
                .len(),
            3
        ),
        _ => unreachable!(),
    }
}

#[test]
fn physical_request_for_aliased_buffer_keeps_document_highlights() {
    physical_navigation(0);
}

#[test]
fn physical_request_for_aliased_buffer_can_prepare_rename() {
    physical_navigation(1);
}

#[test]
fn physical_request_for_aliased_buffer_can_rename() {
    physical_navigation(2);
}

#[test]
fn physical_request_for_aliased_buffer_keeps_linked_ranges() {
    physical_navigation(3);
}

#[test]
fn failed_prepare_rename_member_guard_does_not_start_a_same_request_recheck() {
    let (_dir, session, fs, uri, source) = external_rewrite_fixture();
    let position = at_nth(source, "leaf", 0, 1);
    let expected = session.prepare_rename(&uri, position).unwrap();
    assert!(expected.is_some());
    fs.reads.set(0);
    assert_eq!(session.prepare_rename(&uri, position).unwrap(), expected);
    assert!(
        fs.reads.get() >= 3,
        "discovery, selection and first final guard control"
    );
    let compiled = session.compile_counts().0;
    fs.reads.set(0);
    fs.rewrite_at.set(Some(3));
    let found = session.prepare_rename(&uri, position).unwrap();
    assert!(fs.changed.get());
    assert!(
        found.is_none(),
        "a failed selected member query is unavailable: {found:?}"
    );
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "prepareRename must not reinterpret a failed member guard as an accepted nonmember and compile again"
    );
    assert_eq!(session.prepare_rename(&uri, position).unwrap(), expected);
    assert_eq!(session.compile_counts().0, compiled + 1);
}


#[test]
fn failed_published_reference_guard_preserves_restored_evicted_facts() {
    let (_dir, session, fs, uri, source) = external_rewrite_fixture();
    let position = at_nth(source, "leaf", 0, 1);
    let expected = session.references(&uri, position, true).unwrap();
    assert_eq!(expected.len(), 2);
    let modified = std::fs::metadata(&fs.path).unwrap().modified().unwrap();
    let compiled = session.compile_counts().0;
    fs.reads.set(0);
    fs.rewrite_at.set(Some(2));
    assert!(session.references(&uri, position, true).unwrap().is_empty());
    assert!(fs.changed.get());
    assert_eq!(session.compile_counts().0, compiled);
    std::fs::write(&fs.path, "leaf :: () {}\n").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&fs.path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    session.defer_compiles(true);
    assert_eq!(session.references(&uri, position, true).unwrap(), expected);
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "independently frozen restored facts survive retirement of their invalid live compiler"
    );
}


#[test]
fn interruption_during_published_reference_replay_returns_no_answer() {
    struct Armed;
    impl Drop for Armed {
        fn drop(&mut self) {
            jaic::interrupt::disarm();
            jaic::interrupt::request(false);
        }
    }
    let (_dir, session, fs, uri, source) = external_rewrite_fixture();
    let position = at_nth(source, "leaf", 0, 1);
    let expected = session.references(&uri, position, true).unwrap();
    assert_eq!(expected.len(), 2);
    let compiled = session.compile_counts().0;
    fs.reads.set(0);
    fs.interrupt_at.set(Some(2));
    jaic::interrupt::request(false);
    jaic::interrupt::arm();
    let armed = Armed;
    assert!(session.references(&uri, position, true).unwrap().is_empty());
    assert!(jaic::interrupt::fired());
    assert!(!fs.changed.get(), "this case changes no native dependency");
    assert_eq!(session.compile_counts().0, compiled);
    drop(armed);
    session.defer_compiles(true);
    assert_eq!(session.references(&uri, position, true).unwrap(), expected);
    assert_eq!(session.compile_counts().0, compiled);
}
