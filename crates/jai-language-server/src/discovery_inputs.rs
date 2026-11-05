//! Native project/configuration proof before semantic input memo reuse.
use crate::semantic::{Observations, ObservedFs};
use crate::session::Session;

pub(crate) struct Capture<'a>(&'a std::cell::Cell<usize>);

impl Drop for Capture<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

/// Nested semantic work shares one discovery transaction; a new request may start afresh.
pub(crate) struct Query {
    depth: std::rc::Rc<std::cell::Cell<usize>>,
    prefix: std::rc::Rc<std::cell::RefCell<crate::semantic::Observations>>,
    proof: std::rc::Rc<std::cell::RefCell<crate::semantic::Observations>>,
    memo_proof: std::rc::Rc<std::cell::RefCell<crate::semantic::Observations>>,
    semantic: std::rc::Rc<std::cell::RefCell<crate::semantic::Cache>>,
    index: std::rc::Rc<std::cell::RefCell<crate::auto_import::Index>>,
    inputs: std::rc::Rc<std::cell::RefCell<Option<crate::session::InputsMemo>>>,
    advisory: std::rc::Rc<std::cell::RefCell<Option<crate::session::CompileInputs>>>,
}

impl Drop for Query {
    fn drop(&mut self) {
        self.depth.set(self.depth.get() - 1);
        if self.depth.get() == 0 {
            let mut cache = self.semantic.borrow_mut();
            if cache.end_request() {
                let mut index = self.index.borrow_mut();
                index.builds = cache.recorded.clone();
                index.probed = cache.probe_statuses();
                index.advisory_builds = cache.advisory_builds();
                index.discard_input_memos();
                *self.inputs.borrow_mut() = None;
            }
            *self.proof.borrow_mut() = if self.inputs.borrow().is_some() {
                self.memo_proof.borrow().clone()
            } else {
                Observations::default()
            };
            *self.prefix.borrow_mut() = Observations::default();
            *self.advisory.borrow_mut() = None;
        }
    }
}

impl Session {
    pub(crate) fn discovery_query(&self) -> Query {
        if self.input_queries.get() == 0 {
            self.validate_discovery_inputs();
            *self.request_input_proof.borrow_mut() = Observations::default();
            *self.advisory_inputs.borrow_mut() = None;
        }
        self.input_queries.set(self.input_queries.get() + 1);
        Query {
            depth: self.input_queries.clone(),
            prefix: self.request_input_proof.clone(),
            proof: self.input_proof.clone(),
            memo_proof: self.memo_input_proof.clone(),
            semantic: self.semantic.clone(),
            index: self.index.clone(),
            inputs: self.inputs.clone(),
            advisory: self.advisory_inputs.clone(),
        }
    }

    pub(crate) fn capture_discovery_inputs(&self) -> Capture<'_> {
        self.capture_inputs.set(self.capture_inputs.get() + 1);
        Capture(&self.capture_inputs)
    }

    pub(crate) fn config_filesystem(&self) -> Option<ObservedFs<'_>> {
        Some(ObservedFs {
            base: &*self.environment.as_ref()?.fs,
            proof: Some(&self.input_proof),
            secondary: (self.input_queries.get() > 0).then_some(&self.request_input_proof),
        })
    }

    /// Changed first observations retire dependent discovery caches before a new request.
    pub(crate) fn validate_discovery_inputs(&self) {
        let Some(env) = &self.environment else {
            return;
        };
        if !env.fs.is_native() {
            return;
        }
        let changed = self.input_proof.borrow_mut().remove_invalid(&*env.fs);
        if changed.is_empty() {
            return;
        }
        *self.inputs.borrow_mut() = None;
        *self.open_file_identities.borrow_mut() = None;
        let mut index = self.index.borrow_mut();
        for path in changed {
            index.created_or_deleted(&path);
        }
        index.probed.clear();
    }

    pub(crate) fn discovery_inputs_current(&self) -> bool {
        self.environment
            .as_ref()
            .is_none_or(|env| !env.fs.is_native() || self.input_proof.borrow().matches(&*env.fs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn helper_cached_discovery_is_not_an_unobserved_semantic_input() {
        let temp = test_support::TempDir::new("unobserved-discovery-cache");
        let root = temp.path().join("workspace/project/source");
        std::fs::create_dir_all(&root).unwrap();
        let entry = root.join("entry.jai");
        let child = root.join("child.jai");
        std::fs::write(
            root.join("jacket.toml"),
            "[project]\nentry = [\"entry.jai\"]\n",
        )
        .unwrap();
        std::fs::write(&entry, "#load \"child.jai\"; supplied :: 42;").unwrap();
        let source = "main :: () { value: int = supplied; }";
        std::fs::write(&child, source).unwrap();
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
        session.set_workspace_folders(vec![root]);
        let uri = crate::DocumentUri::from_path(&child).unwrap();
        session.open(uri.clone(), 1, source.into()).unwrap();
        assert_eq!(
            session.project_root(&uri).unwrap(),
            test_support::lsp::native_identity(&entry)
        );
        assert_eq!(
            session.input_proof.borrow().retained_bytes(),
            0,
            "helper populated a cache outside input capture"
        );
        std::fs::write(&entry, "supplied :: 42;").unwrap();
        let (_, actual, _) = session.compile_inputs(&uri, source).unwrap();
        assert_eq!(
            actual,
            test_support::lsp::native_identity(&child),
            "semantic discovery needs its own captured first responses"
        );
    }

    #[test]
    fn silent_data_can_turn_a_negative_entry_into_a_child_producer_after_eviction() {
        let temp = test_support::TempDir::new("negative-build-proof");
        let root = temp.path().join("workspace/project/source");
        std::fs::create_dir_all(&root).unwrap();
        let flag = root.join("flag.txt");
        std::fs::write(&flag, "off").unwrap();
        let flag = test_support::lsp::native_identity(&flag);
        std::fs::write(
            root.join("jacket.toml"),
            "[project]\nentry = [\"build.jai\"]\n",
        )
        .unwrap();
        let build = format!(
            r#"#import "Basic"; #import "Compiler"; #import "File";
#run {{
    set_build_options_dc(.{{do_output=false}});
    flag := read_entire_file("{}");
    if flag == "on" {{
        w := compiler_create_workspace("child");
        options := get_build_options(w); options.output_type = .NO_OUTPUT; set_build_options(options, w);
        add_build_string("provided :: 42;", w);
        name := "child";
        add_build_file(tprint("%/%.jai", #filepath, name), w);
    }}
}}
"#,
            jacket_config::slashed(&flag)
        );
        std::fs::write(root.join("build.jai"), build).unwrap();
        let child = root.join("child.jai");
        let source = "main :: () { value: int = provided; }";
        std::fs::write(&child, source).unwrap();
        let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                services: None,
                fs: Rc::new(jaic::sema::NativeFs),
                options: Box::new(move |fs, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = true;
                    options.stack_trace = false;
                    options.import_paths = vec![stdlib.clone()];
                    options.set_stdlib(fs, &stdlib);
                    options
                }),
            },
        );
        session.set_workspace_folders(vec![root]);
        let uri = crate::DocumentUri::from_path(&child).unwrap();
        session.open(uri.clone(), 1, source.into()).unwrap();
        let errors = |session: &Session| {
            session
                .diagnostics(&uri)
                .unwrap()
                .iter()
                .filter(|d| d.severity == crate::DiagnosticSeverity::Error)
                .count()
        };
        assert!(
            errors(&session) > 0,
            "negative run leaves child standalone without provided"
        );
        assert_eq!(
            session
                .semantic
                .borrow()
                .probe_statuses()
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [crate::semantic::ProbeStatus::Complete],
            "producer really completed successfully without a child"
        );

        assert!(
            session.compile_counts().0 >= 2,
            "entry probe and standalone child both ran"
        );
        session.semantic.borrow_mut().forget_compiles();
        assert_eq!(
            session.compile_counts().1,
            0,
            "all full compilers actually evicted"
        );
        std::fs::write(&flag, "on").unwrap();
        assert_eq!(
            errors(&session),
            0,
            "silent external data must retire the negative discovery outcome"
        );
    }

    #[test]
    fn checked_positive_child_survives_unrelated_failure_but_absent_child_is_not_negative() {
        for missing_child in [false, true] {
            let temp = test_support::TempDir::new("partial-build-membership");
            let root = temp.path().join("workspace/project/source");
            std::fs::create_dir_all(&root).unwrap();
            let entry = root.join("build.jai");
            std::fs::write(
                root.join("jacket.toml"),
                "[project]\nentry = [\"build.jai\"]\n",
            )
            .unwrap();
            let build = r#"#import "Basic"; #import "Compiler";
#run {
    set_build_options_dc(.{do_output=false});
    good := compiler_create_workspace("good");
    options := get_build_options(good); options.output_type = .NO_OUTPUT; set_build_options(options,good);
    add_build_string("provided :: 42;", good);
    name := "good"; add_build_file(tprint("%/%.jai", #filepath,name),good);
    bad := compiler_create_workspace("bad");
    options = get_build_options(bad); options.output_type = .NO_OUTPUT; set_build_options(options,bad);
    name = "bad"; add_build_file(tprint("%/%.jai", #filepath,name),bad);
}
"#;
            std::fs::write(&entry, build).unwrap();
            let source = "main :: () { local := provided; local += 1; local += 2; }";
            let absent = "main :: () { local := 1; local += 2; }";
            std::fs::write(root.join("good.jai"), source).unwrap();
            if !missing_child {
                std::fs::write(root.join("bad.jai"), "main :: () { missing(); }").unwrap();
            }
            std::fs::write(root.join("absent.jai"), absent).unwrap();
            let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
            let mut session = Session::with_environment(
                crate::Limits::default(),
                crate::Environment {
                    services: None,
                    fs: Rc::new(jaic::sema::NativeFs),
                    options: Box::new(move |fs, _| {
                        let mut o = jaic::sema::Options::host();
                        o.import_paths = vec![stdlib.clone()];
                        o.set_stdlib(fs, &stdlib);
                        o
                    }),
                },
            );
            session.set_workspace_folders(vec![root.clone()]);
            let uri = crate::DocumentUri::from_path(&root.join("good.jai")).unwrap();
            let absent_uri = crate::DocumentUri::from_path(&root.join("absent.jai")).unwrap();
            session.open(uri.clone(), 1, source.into()).unwrap();
            session.open(absent_uri.clone(), 1, absent.into()).unwrap();
            let diagnostics = session.diagnostics(&uri);
            if missing_child {
                assert_eq!(
                    diagnostics.unwrap_err(),
                    crate::Error::DiagnosticsUnavailable
                );
                assert!(
                    session.semantic_coverage(&uri).unwrap().is_none(),
                    "an immediate add_build_file failure did not finish the producer"
                );
                assert!(
                    session
                        .semantic
                        .borrow()
                        .probe_statuses()
                        .values()
                        .all(|status| *status == crate::semantic::ProbeStatus::Failed)
                );
                continue;
            }
            assert!(
                !diagnostics
                    .unwrap()
                    .iter()
                    .any(|d| d.severity == crate::DiagnosticSeverity::Error)
            );
            let coverage = session
                .semantic_coverage(&uri)
                .unwrap()
                .expect("present checked child still has useful coverage");
            assert!(coverage.source_loaded && coverage.program_complete);
            assert_eq!(
                coverage.analysis.root.unwrap().path,
                test_support::lsp::native_identity(&entry)
            );
            assert_eq!(
                session
                    .semantic
                    .borrow()
                    .probe_statuses()
                    .values()
                    .copied()
                    .collect::<Vec<_>>(),
                [crate::semantic::ProbeStatus::Failed],
                "unrelated failed child is not a successful whole workspace"
            );
            let position = session
                .document(&uri)
                .unwrap()
                .index
                .position(source, source.find("local").unwrap() + 1)
                .unwrap();
            assert_eq!(session.references(&uri, position, true).unwrap().len(), 3);
            let absent_position = session
                .document(&absent_uri)
                .unwrap()
                .index
                .position(absent, absent.find("local").unwrap() + 1)
                .unwrap();
            let before = session.compile_counts();
            assert!(
                session
                    .references(&absent_uri, absent_position, true)
                    .unwrap()
                    .is_empty(),
                "failure cannot prove an unattributed file is standalone"
            );
            assert_eq!(
                session.compile_counts(),
                before,
                "no standalone compiler was accepted"
            );
            assert!(
                session
                    .prepare_rename(&absent_uri, absent_position)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn a_syntax_hash_candidate_cannot_reuse_another_document_revision() {
        let mut session = Session::new(crate::Limits::default());
        let uri = crate::DocumentUri::parse("file:///syntax-proof/main.jai").unwrap();
        session.open(uri.clone(), 1, "old :: 1;".into()).unwrap();
        Rc::make_mut(session.analyses.get_mut(&uri).unwrap()).key =
            crate::semantic::text_key("new :: 2;");
        session
            .change(
                &uri,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: "new :: 2;".into(),
                }],
            )
            .unwrap();
        assert_eq!(session.document_symbols(&uri).unwrap()[0].name, "new");
    }
}
