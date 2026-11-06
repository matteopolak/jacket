//! Notification handlers, publications and configuration.
use super::JsonSession;
use super::helpers::{decode, domain, fail, file_document, is_file_uri, uri};
use super::methods::{ErrorCode, Failure, Notification};
use super::params::{OpenParams, change_params};
use super::wire::diagnostics_notification;
use crate::{Diagnostic, DocumentUri, Error};
use serde_json::Value;
use std::path::PathBuf;

impl JsonSession {
    pub(super) fn notification(
        &mut self,
        notification: Option<Notification>,
        params: Value,
    ) -> Result<Vec<Value>, Failure> {
        self.ensure_running()?;
        // Unknown notifications are ignored as required by JSON-RPC/LSP.
        let Some(notification) = notification else {
            return Ok(vec![]);
        };
        match notification {
            // `$/cancelRequest` and `exit` are handled before the lifecycle is checked.
            Notification::Initialized | Notification::Cancel | Notification::Exit => Ok(vec![]),
            Notification::DidOpen => self.did_open(params),
            Notification::DidChange => self.did_change(params),
            Notification::DidSave => {
                let Some(target) = file_document(params)? else {
                    return Ok(vec![]);
                };
                self.session.save(&target).map_err(domain)?;
                Ok(self.edited(crate::lints::is_config(&target)))
            }
            Notification::DidClose => self.did_close(params),
            Notification::DidChangeConfiguration => Ok(self.did_change_configuration(&params)),
            Notification::SetTrusted => {
                let Some(on) = params["trusted"].as_bool() else {
                    return Err(fail(
                        ErrorCode::InvalidParams,
                        "jai/setTrusted needs a boolean `trusted`",
                    ));
                };
                self.session.set_trusted(on);
                Ok(self.edited(true))
            }
            Notification::DidChangeWorkspaceFolders => {
                let params: gen_lsp_types::DidChangeWorkspaceFoldersParams = decode(params)?;
                self.did_change_workspace_folders(params.event);
                Ok(vec![])
            }
            Notification::DidChangeWatchedFiles => {
                let params: gen_lsp_types::DidChangeWatchedFilesParams = decode(params)?;
                for change in params.changes {
                    if let Ok(uri) = DocumentUri::parse(change.uri.as_str()) {
                        let changed = change.kind == gen_lsp_types::FileChangeType::Changed;
                        self.session
                            .saved_file_changed(std::path::Path::new(uri.path()), !changed);
                    }
                }
                Ok(self.edited(true))
            }
        }
    }

    /// The publications after an edit, and a pull request when a settings file changed.
    pub(super) fn edited(&mut self, settings_touched: bool) -> Vec<Value> {
        let mut messages = self.publications();
        if settings_touched {
            messages.extend(self.diagnostic_refresh());
        }
        messages
    }

    fn did_open(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let OpenParams {
            text_document: item,
        } = decode(params)?;
        let settings = item
            .uri
            .as_str()
            .ends_with(&format!("/{}", jacket_config::FILE_NAME));
        if !is_file_uri(item.uri.as_str()) {
            // Untitled buffers and other schemes have no path to analyse.
            return Ok(vec![]);
        }
        let jai_file = item.uri.as_str().ends_with(".jai")
            || item.language_id.as_str().eq_ignore_ascii_case("jai");
        if !jai_file && !settings {
            return Err(fail(
                ErrorCode::InvalidParams,
                "Only Jai documents are supported",
            ));
        }
        let size = item.text.len();
        let target = uri(item.uri.as_str())?;
        if let Err(error) = self.session.open(target, item.version, item.text) {
            // Never drop a document without telling the user.
            if let Error::Limit(what) = &error {
                self.session.notify(format!(
                    "jailsp did not open {} ({:.1} MiB): {what}. Its diagnostics, hover and completion are off.",
                    item.uri.as_str(),
                    size as f64 / (1024.0 * 1024.0)
                ));
            }
            return Err(domain(error));
        }
        Ok(self.edited(settings))
    }

    fn did_change(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let (document, changes) = change_params(params)?;
        let document_uri = document.text_document_identifier.uri;
        if !is_file_uri(document_uri.as_str()) {
            return Ok(vec![]);
        }
        let target = uri(document_uri.as_str())?;
        let settings = crate::lints::is_config(&target);
        let changed = self.session.change(&target, document.version, &changes);
        if let Err(error) = changed {
            // A version the document is already past is a repeat; any other failure leaves the
            // client's text ahead of ours, so drop ours and wait for the file to be opened again.
            if matches!(error, Error::StaleVersion | Error::MissingDocument) {
                return Err(domain(error));
            }
            self.session.notify(format!(
                "jailsp could not apply an edit to {} ({error}) and closed it: its diagnostics, hover and completion are off until it is opened again.",
                document_uri.as_str()
            ));
            let message = format!("textDocument/didChange failed: {error}");
            let _ = self.session.close(&target);
            let mut messages = vec![super::helpers::log_error(&message)];
            messages.extend(self.edited(settings));
            if !self.pull_diagnostics {
                messages.push(diagnostics_notification(target.as_str(), None, &[]));
            }
            return Ok(messages);
        }
        if settings {
            return Ok(vec![]);
        }
        Ok(self.edited(false))
    }

    fn did_close(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let Some(closed) = file_document(params)? else {
            return Ok(vec![]);
        };
        self.session.close(&closed).map_err(domain)?;
        if self.pull_diagnostics {
            return Ok(if crate::lints::is_config(&closed) {
                self.diagnostic_refresh()
            } else {
                Vec::new()
            });
        }
        let mut publications = self.publications();
        publications.push(diagnostics_notification(closed.as_str(), None, &[]));
        Ok(publications)
    }

    fn did_change_configuration(&mut self, params: &Value) -> Vec<Value> {
        let settings = &params["settings"]["jai"];
        let trusted = self.session.trusted();
        let lenses = self.session.test_lenses_enabled();
        let limits = self.session.check_limits;
        self.configure(settings);
        if let Some(on) = settings["trusted"].as_bool() {
            self.session.set_trusted(on);
        }
        // Trust decides what the compile may run, so every document's diagnostics change.
        let mut messages =
            if trusted == self.session.trusted() && limits == self.session.check_limits {
                Vec::new()
            } else {
                self.publications()
            };
        messages.extend(self.diagnostic_refresh());
        if lenses != self.session.test_lenses_enabled() {
            messages.extend(self.code_lens_refresh());
        }
        messages
    }

    fn did_change_workspace_folders(&mut self, event: gen_lsp_types::WorkspaceFoldersChangeEvent) {
        let folders = |folders: Vec<gen_lsp_types::WorkspaceFolder>| -> Vec<PathBuf> {
            folders
                .into_iter()
                .filter_map(|folder| DocumentUri::parse(folder.uri.as_str()).ok())
                .map(|uri| PathBuf::from(uri.path()))
                .collect()
        };
        let removed = folders(event.removed);
        let mut kept: Vec<PathBuf> = self
            .session
            .workspace_folders
            .iter()
            .filter(|f| !removed.contains(f))
            .cloned()
            .collect();
        kept.extend(folders(event.added));
        self.session.set_workspace_folders(kept);
    }


    /// The version and diagnostics of `target` for a pull (none for a document that is not open).
    pub(super) fn pulled(
        &self,
        target: &DocumentUri,
    ) -> Result<(Option<i32>, Vec<Diagnostic>), Failure> {
        let Ok(version) = self.session.version(target) else {
            // A closed document retains the existing explicit empty-report contract.
            return Ok((None, Vec::new()));
        };
        Ok((
            Some(version),
            self.session.diagnostics(target).map_err(domain)?,
        ))
    }

    /// A request for the client to pull again, when it supports being asked: settings files
    /// changed the diagnostics of documents that did not.
    pub(super) fn diagnostic_refresh(&mut self) -> Vec<Value> {
        if !self.refresh_diagnostics {
            return Vec::new();
        }
        self.refresh_request(gen_lsp_types::LspRequestMethod::WorkspaceDiagnosticRefresh.as_str())
            .into_iter()
            .collect()
    }

    fn publications(&mut self) -> Vec<Value> {
        if self.pull_diagnostics {
            return Vec::new();
        }
        if self.defer_publications {
            self.publications_pending = true;
            return Vec::new();
        }
        self.publications_now()
    }

    pub(super) fn publications_now(&self) -> Vec<Value> {
        Self::publication_messages(self.session.publications())
    }

    pub(super) fn publication_messages(
        publications: Vec<(String, i32, Vec<crate::model::Diagnostic>)>,
    ) -> Vec<Value> {
        publications
            .into_iter()
            .map(|(uri, version, diagnostics)| {
                diagnostics_notification(&uri, Some(version), &diagnostics)
            })
            .collect()
    }

    /// Apply the settings in `settings` (`initializationOptions`, or the `jai` section of
    /// `workspace/didChangeConfiguration`): `{ "completion": { "autoImport": false,
    /// "callSnippets": "fillArguments" }, "formatter": { "path": "/opt/jacket/jaifmt" }, "compileTime": { "evaluationBudget": 100000000,
    /// "checkTimeLimit": 10 } }`. A setting the client leaves out keeps its default.
    pub(super) fn configure(&mut self, settings: &Value) {
        if let Some(on) = settings["completion"]["autoImport"].as_bool() {
            self.session.set_auto_import(on);
        }
        if let Some(on) = settings["tests"]["codeLens"].as_bool() {
            self.session.set_test_lenses(on);
        }
        if let Some(path) = settings["formatter"]["path"].as_str() {
            let path = (!path.is_empty()).then(|| PathBuf::from(path));
            self.session.set_formatter_path(path);
        }
        if let Some(lines) = settings["inlayHints"]["closingBraceMinLines"].as_u64() {
            self.session.set_closing_brace_min_lines(lines as usize);
        }
        let compile_time = &settings["compileTime"];
        if let Some(blocks) = compile_time["evaluationBudget"].as_u64() {
            self.session.set_evaluation_budget(blocks);
        }
        if let Some(seconds) = compile_time["checkTimeLimit"].as_f64() {
            self.session.set_check_time_limit(seconds);
        }
        if let Some(blocks) = compile_time["costLensBlocks"].as_u64() {
            self.session.set_cost_lens_blocks(blocks);
        }
        if let Some(on) = compile_time["limitNotices"].as_bool() {
            self.session.set_limit_notices(on);
        }
        if let Some(mode) = settings["completion"]["callSnippets"]
            .as_str()
            .and_then(crate::call_snippets::CallSnippets::parse)
        {
            self.session.set_call_snippets(mode);
        }
    }
}
