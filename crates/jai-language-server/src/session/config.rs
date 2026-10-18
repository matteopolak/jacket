//! Session settings, workspace folders and source snapshots.
use super::Session;
use crate::semantic::{self, Environment};
use crate::{DocumentUri, Error, Limits, VirtualSources};
use std::path::{Path, PathBuf};

impl Session {
    /// Turn auto-import completion on or off.
    pub fn set_auto_import(&mut self, on: bool) {
        self.auto_import = on;
    }

    /// Choose what accepting a procedure completion inserts.
    pub fn set_call_snippets(&mut self, mode: crate::call_snippets::CallSnippets) {
        self.call_snippets = mode;
    }

    /// Say whether the client renders snippet completion items; call parentheses and
    /// statement skeletons are only inserted when it does.
    pub fn set_snippet_support(&mut self, on: bool) {
        self.snippet_support = on;
    }

    /// The client's workspace folders (absolute paths).
    /// Messages for the user the server has found since the last call (a settings file that
    /// does not parse), each to be sent once.
    pub fn take_logs(&self) -> Vec<String> {
        self.index.borrow_mut().take_logs()
    }

    /// Messages for the user (see `take_logs` for the log lines).
    pub fn take_messages(&self) -> Vec<String> {
        self.index.borrow_mut().take_messages()
    }

    /// How many programs the type checker has compiled for this session, and how many of them
    /// are kept (for tests and the benchmark: an edit that needs no compile leaves the first
    /// number alone, and the second stays bounded).
    pub fn compile_counts(&self) -> (usize, usize) {
        self.semantic.borrow().counts()
    }

    /// Coverage for the current open buffer and its real analysis context. A lack
    /// of file-local diagnostics is not proof that the compiler reached the file.
    pub fn semantic_coverage(
        &self,
        uri: &DocumentUri,
    ) -> Result<Option<semantic::SemanticCoverage>, Error> {
        let doc = self.document(uri)?;
        let probe = &doc.text;
        let base = self.with_semantic(uri, probe, |analysis, path| Some(analysis.coverage(path)));
        if base.as_ref().is_some_and(|coverage| coverage.source_loaded) {
            return Ok(base);
        }
        Ok(self
            .with_semantic_at(uri, probe, 0, |analysis, path| {
                let coverage = analysis.coverage(path);
                coverage.source_loaded.then_some(coverage)
            })
            .or(base))
    }

    /// Queue a message for the user (`window/showMessage`), sent once.
    pub fn notify(&self, message: String) {
        self.index.borrow_mut().notify(message);
    }

    /// Drop everything cached (compiles, scans) after a failure: it is rebuilt on demand.
    pub fn reset_caches(&self) {
        // What the metaprograms did is not lost with the compiles that recorded it.
        let trusted = self.semantic.borrow().trusted();
        let recorded = std::mem::take(&mut self.semantic.borrow_mut().recorded);
        let version = self.semantic.borrow().recorded_version;
        *self.semantic.borrow_mut() = Default::default();
        {
            let mut cache = self.semantic.borrow_mut();
            cache.set_trusted(trusted);
            cache.recorded = recorded;
            cache.recorded_version = version + 1;
        }
        *self.index.borrow_mut() = Default::default();
        self.synced_builds.set(0);
        self.touch();
    }

    /// Allow or withhold running the workspace's compile-time code (`#run`, build
    /// metaprograms, computed `#insert`). Trusted by default. A change drops the compiles made
    /// under the other setting; the caller republishes diagnostics.
    pub fn set_trusted(&mut self, trusted: bool) {
        if self.semantic.borrow().trusted() != trusted {
            self.semantic.borrow_mut().set_trusted(trusted);
            // Without trust nothing runs, so discovery falls back to the static scan; what
            // earlier runs recorded goes with the cache.
            let mut index = self.index.borrow_mut();
            index.builds.clear();
            index.probed.clear();
            index.import_dirs.clear();
            self.synced_builds.set(0);
            self.touch();
        }
    }

    pub fn trusted(&self) -> bool {
        self.semantic.borrow().trusted()
    }

    /// The inputs of a compile may have changed.
    fn touch(&self) {
        self.generation.set(self.generation.get() + 1);
        *self.inputs.borrow_mut() = None;
        *self.open_file_identities.borrow_mut() = None;
    }

    pub fn set_workspace_folders(&mut self, folders: Vec<PathBuf>) {
        self.workspace_folders = folders
            .iter()
            .map(|p| crate::document::path_key(p))
            .collect();
        self.index.borrow_mut().import_dirs.clear();
        self.touch();
    }

    /// A file changed on disk (`created_or_deleted` when it appeared or went away): what was
    /// read from it is read again.
    pub fn file_changed(&mut self, path: &Path, created_or_deleted: bool) {
        self.touch();
        if created_or_deleted {
            self.semantic.borrow_mut().forget_paths();
        }
        let path = self.environment.as_ref().map_or_else(
            || crate::document::path_key(path),
            |env| crate::auto_import::canonical_key(env, path),
        );
        let path = path.as_path();
        let mut index = self.index.borrow_mut();
        if created_or_deleted {
            index.created_or_deleted(path);
        } else {
            index.changed(path);
        }
    }

    /// A session that also type-checks the open documents against `environment`'s modules.
    pub fn with_environment(limits: Limits, environment: Environment) -> Self {
        Self {
            environment: Some(environment),
            ..Self::new(limits)
        }
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    pub fn source_snapshot(&self) -> VirtualSources {
        VirtualSources {
            files: self
                .documents
                .iter()
                .map(|(uri, doc)| (uri.path().to_owned(), doc.text.clone()))
                .collect(),
        }
    }

    pub fn document_text(&self, uri: &DocumentUri) -> Result<&str, Error> {
        Ok(self.document(uri)?.text.as_str())
    }

    pub fn version(&self, uri: &DocumentUri) -> Result<i32, Error> {
        Ok(self.document(uri)?.version)
    }
}
