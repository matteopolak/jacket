//! Exact checked navigation independent of the bounded full-compiler cache.
use crate::semantic::{Analysis, Reads};
use crate::session::Session;
use crate::{DocumentUri, Location, Position};
use jaic::ide::meta::ReferenceIndex;
use jaic::source::FileId;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

#[cfg(all(test, unix))]
use crate::projection_fixture;

const BUDGET: usize = 16 << 20;

type Sources = BTreeMap<FileId, (String, Arc<str>)>;

struct Snapshot {
    validated_generation: std::cell::Cell<Option<u64>>,
    root: PathBuf,
    dirs: Vec<PathBuf>,
    overlays: BTreeMap<PathBuf, Arc<[u8]>>,
    reads: Rc<Reads>,
    index: Rc<ReferenceIndex>,
    owner: Rc<()>,
    sources: Sources,
    bytes: usize,
}

#[derive(Clone)]
struct Record {
    uri: DocumentUri,
    file: FileId,
    snapshot: Rc<Snapshot>,
}

/// Oldest checked records are evicted first. The compiler cache has a separate budget.
#[derive(Default)]
pub(crate) struct Store {
    records: Vec<Record>,
}

impl Store {
    pub(crate) fn clear(&mut self) {
        self.records.clear();
    }

    pub(crate) fn remove(&mut self, uri: &DocumentUri) {
        self.records.retain(|r| &r.uri != uri);
    }

    pub(crate) fn invalidate(&mut self, path: &std::path::Path) {
        self.records
            .retain(|r| !r.snapshot.reads.concerns_tree(path));
    }

    pub(crate) fn counts(&self) -> (usize, usize) {
        (self.records.len(), self.bytes())
    }

    fn bytes(&self) -> usize {
        let mut seen = std::collections::BTreeSet::new();
        self.records
            .iter()
            .map(|r| {
                std::mem::size_of::<Record>()
                    + r.uri.as_str().len()
                    + if seen.insert(Rc::as_ptr(&r.snapshot) as usize) {
                        r.snapshot.bytes
                    } else {
                        0
                    }
            })
            .sum()
    }

    fn trim(&mut self) {
        self.trim_to(BUDGET);
    }

    fn trim_to(&mut self, budget: usize) {
        let mut bytes = self.bytes();
        if bytes <= budget {
            return;
        }
        let mut owners = BTreeMap::new();
        for record in &self.records {
            *owners.entry(Rc::as_ptr(&record.snapshot)).or_insert(0usize) += 1;
        }
        let mut removed = 0;
        for record in &self.records {
            if bytes <= budget {
                break;
            }
            bytes -= std::mem::size_of::<Record>() + record.uri.as_str().len();
            let count = owners
                .get_mut(&Rc::as_ptr(&record.snapshot))
                .expect("counted snapshot");
            *count -= 1;
            if *count == 0 {
                bytes -= record.snapshot.bytes;
            }
            removed += 1;
        }
        self.records.drain(..removed);
    }
}

/// Fully owned candidate facts, not visible until the final query guard succeeds.
#[derive(Default)]
pub(crate) struct PreparedReferences(Vec<Record>);

impl Session {
    pub(crate) fn capture_reference_records(&self, analysis: &mut Analysis) -> PreparedReferences {
        let mut prepared = PreparedReferences::default();
        if jaic::interrupt::fired() || !analysis.reference_records_allowed() {
            return prepared;
        }
        let root = analysis.root.clone();
        let dirs = analysis.dirs.clone();
        let files = analysis.files.clone();
        let observed = analysis.reads.clone();
        let mut assigned = std::collections::BTreeSet::new();
        let mut programs = Vec::new();
        let Some(identities) = self.open_identities() else {
            return prepared;
        };
        let current: BTreeMap<_, _> = identities
            .iter()
            .filter_map(|(path, uri)| self.documents.get(uri).map(|doc| (path, doc)))
            .collect();
        for (compiler, owner, view) in analysis.reference_programs() {
            let exact_inputs = {
                let reads = observed.borrow();
                files
                    .iter()
                    .filter(|(path, _)| reads.concerns(path))
                    .all(|(path, text)| {
                        current
                            .get(path)
                            .is_some_and(|doc| doc.ide_text().as_bytes() == text.as_ref())
                    })
                    && current
                        .keys()
                        .filter(|path| reads.concerns(path))
                        .all(|path| files.contains_key(*path))
            };
            let selected: Vec<_> = self
                .documents
                .iter()
                .filter_map(|(uri, doc)| {
                    if assigned.contains(uri) {
                        return None;
                    }
                    let file =
                        crate::semantic::find_file(compiler, std::path::Path::new(uri.path()))?;
                    let source = compiler.src.sources.try_get(file)?;
                    let same_owner = self
                        .reference_records
                        .borrow()
                        .records
                        .iter()
                        .any(|record| {
                            &record.uri == uri && Rc::ptr_eq(&record.snapshot.owner, owner)
                        });
                    ((exact_inputs && doc.same_positions(&source.text)) || same_owner)
                        .then(|| (uri.clone(), file))
                })
                .collect();
            if selected.is_empty() {
                continue;
            }
            let index = view
                .get_or_insert_with(|| Rc::new(compiler.ide_reference_index()))
                .clone();
            let mut sources = Sources::new();
            for span in index.spans() {
                if let std::collections::btree_map::Entry::Vacant(entry) = sources.entry(span.file)
                    && let Some(source) = compiler.src.sources.try_get(span.file)
                {
                    entry.insert((source.path.to_string(), source.text.clone()));
                }
            }
            assigned.extend(selected.iter().map(|(uri, _)| uri.clone()));
            programs.push((selected, index, owner.clone(), sources));
        }
        // All source identity and discovery reads belong to every staged snapshot.
        observed.borrow_mut().disk.merge(&self.input_proof.borrow());
        let reads = Rc::new(observed.borrow().clone());
        let overlays: BTreeMap<_, _> = files
            .iter()
            .filter(|(path, _)| reads.concerns(path))
            .map(|(path, text)| (path.clone(), text.clone()))
            .collect();
        for (selected, index, owner, sources) in programs {
            let bytes = reads.retained_bytes()
                + index.retained_bytes()
                + std::mem::size_of::<Snapshot>()
                + sources
                    .values()
                    .map(|(path, text)| {
                        path.capacity()
                            + text.len()
                            + std::mem::size_of::<(FileId, (String, Arc<str>))>()
                            + 64
                    })
                    .sum::<usize>()
                + overlays
                    .iter()
                    .map(|(path, text)| {
                        path.capacity()
                            + text.len()
                            + std::mem::size_of::<(PathBuf, Arc<[u8]>)>()
                            + 64
                    })
                    .sum::<usize>()
                + root.capacity()
                + dirs.capacity() * std::mem::size_of::<PathBuf>()
                + dirs.iter().map(PathBuf::capacity).sum::<usize>();
            let snapshot = Rc::new(Snapshot {
                validated_generation: std::cell::Cell::new(None),
                root: root.clone(),
                dirs: dirs.clone(),
                overlays: overlays.clone(),
                reads: reads.clone(),
                index,
                owner,
                sources,
                bytes,
            });
            for (uri, file) in selected {
                prepared.0.push(Record {
                    uri,
                    file,
                    snapshot: snapshot.clone(),
                });
            }
        }
        prepared
    }

    /// Commit without filesystem access after the caller validates every staged observation.
    pub(crate) fn publish_reference_records(&self, prepared: PreparedReferences) {
        let mut store = self.reference_records.borrow_mut();
        for record in prepared.0 {
            store.remove(&record.uri);
            store.records.push(record);
        }
        store.trim();
    }

    /// Use an already published current Program view without a mutable compiler query.
    /// Outer `Some` owns this attempt, including a failed final guard.
    pub(crate) fn current_references(
        &self,
        uri: &DocumentUri,
        position: Position,
        declaration: bool,
    ) -> Option<Option<Vec<Location>>> {
        let doc = self.documents.get(uri)?;
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, &doc.text)?;
        let record = self
            .reference_records
            .borrow()
            .records
            .iter()
            .rev()
            .find(|record| &record.uri == uri)
            .cloned()?;
        let snapshot = &record.snapshot;
        if root != snapshot.root
            || dirs != snapshot.dirs
            || !doc.same_positions(&snapshot.sources.get(&record.file)?.1)
        {
            return None;
        }
        let (inputs, authority) = {
            let cache = self.semantic.borrow();
            (
                cache.published_reference_inputs(
                    &root,
                    &dirs,
                    &files,
                    &snapshot.owner,
                    &snapshot.index,
                )?,
                cache.authority().clone(),
            )
        };
        let answer = (|| {
            if jaic::interrupt::fired() {
                return None;
            }
            let fs = self.config_filesystem()?;
            let byte = doc.index.byte(&doc.text, position);
            let locations = snapshot
                .index
                .references(record.file, byte as u32)
                .into_iter()
                .filter(|(_, declared)| declaration || !declared)
                .filter_map(|(span, _)| {
                    let (path, text) = snapshot.sources.get(&span.file)?;
                    self.projected_location(&fs, path, text, span.start as usize, span.end as usize)
                })
                .collect();
            let mut output = snapshot.reads.disk.clone();
            output.merge(&self.input_proof.borrow());
            let mut reads =
                inputs.projected_reads(environment, &files, &authority, None, &output)?;
            reads.merge(&snapshot.reads);
            if jaic::interrupt::fired()
                || !reads.is_current(&*environment.fs)
                || jaic::interrupt::fired()
            {
                return None;
            }
            Some(locations)
        })();
        if answer.is_none() {
            self.semantic.borrow_mut().retire_published_reference_view(
                &root,
                &dirs,
                &files,
                &snapshot.owner,
                &snapshot.index,
            );
        }
        Some(answer)
    }

    pub(crate) fn retained_references(
        &self,
        uri: &DocumentUri,
        position: Position,
        declaration: bool,
    ) -> Option<Vec<Location>> {
        if self.may_compile() {
            return None;
        }
        let (uri, doc) = self.document_entry(uri).ok()?;
        let (files, root, dirs) = self.compile_inputs(uri, &doc.text)?;
        let store = self.reference_records.borrow();
        let record = store.records.iter().rev().find(|r| &r.uri == uri)?;
        let snapshot = &record.snapshot;
        if root != snapshot.root || dirs != snapshot.dirs {
            return None;
        }
        let environment = self.environment.as_ref()?;
        let fs = self.config_filesystem()?;
        if snapshot.validated_generation.get() != Some(self.semantic_revision()) {
            let current: BTreeMap<_, _> = files
                .iter()
                .map(|(path, text)| (crate::auto_import::filesystem_key(&fs, path), text.clone()))
                .filter(|(path, _)| snapshot.reads.concerns(path))
                .collect();
            if current != snapshot.overlays {
                return None;
            }
            let checked = &snapshot.sources.get(&record.file)?.1;
            if !doc.same_positions(checked) {
                return None;
            }
        }
        let byte = doc.index.byte(&doc.text, position);
        let locations = snapshot
            .index
            .references(record.file, byte as u32)
            .into_iter()
            .filter(|(_, decl)| declaration || !decl)
            .filter_map(|(span, _)| {
                let (path, text) = snapshot.sources.get(&span.file)?;
                self.projected_location(&fs, path, text, span.start as usize, span.end as usize)
            })
            .collect();
        // Mapping observations are part of this answer, not new authority attached after a
        // successful check. Freeze the union, then accept without further filesystem work.
        let mut reads = (*snapshot.reads).clone();
        reads.disk.merge(&self.input_proof.borrow());
        if !reads.is_current(&*environment.fs) {
            return None;
        }
        snapshot
            .validated_generation
            .set(Some(self.semantic_revision()));
        Some(locations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_reference_answers_keep_the_published_index() {
        let supplied = test_support::lsp::environment();
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                fs: supplied.fs,
                options: supplied.options,
                services: supplied.services,
            },
        );
        let fixture = test_support::TempDir::new("published-view");
        let uri = DocumentUri::from_path(&fixture.join("main.jai")).unwrap();
        let source =
            "consume :: (value: int) {} main :: () { local := 1; local += 2; consume(local); }";
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.diagnostics(&uri).unwrap();
        let index = session
            .reference_records
            .borrow()
            .records
            .iter()
            .find(|record| record.uri == uri)
            .unwrap()
            .snapshot
            .index
            .clone();
        let position = test_support::lsp::at_nth(source, "local", 0, 1);
        assert_eq!(session.references(&uri, position, true).unwrap().len(), 3);
        let store = session.reference_records.borrow();
        let after = &store
            .records
            .iter()
            .find(|record| record.uri == uri)
            .unwrap()
            .snapshot
            .index;
        assert!(
            Rc::ptr_eq(&index, after),
            "an immutable published reference query must not retire/rebuild its own semantic index"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_projection_validates_mapping_after_compiler_eviction() {
        projection_fixture::exercise_query(
            |session| {
                session.semantic.borrow_mut().forget_compiles();
                session.defer_compiles(true);
            },
            true,
            projection_fixture::REFERENCES,
            0,
            3,
            |session, uri, position| session.references(uri, position, true),
        );
    }

    fn record(name: &str, snapshot: Rc<Snapshot>) -> Record {
        Record {
            uri: DocumentUri::parse(name).unwrap(),
            file: FileId(0),
            snapshot,
        }
    }

    fn snapshot(bytes: usize) -> Rc<Snapshot> {
        Rc::new(Snapshot {
            validated_generation: std::cell::Cell::new(None),
            root: PathBuf::from("/main.jai"),
            dirs: vec![],
            overlays: BTreeMap::new(),
            reads: Rc::default(),
            index: Rc::new(ReferenceIndex::default()),
            owner: Rc::new(()),
            sources: BTreeMap::new(),
            bytes,
        })
    }

    #[cfg(unix)]
    #[test]
    fn publication_identity_reads_precede_the_final_freshness_guard() {
        use jaic::sema::{FileSystem, NativeFs};
        use std::cell::Cell;
        use std::path::Path;

        struct RetargetFs {
            alias: PathBuf,
            destination: PathBuf,
            calls: Cell<Option<usize>>,
        }
        impl FileSystem for RetargetFs {
            fn read(&self, path: &Path) -> Option<Vec<u8>> {
                NativeFs.read(path)
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
            fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
                NativeFs.canonical_exact(path)
            }
            fn is_native(&self) -> bool {
                true
            }
            fn canonical(&self, path: &Path) -> PathBuf {
                let result = NativeFs.canonical(path);
                if path == self.alias
                    && let Some(count) = self.calls.get()
                {
                    self.calls.set(Some(count + 1));
                    if count == 0 {
                        std::fs::remove_file(&self.alias).unwrap();
                        std::os::unix::fs::symlink(&self.destination, &self.alias).unwrap();
                    }
                }
                result
            }
        }
        let temp = test_support::TempDir::new("publication-identity");
        let root = temp.path().join("workspace/project/source");
        std::fs::create_dir_all(&root).unwrap();
        let first = root.join("first.jai");
        let second = root.join("second.jai");
        let alias = root.join("alias.jai");
        let source = "main :: () { value := 1; value += 2; }";
        std::fs::write(&first, source).unwrap();
        std::fs::write(&second, source).unwrap();
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let fs = Rc::new(RetargetFs {
            alias: alias.clone(),
            destination: second,
            calls: Cell::new(None),
        });
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                services: None,
                fs: fs.clone(),
                options: Box::new(|_, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options
                }),
            },
        );
        let uri = DocumentUri::from_path(&alias).unwrap();
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.diagnostics(&uri).unwrap();
        let previous = session
            .reference_records
            .borrow()
            .records
            .iter()
            .find(|r| r.uri == uri)
            .unwrap()
            .snapshot
            .clone();
        let answer = session.with_semantic(&uri, source, |_, _| {
            fs.calls.set(Some(0));
            Some(42)
        });
        assert!(
            fs.calls.get().unwrap() >= 2,
            "retarget actually happened at publication boundary"
        );
        assert_eq!(
            answer, None,
            "late contradictory identity observations must reject the answer"
        );
        let store = session.reference_records.borrow();
        assert!(
            Rc::ptr_eq(
                &previous,
                &store
                    .records
                    .iter()
                    .find(|r| r.uri == uri)
                    .unwrap()
                    .snapshot
            ),
            "failed publication preserves prior accepted facts"
        );
    }

    #[test]
    fn pressure_evicts_oldest_records_and_counts_shared_snapshots_once() {
        let shared = snapshot(1024);
        let mut store = Store {
            records: vec![
                record("file:///a.jai", shared.clone()),
                record("file:///b.jai", shared),
            ],
        };
        assert!(
            store.bytes() < 2048,
            "shared program storage counted only once"
        );
        store.records.push(record("file:///c.jai", snapshot(2048)));
        store.trim_to(2500);
        assert_eq!(store.records.len(), 1);
        assert_eq!(store.records[0].uri.as_str(), "file:///c.jai");
        store.trim_to(1);
        assert!(
            store.records.is_empty(),
            "oversized record is unavailable rather than breaking budget"
        );
    }

    #[test]
    fn record_pressure_falls_back_to_a_queued_check_without_compiling_in_the_request() {
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                services: None,
                fs: Rc::new(jaic::sema::NativeFs),
                options: Box::new(|_, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options
                }),
            },
        );
        let fixture = test_support::TempDir::new("pressure");
        let uri = DocumentUri::from_path(&fixture.join("main.jai")).unwrap();
        let source = "main :: () { local := 1; local += 2; }";
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.diagnostics(&uri).unwrap();
        session.defer_compiles(true);
        let position = Position {
            line: 0,
            character: source.find("local").unwrap() as u32 + 1,
        };
        assert_eq!(session.references(&uri, position, true).unwrap().len(), 2);
        assert!(session.reference_record_counts().1 > 0);
        session.reference_records.borrow_mut().trim_to(1);
        session.semantic.borrow_mut().forget_compiles();
        let counts = session.compile_counts();
        assert!(session.references(&uri, position, true).unwrap().is_empty());
        assert_eq!(session.compile_counts(), counts);
        assert!(
            session.compile_pending(),
            "unavailability schedules a fresh exact check"
        );
        assert_eq!(session.warm_pending(), Some(true));
        assert_eq!(session.references(&uri, position, true).unwrap().len(), 2);
    }

    fn query_session(source: &str, module: Option<&str>) -> (Session, DocumentUri) {
        let mut fs = jaic::sema::VirtualFs::default();
        if let Some(module) = module {
            fs.insert("/outside/Library/module.jai", module.as_bytes());
        }
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                services: None,
                fs: Rc::new(fs),
                options: Box::new(|_, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options.import_paths = vec!["/outside".into()];
                    options
                }),
            },
        );
        let uri = DocumentUri::parse("file:///query-facts/main.jai").unwrap();
        session.open(uri.clone(), 1, source.into()).unwrap();
        // Keep the entire overlay manifest unchanged during the later eviction workload.
        // Partial checks may conservatively depend on directory searches for missing types.
        for n in 0..8 {
            let other =
                DocumentUri::parse(&format!("file:///query-eviction/other{n}.jai")).unwrap();
            session
                .open(
                    other,
                    1,
                    format!("{}main :: () {{}}", "// padding\n".repeat(512)),
                )
                .unwrap();
        }
        session.defer_compiles(true);
        session.diagnostics(&uri).unwrap();
        (session, uri)
    }

    fn cursor(session: &Session, uri: &DocumentUri, offset: usize) -> Position {
        let doc = session.document(uri).unwrap();
        doc.index.position(&doc.text, offset).unwrap()
    }

    fn evict(session: &mut Session, uri: &DocumentUri) {
        for n in 0..8 {
            let other =
                DocumentUri::parse(&format!("file:///query-eviction/other{n}.jai")).unwrap();
            assert!(session.documents.contains_key(&other));
            session.diagnostics(&other).unwrap();
        }
        let text = session.document_text(uri).unwrap();
        assert!(
            session
                .with_cached_semantic(uri, text, |_, _| Some(()))
                .is_none(),
            "original compiler actually evicted"
        );
    }

    #[test]
    fn post_check_alias_completion_refreshes_targets_without_changing_fact_counts() {
        let source =
            "using,only(alias) Lib :: #import \"Library\";\nmain :: () { Lib.original(); }\n";
        let (mut session, uri) =
            query_session(source, Some("alias :: original; original :: () {}"));
        let alias = source.find("alias").unwrap();
        let state = |a: &mut Analysis, f: FileId| {
            let facts = a.compiler.ed.facts().unwrap();
            (
                (
                    facts.refs.len(),
                    facts.member_uses.len(),
                    a.compiler.ns.entities.len(),
                    a.compiler.pr.procs.len(),
                ),
                a.compiler.ide_references(f, alias as u32),
            )
        };
        let before = session.checked(&uri, state).unwrap();
        // Isolate the live per-compiler index defect from the independent retained-store bug.
        session.reference_records.borrow_mut().clear();
        let completion = session
            .completion(
                &uri,
                cursor(&session, &uri, source.find("Lib.original").unwrap()),
            )
            .unwrap();
        assert!(completion.items.iter().any(|item| item.label == "alias"));
        let after = session.checked(&uri, state).unwrap();
        assert_eq!(
            before.0, after.0,
            "fixture must mutate existing semantic state, not add facts"
        );
        assert!(
            after.1.len() > before.1.len(),
            "completion must resolve the imported alias target"
        );
        let at = cursor(&session, &uri, alias + 1);
        let expected = session.references(&uri, at, true).unwrap();
        assert_eq!(
            expected.len(),
            after.1.len(),
            "live reference index must reflect alias resolution"
        );
        evict(&mut session, &uri);
        let counts = session.compile_counts();
        assert_eq!(session.references(&uri, at, true).unwrap(), expected);
        assert_eq!(session.compile_counts(), counts);
    }

    #[test]
    fn post_check_definition_enrichment_is_retained_before_and_after_eviction() {
        let source =
            "helper :: () {}\nbroken :: (value: Missing_Type) { helper(); }\nmain :: () {}\n";
        let (mut session, uri) = query_session(source, None);
        let declaration = source.find("helper").unwrap();
        let call = source.rfind("helper").unwrap();
        let before = session
            .checked(&uri, |a, f| {
                a.compiler.ide_references(f, declaration as u32)
            })
            .unwrap();
        assert_eq!(
            before.len(),
            1,
            "failed body has not recorded the later call yet"
        );
        let definitions = session
            .definition(&uri, cursor(&session, &uri, call + 1))
            .unwrap();
        assert_eq!(definitions.len(), 1);
        let fresh = session
            .checked(&uri, |a, f| {
                a.compiler.ide_references(f, declaration as u32)
            })
            .unwrap();
        assert_eq!(
            fresh.len(),
            2,
            "definition query must add the missing callee fact"
        );
        let at = cursor(&session, &uri, declaration + 1);
        let expected = session.references(&uri, at, true).unwrap();
        assert_eq!(
            expected.len(),
            fresh.len(),
            "retained results must not hide newer compiler facts"
        );
        evict(&mut session, &uri);
        let counts = session.compile_counts();
        assert_eq!(session.references(&uri, at, true).unwrap(), expected);
        assert_eq!(session.compile_counts(), counts);
    }

    #[test]
    fn alternate_target_queries_cannot_replace_primary_reference_records() {
        let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                services: None,
                fs: Rc::new(jaic::sema::NativeFs),
                options: Box::new(move |fs, _| {
                    let mut options = jaic::sema::Options::host();
                    options.os = jaic::target::TargetOs::Linux;
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options.import_paths = vec![stdlib.clone()];
                    options.set_stdlib(fs, &stdlib);
                    options
                }),
            },
        );
        let fixture = test_support::TempDir::new("primary-records");
        let uri = DocumentUri::from_path(&fixture.join("main.jai")).unwrap();
        let source = "#import \"Basic\";\n#if OS == .WINDOWS { Windows_Only :: struct { value: int; } }\nhelper :: () {}\nmain :: () { #if OS == .LINUX { helper(); } #if OS == .WINDOWS { marker: Windows_Only; } }\n";
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.diagnostics(&uri).unwrap();
        let at = cursor(&session, &uri, source.find("helper").unwrap() + 1);
        let expected = session.references(&uri, at, true).unwrap();
        assert_eq!(expected.len(), 2);
        let compiled = session.compile_counts().0;
        let byte = source.rfind("Windows_Only").unwrap();
        assert!(
            session
                .with_semantic_at(&uri, source, byte, crate::semantic::Analysis::hover)
                .is_some()
        );
        assert!(
            session.compile_counts().0 > compiled,
            "query must actually compile an alternate target"
        );
        session.defer_compiles(true);
        // Target isolation is independent of which compiler the cost policy protects.
        // Other tests exercise eviction through competing checked programs.
        session.semantic.borrow_mut().forget_compiles();
        assert!(
            session
                .with_cached_semantic(&uri, source, |_, _| Some(()))
                .is_none()
        );
        let compiled = session.compile_counts().0;
        assert_eq!(session.references(&uri, at, true).unwrap(), expected);
        assert_eq!(session.compile_counts().0, compiled);
    }

    #[test]
    fn a_late_semantic_file_read_observes_current_unrelated_overlay_bytes() {
        let dir = test_support::TempDir::new("late-query-read");
        let project = dir.path().join("project");
        let modules = dir.path().join("modules");
        let module = modules.join("Library");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&module).unwrap();
        let data = dir.path().join("data.txt");
        std::fs::write(&data, "disk").unwrap();
        let data = data.canonicalize().unwrap();
        std::fs::write(module.join("module.jai"), format!("#import \"Basic\"; #import \"File\"; VALUE :: #run read_entire_file(\"{}\"); original :: () {{}}", jacket_config::slashed(&data))).unwrap();
        let source = "using,only(VALUE) Lib :: #import \"Library\"; main :: () { Lib.original(); }";
        let main = project.join("main.jai");
        std::fs::write(&main, source).unwrap();
        let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                services: None,
                fs: Rc::new(jaic::sema::NativeFs),
                options: Box::new(move |fs, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options.import_paths = vec![modules.clone(), stdlib.clone()];
                    options.set_stdlib(fs, &stdlib);
                    options
                }),
            },
        );
        let uri = DocumentUri::from_path(&main).unwrap();
        let data_uri = DocumentUri::from_path(&data).unwrap();
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.open(data_uri.clone(), 1, "one".into()).unwrap();
        assert!(
            session
                .prepare_rename(
                    &uri,
                    cursor(&session, &uri, source.find("main ::").unwrap() + 1)
                )
                .unwrap()
                .is_some()
        );
        let read = session
            .with_cached_semantic(&uri, source, |a, _| Some(a.reads.borrow().concerns(&data)))
            .flatten()
            .unwrap();
        assert!(!read, "the unused constant must not have read its data yet");
        session
            .change(
                &data_uri,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: "two".into(),
                }],
            )
            .unwrap();
        let bytes = session
            .with_cached_semantic(&uri, source, |a, _| a.compiler.src.fs.read(&data))
            .flatten()
            .expect("late read");
        assert_eq!(
            bytes, b"two",
            "first newly relevant compiler read must use current overlay bytes"
        );
        let large = dir.path().join("large-data.txt");
        let payload = vec![b'x'; BUDGET + 1];
        std::fs::write(&large, &payload).unwrap();
        let observed = session
            .with_cached_semantic(&uri, source, |a, _| {
                let bytes = a.compiler.src.fs.read(&large)?;
                assert_eq!(bytes.len(), payload.len());
                Some(a.retained_cost())
            })
            .flatten()
            .unwrap();
        assert!(
            observed > payload.len(),
            "external proof is charged to compiler retention cost"
        );
        assert_eq!(
            session.reference_record_counts(),
            (0, 0),
            "oversized external proof cannot hide outside the compact-record budget"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_overlay_added_through_reuse_keeps_its_identity_proof() {
        let temp = test_support::TempDir::new("reused-overlay-alias");
        let root = temp.path().join("main.jai");
        let first = temp.path().join("first.txt");
        let second = temp.path().join("second.txt");
        let alias = temp.path().join("alias.txt");
        std::fs::write(&first, "first disk").unwrap();
        std::fs::write(&second, "second disk").unwrap();
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let env = crate::Environment {
            services: None,
            fs: Rc::new(jaic::sema::NativeFs),
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        };
        let mut cache = crate::semantic::Cache::default();
        let mut files = BTreeMap::from([(root.clone(), Arc::<[u8]>::from(&b"main :: () {}"[..]))]);
        cache
            .analyze(&env, &root, &[], Rc::new(files.clone()))
            .unwrap();
        let compiled = cache.counts().0;
        files.insert(alias.clone(), Arc::from(&b"overlay"[..]));
        let files = Rc::new(files);
        let reused = cache.cached(&env, &root, &[], &files).unwrap();
        assert_eq!(reused.compiler.src.fs.read(&first).unwrap(), b"overlay");
        assert_eq!(
            cache.counts().0,
            compiled,
            "new unrelated overlay must enter through reuse"
        );
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(&second, &alias).unwrap();
        assert!(
            cache.cached(&env, &root, &[], &files).is_none(),
            "silent retarget must retire the stale accepted overlay binding"
        );
    }

    #[test]
    fn an_older_context_cannot_replace_a_newer_record_with_changed_dependencies() {
        let source =
            "using,only(alias) Lib :: #import \"Library\";\nmain :: () { Lib.original(); }\n";
        let (mut session, uri) =
            query_session(source, Some("alias :: original; original :: () {}"));
        let module = DocumentUri::parse("file:///outside/Library/module.jai").unwrap();
        session
            .open(
                module.clone(),
                1,
                "alias :: original; original :: () {}".into(),
            )
            .unwrap();
        session.diagnostics(&uri).unwrap();
        let (files, root, dirs) = session.compile_inputs(&uri, source).unwrap();
        let mut old_cache = crate::semantic::Cache::default();
        let old = old_cache
            .analyze(session.environment.as_ref().unwrap(), &root, &dirs, files)
            .unwrap();
        session.publish_reference_records(session.capture_reference_records(old));
        let old_owner = session
            .reference_records
            .borrow()
            .records
            .iter()
            .find(|r| r.uri == uri)
            .unwrap()
            .snapshot
            .owner
            .clone();
        session
            .change(
                &module,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: "alias :: original; original :: () { extra := 1; extra += 2; }".into(),
                }],
            )
            .unwrap();
        session.diagnostics(&uri).unwrap();
        let newest = session
            .reference_records
            .borrow()
            .records
            .iter()
            .find(|r| r.uri == uri)
            .unwrap()
            .snapshot
            .clone();
        assert!(
            !Rc::ptr_eq(&old_owner, &newest.owner),
            "dependency edit must produce another accepted context"
        );
        session.publish_reference_records(session.capture_reference_records(old));
        let store = session.reference_records.borrow();
        let after = store.records.iter().find(|r| r.uri == uri).unwrap();
        assert!(
            Rc::ptr_eq(&newest, &after.snapshot),
            "same main text does not authorize replacing facts from newer dependencies"
        );
    }

    #[test]
    fn enrichment_of_an_older_compile_survives_eviction_and_undo() {
        let source =
            "using,only(alias) Lib :: #import \"Library\";\nmain :: () { Lib.original(); }\n";
        let (mut session, uri) =
            query_session(source, Some("alias :: original; original :: () {}"));
        let alias = source.find("alias").unwrap();
        let at = cursor(&session, &uri, alias + 1);
        assert_eq!(session.references(&uri, at, true).unwrap().len(), 1);
        let edited = format!("{source}// typing\n");
        session
            .change(
                &uri,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: edited.clone(),
                }],
            )
            .unwrap();
        let completion = session
            .completion(
                &uri,
                cursor(&session, &uri, source.find("Lib.original").unwrap()),
            )
            .unwrap();
        assert!(completion.items.iter().any(|item| item.label == "alias"));
        let fresh = session
            .on_older_compile(&uri, &edited, |a, path, _| {
                let file = a.file(path)?;
                Some(a.compiler.ide_references(file, alias as u32))
            })
            .unwrap();
        assert_eq!(fresh.len(), 2);
        for n in 0..8 {
            let other =
                DocumentUri::parse(&format!("file:///query-eviction/other{n}.jai")).unwrap();
            session.diagnostics(&other).unwrap();
        }
        assert!(
            session
                .on_older_compile(&uri, &edited, |_, _, _| Some(()))
                .is_none(),
            "old compiler actually evicted"
        );
        session
            .change(
                &uri,
                3,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: source.into(),
                }],
            )
            .unwrap();
        let compiled = session.compile_counts().0;
        assert_eq!(
            session.references(&uri, at, true).unwrap().len(),
            fresh.len()
        );
        assert_eq!(session.compile_counts().0, compiled);
    }
}

#[cfg(test)]
#[path = "request_work_tests.rs"]
pub(crate) mod request_work_tests;
