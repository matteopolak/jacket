//! Request handlers.
use super::helpers::{array, at, decode, document, domain, fail, internal, uri};
use super::methods::{ErrorCode, Failure, Request, ServerCommand};
use super::params::{
    CallHierarchyIncomingCallsParams, CallHierarchyOutgoingCallsParams, CodeActionParams,
    CommandParams, CommandTarget, DiagnosticParams, DocumentIdentifier, RangeParams,
    ReferenceParams, RenameParams, SelectionParams, TypeHierarchySubtypesParams,
    TypeHierarchySupertypesParams, WorkspaceDiagnosticParams, WorkspaceSymbolParams, call_item,
    type_item,
};
use super::wire::{
    code_action_wire, code_lens_wire, document_report, document_symbol_wire, edit_wire,
    expansion_wire, folding_wire, hierarchy_call_wire, hierarchy_item_wire, hover_kind, hover_wire,
    initialize_wire, inlay_wire, locations_wire, selection_wire, semantic_tokens_wire,
    signature_kind, signature_wire, standard_uri, standard_value, symbol_information_wire,
    token_delta_wire, workspace_document_report, workspace_edit_wire,
};
use super::{JsonSession, Lifecycle};
use crate::DocumentUri;
use crate::completion_wire::completion_wire;
use serde_json::{Value, json};
use std::path::PathBuf;

impl JsonSession {
    pub(super) fn request(&mut self, method: &str, params: Value) -> Result<Value, Failure> {
        let _inputs = self.session.discovery_query();
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
            Request::SemanticTokensRange => return self.semantic_tokens_range(params),
            Request::SemanticTokensDelta => return self.semantic_tokens_delta(params),
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
                return serde_json::to_value(completion_wire(list.map_err(domain)?))
                    .map_err(internal);
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
            Request::LinkedEditingRange => {
                let (document, position) = at(params)?;
                let ranges = self
                    .session
                    .linked_editing_ranges(&document, position)
                    .map_err(domain)?;
                if ranges.is_empty() {
                    Value::Null
                } else {
                    standard_value(gen_lsp_types::LinkedEditingRanges {
                        ranges,
                        word_pattern: Some("[A-Za-z_][A-Za-z0-9_]*".into()),
                    })
                }
            }
            Request::PrepareTypeHierarchy => {
                let (document, position) = at(params)?;
                let items = self.session.prepare_type_hierarchy(&document, position);
                match items.map_err(domain)? {
                    items if items.is_empty() => Value::Null,
                    items => array(&items, hierarchy_item_wire),
                }
            }
            Request::Supertypes => {
                let p: TypeHierarchySupertypesParams = decode(params)?;
                let items = self.session.supertypes(&type_item(p.item));
                array(&items.map_err(domain)?, hierarchy_item_wire)
            }
            Request::Subtypes => {
                let p: TypeHierarchySubtypesParams = decode(params)?;
                let items = self.session.subtypes(&type_item(p.item));
                array(&items.map_err(domain)?, hierarchy_item_wire)
            }
            Request::Declaration => {
                let (document, position) = at(params)?;
                let found = self.session.declaration(&document, position);
                locations_wire(&found.map_err(domain)?)
            }
            Request::Implementation => {
                let (document, position) = at(params)?;
                let found = self.session.implementation(&document, position);
                locations_wire(&found.map_err(domain)?)
            }
            Request::Formatting => {
                let edits = self.session.format(&document(params)?, None);
                array(&edits.map_err(domain)?, edit_wire)
            }
            Request::RangeFormatting => {
                let p: RangeParams = decode(params)?;
                let edits = self
                    .session
                    .format(&uri(p.text_document.uri.as_str())?, Some(p.range));
                array(&edits.map_err(domain)?, edit_wire)
            }
            Request::References => {
                let p: ReferenceParams = decode(params)?;
                let declaration = p.context.is_none_or(|c| c.include_declaration);
                let document = uri(p.text_document.uri.as_str())?;
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
                let at = p.text_document_position_params;
                let document = uri(at.text_document.uri.as_str())?;
                let edits = self.session.rename(&document, at.position, &p.new_name);
                edits
                    .map_err(domain)?
                    .map_or(Value::Null, workspace_edit_wire)
            }
            Request::DocumentHighlight => {
                let (document, position) = at(params)?;
                let ranges = self.session.document_highlights(&document, position);
                array(ranges.map_err(domain)?, |range| {
                    standard_value(gen_lsp_types::DocumentHighlight {
                        range,
                        kind: Some(gen_lsp_types::DocumentHighlightKind::Text),
                    })
                })
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
                array(links.map_err(domain)?, |(range, target)| {
                    standard_value(gen_lsp_types::DocumentLink {
                        range,
                        target: Some(standard_uri(&target)),
                        tooltip: None,
                        data: None,
                    })
                })
            }
            Request::FoldingRange => {
                let ranges = self.session.folding_ranges(&document(params)?);
                array(&ranges.map_err(domain)?, folding_wire)
            }
            Request::InlayHint => {
                let p: RangeParams = decode(params)?;
                let hints = self
                    .session
                    .inlay_hints(&uri(p.text_document.uri.as_str())?, p.range);
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
                let p: CallHierarchyIncomingCallsParams = decode(params)?;
                let calls = self.session.incoming_calls(&call_item(p.item));
                array(&calls.map_err(domain)?, |c| hierarchy_call_wire(c, "from"))
            }
            Request::OutgoingCalls => {
                let p: CallHierarchyOutgoingCallsParams = decode(params)?;
                let calls = self.session.outgoing_calls(&call_item(p.item));
                array(&calls.map_err(domain)?, |c| hierarchy_call_wire(c, "to"))
            }
            Request::SelectionRange => {
                let p: SelectionParams = decode(params)?;
                let ranges = self
                    .session
                    .selection_ranges(&uri(p.text_document.uri.as_str())?, &p.positions);
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
                let target = uri(p.text_document.uri.as_str())?;
                let (_, diagnostics) = self.pulled(&target)?;
                standard_value(document_report(
                    &diagnostics,
                    p.previous_result_id.as_deref(),
                ))
            }
            Request::WorkspaceDiagnostic => self.workspace_diagnostics(params)?,
            Request::Source => self.source(params)?,
        };
        Ok(result)
    }

    fn initialize(&mut self, params: Value) -> Result<Value, Failure> {
        if self.lifecycle != Lifecycle::New {
            return Err(fail(
                ErrorCode::InvalidRequest,
                "Server is already initialized",
            ));
        }
        let params: gen_lsp_types::InitializeParams = decode(params)?;
        let caps = &params.capabilities;
        let document = caps.text_document.as_ref();
        let workspace = caps.workspace.as_ref();
        self.lifecycle = Lifecycle::AwaitInitialized;
        self.hover_kind = hover_kind(caps);
        self.signature_kind = signature_kind(caps);
        self.pull_diagnostics = document.and_then(|d| d.diagnostic.as_ref()).is_some();
        self.refresh_diagnostics = self.pull_diagnostics
            && workspace
                .and_then(|w| w.diagnostics.as_ref())
                .and_then(|d| d.refresh_support)
                == Some(true);
        self.refresh_code_lenses = workspace
            .and_then(|w| w.code_lens.as_ref())
            .and_then(|c| c.refresh_support)
            == Some(true);
        self.refresh_inlay_hints = workspace
            .and_then(|w| w.inlay_hint.as_ref())
            .and_then(|c| c.refresh_support)
            == Some(true);
        self.refresh_semantic_tokens = workspace
            .and_then(|w| w.semantic_tokens.as_ref())
            .and_then(|c| c.refresh_support)
            == Some(true);
        self.session.set_snippet_support(
            document
                .and_then(|d| d.completion.as_ref())
                .and_then(|c| c.completion_item.as_ref())
                .and_then(|c| c.snippet_support)
                == Some(true),
        );
        let options = params.initialization_options.unwrap_or(Value::Null);
        self.configure(&options);
        // An untrusted server stays so unless the client explicitly grants trust later.
        if options["trusted"].as_bool() == Some(false) {
            self.session.set_trusted(false);
        }
        let path = |uri: &gen_lsp_types::Uri| {
            DocumentUri::parse(uri.as_str())
                .ok()
                .map(|uri| PathBuf::from(uri.path()))
        };
        #[expect(deprecated, reason = "rootUri remains supported for older clients")]
        let folders = match params.workspace_folders_initialize_params.workspace_folders {
            Some(gen_lsp_types::WorkspaceFolders::WorkspaceFolderList(folders)) => folders
                .iter()
                .filter_map(|folder| path(&folder.uri))
                .collect(),
            None | Some(gen_lsp_types::WorkspaceFolders::Null) => params
                .root_uri
                .as_ref()
                .and_then(path)
                .into_iter()
                .collect(),
        };
        self.session.set_workspace_folders(folders);
        Ok(initialize_wire(self.pull_diagnostics))
    }

    /// The whole document's tokens, remembered under a new result id for `delta` requests.
    fn semantic_tokens(&mut self, params: Value) -> Result<Value, Failure> {
        let document = document(params)?;
        let tokens = self.session.semantic_tokens(&document).map_err(domain)?;
        let data = semantic_tokens_wire(&tokens);
        drop(tokens);
        let id = self.remember_tokens(&document, &data);
        serde_json::to_value(&gen_lsp_types::SemanticTokens {
            result_id: Some(id.to_string()),
            data,
        })
        .map_err(internal)
    }

    /// The tokens that start inside a range, encoded from the start of the document.
    fn semantic_tokens_range(&mut self, params: Value) -> Result<Value, Failure> {
        let p: RangeParams = decode(params)?;
        let document = uri(p.text_document.uri.as_str())?;
        let tokens = self
            .session
            .semantic_tokens_in(&document, p.range)
            .map_err(domain)?;
        serde_json::to_value(&gen_lsp_types::SemanticTokens {
            result_id: None,
            data: semantic_tokens_wire(&tokens),
        })
        .map_err(internal)
    }

    /// What changed since the tokens `previousResultId` named, as one edit of the integer
    /// array; the whole array when that result is not the last one sent.
    fn semantic_tokens_delta(&mut self, params: Value) -> Result<Value, Failure> {
        let previous = params["previousResultId"].as_str().map(str::to_owned);
        let document = document(params)?;
        let tokens = self.session.semantic_tokens(&document).map_err(domain)?;
        let data = semantic_tokens_wire(&tokens);
        drop(tokens);
        let old = self.token_results.get(document.as_str()).cloned();
        let id = self.remember_tokens(&document, &data);
        let value = match old {
            Some((old_id, old_data))
                if previous.as_deref() == Some(old_id.to_string().as_str()) =>
            {
                token_delta_wire(id, &old_data, &data).map_err(internal)?
            }
            _ => serde_json::to_value(&gen_lsp_types::SemanticTokens {
                result_id: Some(id.to_string()),
                data,
            })
            .map_err(internal)?,
        };
        Ok(value)
    }

    fn remember_tokens(
        &mut self,
        document: &DocumentUri,
        data: &[gen_lsp_types::SemanticToken],
    ) -> u64 {
        self.token_result_ids += 1;
        // Closed documents are not forgotten one by one; the table is just kept small.
        if self.token_results.len() > 64 {
            self.token_results.clear();
        }
        self.token_results.insert(
            document.as_str().to_owned(),
            (self.token_result_ids, data.to_vec()),
        );
        self.token_result_ids
    }

    fn document_symbols(&mut self, params: Value) -> Result<Value, Failure> {
        let symbols = self
            .session
            .document_symbols(&document(params)?)
            .map_err(domain)?;
        let symbols: Vec<_> = symbols.into_iter().map(document_symbol_wire).collect();
        serde_json::to_value(symbols).map_err(internal)
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
            item["documentation"] = standard_value(gen_lsp_types::MarkupContent {
                kind: gen_lsp_types::MarkupKind::Markdown,
                value: doc,
            });
        }
        item
    }

    fn code_actions(&mut self, params: Value) -> Result<Value, Failure> {
        let p: CodeActionParams = decode(params)?;
        let context = p.context;
        let context = crate::lints::ActionContext {
            only: context.only.map(|kinds| {
                kinds
                    .into_iter()
                    .map(|kind| kind.as_str().to_owned())
                    .collect()
            }),
            diagnostics: context
                .diagnostics
                .into_iter()
                .filter(|d| matches!(d.source.as_deref(), Some("jailint" | "jaic")))
                .filter_map(|d| match d.code? {
                    gen_lsp_types::Code::String(code) => Some((code, d.range)),
                    gen_lsp_types::Code::Int(_) => None,
                })
                .collect(),
        };
        let actions =
            self.session
                .code_actions_in(&uri(p.text_document.uri.as_str())?, p.range, &context);
        Ok(array(&actions.map_err(domain)?, code_action_wire))
    }

    fn execute_command(&mut self, params: Value) -> Result<Value, Failure> {
        let p: CommandParams = decode(params)?;
        let targets: Vec<CommandTarget> = p
            .arguments
            .unwrap_or_default()
            .into_iter()
            .map(decode)
            .collect::<Result<_, _>>()?;
        let Some(target) = targets.first() else {
            return Err(fail(
                ErrorCode::InvalidParams,
                "The command needs a {uri, position} argument",
            ));
        };
        let document = uri(&target.uri)?;
        match ServerCommand::parse(&p.command) {
            Some(ServerCommand::Expansion) => {
                let expansion = self.session.expansion(&document, target.position);
                Ok(expansion
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, expansion_wire))
            }
            Some(ServerCommand::Polymorphs) => {
                Ok(json!(self.session.polymorphs(&document, target.position)))
            }
            Some(ServerCommand::CompileTimeCosts) => Ok(Value::Array(
                self.session
                    .compile_time_costs(&document)
                    .iter()
                    .map(crate::compile_costs::CostRow::wire)
                    .collect(),
            )),
            None => Err(fail(ErrorCode::InvalidParams, "Unknown command")),
        }
    }

    fn workspace_diagnostics(&mut self, params: Value) -> Result<Value, Failure> {
        let p: WorkspaceDiagnosticParams = decode(params)?;
        let items = self
            .session
            .diagnostic_reports()
            .map_err(domain)?
            .into_iter()
            .map(|(target, version, diagnostics)| {
                let previous = p
                    .previous_result_ids
                    .iter()
                    .find(|r| r.uri.as_str() == target)
                    .map(|r| r.value.as_str());
                workspace_document_report(&target, Some(version), &diagnostics, previous)
            })
            .collect();
        Ok(standard_value(gen_lsp_types::WorkspaceDiagnosticReport {
            items,
        }))
    }

    /// The text of a definition's file the client has not opened (a module or stdlib file), so a
    /// browser editor can show it read-only; or of a `jai-expansion:` document.
    fn source(&mut self, params: Value) -> Result<Value, Failure> {
        let p: DocumentIdentifier = decode(params)?;
        let text = if p
            .uri
            .as_str()
            .starts_with(crate::features::EXPANSION_SCHEME)
        {
            self.session.expansion_source(p.uri.as_str())
        } else {
            self.session.source(&uri(p.uri.as_str())?)
        };
        Ok(text.map_or(Value::Null, Value::String))
    }
}
