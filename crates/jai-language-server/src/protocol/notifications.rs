//! Notification handlers, publications and configuration.
use super::JsonSession;
use super::helpers::{decode, domain, fail, file_document, folder_path, is_file_uri, uri};
use super::methods::{ErrorCode, Failure, Notification};
use super::params::{ChangeParams, OpenParams};
use super::wire::diagnostic_wire;
use crate::{Diagnostic, DocumentUri, Error};
use serde_json::{Value, json};
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
                self.did_change_workspace_folders(&params["event"]);
                Ok(vec![])
            }
            Notification::DidChangeWatchedFiles => {
                // Files changed on disk: 1 created, 2 changed, 3 deleted.
                for change in params["changes"].as_array().into_iter().flatten() {
                    if let Some(path) = folder_path(&change["uri"]) {
                        let changed = change["type"].as_u64() == Some(2);
                        self.session.saved_file_changed(&path, !changed);
                    }
                }
                Ok(self.refresh())
            }
        }
    }

    /// The publications after an edit, and a pull request when a settings file changed.
    pub(super) fn edited(&mut self, settings_touched: bool) -> Vec<Value> {
        let mut messages = self.publications();
        if settings_touched {
            messages.extend(self.refresh());
        }
        messages
    }

    fn did_open(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let OpenParams {
            text_document: item,
        } = decode(params)?;
        let settings = item
            .uri
            .ends_with(&format!("/{}", jacket_config::FILE_NAME));
        if !is_file_uri(&item.uri) {
            // Untitled buffers and other schemes have no path to analyse.
            return Ok(vec![]);
        }
        let jai_file = item.uri.ends_with(".jai") || item.language_id.eq_ignore_ascii_case("jai");
        if !jai_file && !settings {
            return Err(fail(
                ErrorCode::InvalidParams,
                "Only Jai documents are supported",
            ));
        }
        let size = item.text.len();
        let target = uri(&item.uri)?;
        if let Err(error) = self.session.open(target, item.version, item.text) {
            // Never drop a document without telling the user.
            if let Error::Limit(what) = &error {
                self.session.notify(format!(
                    "jailsp did not open {} ({:.1} MiB): {what}. Its diagnostics, hover and completion are off.",
                    item.uri,
                    size as f64 / (1024.0 * 1024.0)
                ));
            }
            return Err(domain(error));
        }
        Ok(self.edited(settings))
    }

    fn did_change(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let p: ChangeParams = decode(params)?;
        if !is_file_uri(&p.text_document.uri) {
            return Ok(vec![]);
        }
        let target = uri(&p.text_document.uri)?;
        let settings = crate::lints::is_config(&target);
        let changed = self
            .session
            .change(&target, p.text_document.version, &p.content_changes);
        if let Err(error) = changed {
            // A version the document is already past is a repeat; any other failure leaves the
            // client's text ahead of ours, so drop ours and wait for the file to be opened again.
            if matches!(error, Error::StaleVersion | Error::MissingDocument) {
                return Err(domain(error));
            }
            self.session.notify(format!(
                "jailsp could not apply an edit to {} ({error}) and closed it: its diagnostics, hover and completion are off until it is opened again.",
                p.text_document.uri
            ));
            let message = format!("textDocument/didChange failed: {error}");
            let _ = self.session.close(&target);
            let mut messages = vec![super::helpers::log_error(&message)];
            messages.extend(self.edited(settings));
            if !self.pull_diagnostics {
                messages.push(json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": target.as_str(), "diagnostics": [] },
                }));
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
                self.refresh()
            } else {
                Vec::new()
            });
        }
        let mut publications = self.publications();
        publications.push(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": closed.as_str(), "diagnostics": [] },
        }));
        Ok(publications)
    }

    fn did_change_configuration(&mut self, params: &Value) -> Vec<Value> {
        let settings = &params["settings"]["jai"];
        let trusted = self.session.trusted();
        let lenses = self.session.test_lenses_enabled();
        self.configure(settings);
        if let Some(on) = settings["trusted"].as_bool() {
            self.session.set_trusted(on);
        }
        // Trust decides what the compile may run, so every document's diagnostics change.
        let mut messages = if trusted == self.session.trusted() {
            Vec::new()
        } else {
            self.publications()
        };
        messages.extend(self.refresh());
        if lenses != self.session.test_lenses_enabled() {
            messages.extend(self.code_lens_refresh());
        }
        messages
    }

    fn did_change_workspace_folders(&mut self, event: &Value) {
        let folders = |key: &str| -> Vec<PathBuf> {
            event[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|f| folder_path(&f["uri"]))
                .collect()
        };
        let removed = folders("removed");
        let mut kept: Vec<PathBuf> = self
            .session
            .workspace_folders
            .iter()
            .filter(|f| !removed.contains(f))
            .cloned()
            .collect();
        kept.extend(folders("added"));
        self.session.set_workspace_folders(kept);
    }


    /// The version and diagnostics of `target` for a pull (none for a document that is not open).
    pub(super) fn pulled(&self, target: &DocumentUri) -> (Option<i32>, Vec<Diagnostic>) {
        (
            self.session.version(target).ok(),
            self.session.diagnostics(target).unwrap_or_default(),
        )
    }

    /// A request for the client to pull again, when it supports being asked: settings files
    /// changed the diagnostics of documents that did not.
    fn refresh(&mut self) -> Vec<Value> {
        if !self.refresh_diagnostics {
            return Vec::new();
        }
        self.refreshes += 1;
        vec![json!({
            "jsonrpc": "2.0",
            "id": format!("jai-diagnostic-refresh-{}", self.refreshes),
            "method": "workspace/diagnostic/refresh",
        })]
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
        self.session
            .publications()
            .into_iter()
            .map(|(uri, version, diagnostics)| {
                let diagnostics: Vec<Value> = diagnostics.iter().map(diagnostic_wire).collect();
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": uri, "version": version, "diagnostics": diagnostics },
                })
            })
            .collect()
    }

    /// Apply the settings in `settings` (`initializationOptions`, or the `jai` section of
    /// `workspace/didChangeConfiguration`): `{ "completion": { "autoImport": false,
    /// "callSnippets": "fillArguments" } }`.
    pub(super) fn configure(&mut self, settings: &Value) {
        if let Some(on) = settings["completion"]["autoImport"].as_bool() {
            self.session.set_auto_import(on);
        }
        if let Some(on) = settings["tests"]["codeLens"].as_bool() {
            self.session.set_test_lenses(on);
        }
        if let Some(lines) = settings["inlayHints"]["closingBraceMinLines"].as_u64() {
            self.session.set_closing_brace_min_lines(lines as usize);
        }
        if let Some(mode) = settings["completion"]["callSnippets"]
            .as_str()
            .and_then(crate::call_snippets::CallSnippets::parse)
        {
            self.session.set_call_snippets(mode);
        }
    }
}
