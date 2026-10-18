//! Request handlers.
use super::helpers::{array, at, decode, document, domain, fail, folder_path, internal, uri};
use super::methods::{ErrorCode, Failure, Reply, Request, ServerCommand};
use super::params::{
    CodeActionParams, CommandParams, DiagnosticParams, DocumentIdentifier, HierarchyParams,
    RangeParams, ReferenceParams, RenameParams, SelectionParams, WorkspaceDiagnosticParams,
    WorkspaceSymbolParams,
};
use super::wire::{
    SymbolsWire, capabilities, code_action_wire, code_lens_wire, document_report, expansion_wire,
    folding_wire, hierarchy_call_wire, hierarchy_item_wire, hover_kind, hover_wire, inlay_wire,
    locations_wire, selection_wire, semantic_tokens_wire, signature_kind, signature_wire,
    symbol_information_wire, workspace_edit_wire,
};
use super::{JsonSession, Lifecycle};
use crate::completion_wire::completion_wire;
use serde_json::{Value, json};
use std::path::PathBuf;

impl JsonSession {
    pub(super) fn request(&mut self, method: &str, params: Value) -> Result<Reply, Failure> {
        let request = Request::parse(method);
        if request != Some(Request::Initialize) {
            self.ensure_running()?;
        }
        let request = request
            .filter(|r| self.pull_diagnostics || !r.pulls_diagnostics())
            .ok_or_else(|| fail(ErrorCode::MethodNotFound, "Method not supported"))?;
        let result = match request {
            Request::Initialize => self.initialize(params)?,
            Request::Shutdown => {
                self.lifecycle = Lifecycle::Shutdown;
                Value::Null
            }
            Request::SemanticTokens => return self.semantic_tokens(params),
            Request::DocumentSymbol => return self.document_symbols(params),
            Request::Hover => {
                let (document, position) = at(params)?;
                let hover = self.session.hover_as(&document, position, self.hover_kind);
                hover
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, hover_wire)
            }
            Request::Completion => {
                let (document, position) = at(params)?;
                let list = self.session.completion(&document, position);
                completion_wire(&list.map_err(domain)?)
            }
            Request::CompletionResolve => self.resolve_completion(params),
            Request::Definition => {
                let (document, position) = at(params)?;
                locations_wire(
                    &self
                        .session
                        .definition(&document, position)
                        .map_err(domain)?,
                )
            }
            Request::TypeDefinition => {
                let (document, position) = at(params)?;
                let found = self.session.type_definition(&document, position);
                locations_wire(&found.map_err(domain)?)
            }
            Request::References => {
                let p: ReferenceParams = decode(params)?;
                let declaration = p.context.is_none_or(|c| c.include_declaration);
                let document = uri(&p.text_document.uri)?;
                let found = self.session.references(&document, p.position, declaration);
                locations_wire(&found.map_err(domain)?)
            }
            Request::PrepareRename => {
                let (document, position) = at(params)?;
                let range = self.session.prepare_rename(&document, position);
                range
                    .map_err(domain)?
                    .map_or(Value::Null, |range| json!(range))
            }
            Request::Rename => {
                let p: RenameParams = decode(params)?;
                let document = uri(&p.text_document.uri)?;
                let edits = self.session.rename(&document, p.position, &p.new_name);
                edits
                    .map_err(domain)?
                    .map_or(Value::Null, workspace_edit_wire)
            }
            Request::DocumentHighlight => {
                let (document, position) = at(params)?;
                let ranges = self.session.document_highlights(&document, position);
                array(
                    ranges.map_err(domain)?,
                    |range| json!({ "range": range, "kind": 1 }),
                )
            }
            Request::WorkspaceSymbol => {
                let p: WorkspaceSymbolParams = decode(params)?;
                array(
                    &self.session.workspace_symbols(&p.query),
                    symbol_information_wire,
                )
            }
            Request::DocumentLink => {
                let links = self.session.document_links(&document(params)?);
                array(
                    links.map_err(domain)?,
                    |(range, target)| json!({ "range": range, "target": target }),
                )
            }
            Request::FoldingRange => {
                let ranges = self.session.folding_ranges(&document(params)?);
                array(&ranges.map_err(domain)?, folding_wire)
            }
            Request::InlayHint => {
                let p: RangeParams = decode(params)?;
                let hints = self
                    .session
                    .inlay_hints(&uri(&p.text_document.uri)?, p.range);
                array(&hints.map_err(domain)?, inlay_wire)
            }
            Request::SignatureHelp => {
                let (document, position) = at(params)?;
                let kind = self.signature_kind;
                let help = self.session.signature_help_as(&document, position, kind);
                help.map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, |help| signature_wire(help, kind))
            }
            Request::CodeAction => self.code_actions(params)?,
            Request::PrepareCallHierarchy => {
                let (document, position) = at(params)?;
                let items = self.session.prepare_call_hierarchy(&document, position);
                match items.map_err(domain)? {
                    items if items.is_empty() => Value::Null,
                    items => array(&items, hierarchy_item_wire),
                }
            }
            Request::IncomingCalls => {
                let p: HierarchyParams = decode(params)?;
                let calls = self.session.incoming_calls(&p.item.into_item());
                array(&calls.map_err(domain)?, |c| hierarchy_call_wire(c, "from"))
            }
            Request::OutgoingCalls => {
                let p: HierarchyParams = decode(params)?;
                let calls = self.session.outgoing_calls(&p.item.into_item());
                array(&calls.map_err(domain)?, |c| hierarchy_call_wire(c, "to"))
            }
            Request::SelectionRange => {
                let p: SelectionParams = decode(params)?;
                let ranges = self
                    .session
                    .selection_ranges(&uri(&p.text_document.uri)?, &p.positions);
                array(&ranges.map_err(domain)?, selection_wire)
            }
            Request::CodeLens => {
                let lenses = self.session.code_lenses(&document(params)?);
                array(&lenses.map_err(domain)?, code_lens_wire)
            }
            Request::ExecuteCommand => self.execute_command(params)?,
            Request::Expansion => {
                let (document, position) = at(params)?;
                let expansion = self.session.expansion(&document, position);
                expansion
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, expansion_wire)
            }
            Request::DocumentDiagnostic => {
                let p: DiagnosticParams = decode(params)?;
                let target = uri(&p.text_document.uri)?;
                let (version, diagnostics) = self.pulled(&target);
                document_report(None, version, &diagnostics, p.previous_result_id.as_deref())
            }
            Request::WorkspaceDiagnostic => self.workspace_diagnostics(params)?,
            Request::Source => self.source(params)?,
        };
        Ok(Reply::Value(result))
    }

    fn initialize(&mut self, params: Value) -> Result<Value, Failure> {
        if self.lifecycle != Lifecycle::New {
            return Err(fail(
                ErrorCode::InvalidRequest,
                "Server is already initialized",
            ));
        }
        if !params.is_object() && !params.is_null() {
            return Err(fail(
                ErrorCode::InvalidParams,
                "Initialize parameters must be an object",
            ));
        }
        self.lifecycle = Lifecycle::Running;
        self.hover_kind = hover_kind(&params);
        self.signature_kind = signature_kind(&params);
        self.pull_diagnostics = params
            .pointer("/capabilities/textDocument/diagnostic")
            .is_some_and(Value::is_object);
        self.refresh_diagnostics = self.pull_diagnostics
            && params
                .pointer("/capabilities/workspace/diagnostics/refreshSupport")
                .and_then(Value::as_bool)
                == Some(true);
        self.session.set_snippet_support(
            params
                .pointer("/capabilities/textDocument/completion/completionItem/snippetSupport")
                .and_then(Value::as_bool)
                == Some(true),
        );
        self.configure(&params["initializationOptions"]);
        // A server started untrusted (`--untrusted`) stays so unless the client grants
        // trust later; initialization options can only withhold it.
        if params["initializationOptions"]["trusted"].as_bool() == Some(false) {
            self.session.set_trusted(false);
        }
        let folders: Vec<PathBuf> = match params["workspaceFolders"].as_array() {
            Some(folders) => folders
                .iter()
                .filter_map(|f| folder_path(&f["uri"]))
                .collect(),
            None => folder_path(&params["rootUri"]).into_iter().collect(),
        };
        self.session.set_workspace_folders(folders);
        Ok(json!({
            "capabilities": capabilities(self.pull_diagnostics),
            "serverInfo": {
                "name": "jai-language-server",
                "version": env!("CARGO_PKG_VERSION"),
            },
        }))
    }

    fn semantic_tokens(&mut self, params: Value) -> Result<Reply, Failure> {
        let document = document(params)?;
        let version = self.session.version(&document).map_err(domain)?;
        let tokens = self.session.semantic_tokens(&document).map_err(domain)?;
        let data = semantic_tokens_wire(&tokens);
        drop(tokens);
        let data = serde_json::to_string(&data).map_err(internal)?;
        Ok(Reply::Raw(format!(
            "{{\"resultId\":\"{version}\",\"data\":{data}}}"
        )))
    }

    fn document_symbols(&mut self, params: Value) -> Result<Reply, Failure> {
        let symbols = self
            .session
            .document_symbols(&document(params)?)
            .map_err(domain)?;
        let text = serde_json::to_string(&SymbolsWire(&symbols)).map_err(internal)?;
        Ok(Reply::Raw(text))
    }

    /// The documentation of an item whose list left it out.
    fn resolve_completion(&self, mut item: Value) -> Value {
        let data = item.get("data").and_then(|d| {
            Some(crate::CompletionData {
                document: d.get("document")?.as_str()?.to_owned(),
                path: d.get("path")?.as_str()?.to_owned(),
                offset: d.get("offset")?.as_u64()? as usize,
            })
        });
        if let Some(doc) = data.and_then(|d| self.session.completion_documentation(&d)) {
            item["documentation"] = json!({ "kind": "markdown", "value": doc });
        }
        item
    }

    fn code_actions(&mut self, params: Value) -> Result<Value, Failure> {
        let p: CodeActionParams = decode(params)?;
        let context = p.context.unwrap_or_default();
        let context = crate::lints::ActionContext {
            only: context.only,
            diagnostics: context
                .diagnostics
                .into_iter()
                .filter(|d| matches!(d.source.as_deref(), Some("jailint" | "jaic")))
                .filter_map(|d| Some((d.code?.as_str()?.to_string(), d.range)))
                .collect(),
        };
        let actions = self
            .session
            .code_actions_in(&uri(&p.text_document.uri)?, p.range, &context);
        Ok(array(&actions.map_err(domain)?, code_action_wire))
    }

    fn execute_command(&mut self, params: Value) -> Result<Value, Failure> {
        let p: CommandParams = decode(params)?;
        let Some(target) = p.arguments.first() else {
            return Err(fail(
                ErrorCode::InvalidParams,
                "The command needs a {uri, position} argument",
            ));
        };
        let document = uri(&target.uri)?;
        match ServerCommand::parse(&p.command) {
            Some(ServerCommand::ShowExpansion) => {
                let expansion = self.session.expansion(&document, target.position);
                Ok(expansion
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, expansion_wire))
            }
            Some(ServerCommand::ShowPolymorphs) => {
                Ok(json!(self.session.polymorphs(&document, target.position)))
            }
            None => Err(fail(ErrorCode::InvalidParams, "Unknown command")),
        }
    }

    fn workspace_diagnostics(&mut self, params: Value) -> Result<Value, Failure> {
        let p: WorkspaceDiagnosticParams = decode(params)?;
        let items: Vec<Value> = self
            .session
            .publications()
            .into_iter()
            .map(|(target, version, diagnostics)| {
                let previous = p
                    .previous_result_ids
                    .iter()
                    .find(|r| r.uri == target)
                    .map(|r| r.value.as_str());
                document_report(Some(&target), Some(version), &diagnostics, previous)
            })
            .collect();
        Ok(json!({ "items": items }))
    }

    /// The text of a definition's file the client has not opened (a module or stdlib file), so a
    /// browser editor can show it read-only; or of a `jai-expansion:` document.
    fn source(&mut self, params: Value) -> Result<Value, Failure> {
        let p: DocumentIdentifier = decode(params)?;
        let text = if p.uri.starts_with(crate::features::EXPANSION_SCHEME) {
            self.session.expansion_source(&p.uri)
        } else {
            self.session.source(&uri(&p.uri)?)
        };
        Ok(text.map_or(Value::Null, Value::String))
    }
}
