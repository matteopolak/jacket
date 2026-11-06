//! Constructing a session and routing incoming messages.
use super::helpers::{fail, log_error};
use super::methods::{ErrorCode, Failure, Notification, OUTPUT_LIMIT, debug_panic};
use super::params::CancelParams;
use super::{JsonSession, Lifecycle, ProtocolError, RequestId};
use crate::{Error, Limits, MarkupKind, Session};
use lsp_server::{Message, Response};
use serde_json::{Value, json};

impl JsonSession {
    pub fn new(limits: Limits) -> Self {
        Self::with_session(Session::new(limits))
    }

    /// A session that type-checks the open documents (see `Session::with_environment`).
    pub fn with_environment(limits: Limits, environment: crate::Environment) -> Self {
        Self::with_session(Session::with_environment(limits, environment))
    }

    fn with_session(session: Session) -> Self {
        Self {
            session,
            lifecycle: Lifecycle::New,
            requests: lsp_server::ReqQueue::default(),
            cancelled: std::collections::BTreeMap::new(),
            refresh_pending: std::collections::BTreeSet::new(),
            refresh_dirty: std::collections::BTreeSet::new(),
            hover_kind: MarkupKind::PlainText,
            signature_kind: MarkupKind::PlainText,
            pull_diagnostics: false,
            defer_publications: false,
            publications_pending: false,
            refresh_diagnostics: false,
            refresh_code_lenses: false,
            refresh_inlay_hints: false,
            refresh_semantic_tokens: false,
            lenses_changed: false,
            token_results: std::collections::BTreeMap::new(),
            token_result_ids: 0,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn exit_status(&self) -> Option<u8> {
        if let Lifecycle::Exited(code) = self.lifecycle {
            Some(code)
        } else {
            None
        }
    }

    /// Native hosts wait a bounded interval for exit after a successful shutdown response.
    pub fn shutdown_requested(&self) -> bool {
        self.lifecycle == Lifecycle::Shutdown
    }

    /// Hold back `publishDiagnostics` after `didOpen`, `didChange` and the like until
    /// `flush_publications`, so a transport can answer requests that are already waiting and
    /// publish once typing pauses (see `DIAGNOSTICS_DEBOUNCE` in `main.rs`). Off by default:
    /// `handle_json` then returns the publications with each notification's answer.
    pub fn defer_publications(&mut self, on: bool) {
        self.defer_publications = on;
    }

    /// Withhold (`false`) or allow running the workspace's compile-time code; see
    /// `Session::set_trusted`. A server for an untrusted workspace calls this before serving.
    pub fn set_trusted(&mut self, trusted: bool) {
        self.session.set_trusted(trusted);
    }

    /// (compiles so far, compiles alive); see `Session::compile_counts`.
    pub fn compile_counts(&self) -> (usize, usize) {
        self.session.compile_counts()
    }

    /// Work that makes later requests faster, for a transport to do while it waits for input
    /// (see `Session::warm_auto_import`).
    pub fn warm_up(&self) {
        if self.lifecycle == Lifecycle::Running {
            self.session.warm_auto_import();
        }
    }

    /// False when input interrupted the scan: retry on the next idle turn.
    pub fn warm_up_unless_interrupted(&self) -> bool {
        self.lifecycle != Lifecycle::Running || self.session.warm_auto_import_unless_interrupted()
    }

    /// Answer completion without compiling to rank names by expected type; a transport then
    /// calls `warm_pending` once input is quiet (see `Session::defer_compiles`).
    pub fn defer_compiles(&mut self, on: bool) {
        self.session.defer_compiles(on);
    }

    /// A completion went unranked; `warm_pending` has a compile to do for the next.
    pub fn compile_pending(&self) -> bool {
        self.lifecycle == Lifecycle::Running && self.session.compile_pending()
    }

    /// The compile a request lacked (one compile, stopped by newer input), then the requests
    /// that ask a client to fetch again what was answered without it.
    pub fn warm_pending(&mut self) -> Result<Vec<String>, ProtocolError> {
        let messages = self.warm_pending_messages()?;
        self.serialize_messages(messages)
    }

    pub fn warm_pending_messages(&mut self) -> Result<Vec<Message>, ProtocolError> {
        if self.lifecycle != Lifecycle::Running {
            return Ok(Vec::new());
        }
        let messages = match self.session.warm_pending() {
            Some(true) => self.compile_refreshes(),
            _ => Vec::new(),
        };
        self.encode_unprompted(messages)
    }

    /// Requests for the client to fetch inlay hints and semantic tokens again, for the ones it
    /// supports: what was answered while edits were typed lacked the type checker.
    pub(super) fn compile_refreshes(&mut self) -> Vec<Value> {
        let mut out = Vec::new();
        for (wanted, method) in [
            (
                self.refresh_inlay_hints,
                gen_lsp_types::LspRequestMethod::WorkspaceInlayHintRefresh.as_str(),
            ),
            (
                self.refresh_semantic_tokens,
                gen_lsp_types::LspRequestMethod::WorkspaceSemanticTokensRefresh.as_str(),
            ),
        ] {
            if wanted && let Some(request) = self.refresh_request(method) {
                out.push(request);
            }
        }
        out
    }

    /// Whether active coverage should receive the next bounded owner service opportunity.
    /// Completed, unavailable and shutdown epochs keep the ordinary polling cadence.
    pub fn input_epoch_hydration_pending(&self) -> bool {
        self.lifecycle == Lifecycle::Running && self.session.input_epoch_hydration_pending()
    }

    /// Reconciliation stays separate from finite semantic background work.
    pub fn has_input_epoch_work(&self) -> bool {
        self.lifecycle == Lifecycle::Running && self.session.has_input_epoch_work()
    }

    /// Service one native reconciliation slice without compiling diagnostics inline.
    /// A changed input invalidates its epoch before scheduling client updates.
    pub fn reconcile_input_epoch_messages(&mut self) -> Result<Vec<Message>, ProtocolError> {
        if self.lifecycle != Lifecycle::Running || !self.session.reconcile_input_epoch() {
            return Ok(Vec::new());
        }
        self.publications_pending |= !self.pull_diagnostics;
        let mut messages = self.compile_refreshes();
        messages.extend(self.diagnostic_refresh());
        messages.extend(self.code_lens_refresh());
        self.encode_unprompted(messages)
    }

    /// Whether `background_work` has something to do.
    pub fn has_background_work(&self) -> bool {
        self.lifecycle == Lifecycle::Running && self.session.has_background_work()
    }

    /// One piece of work a request left for later (see `Session::background_work`), for a
    /// transport to do while no input waits. Once none is left, a client that supports it is
    /// asked to fetch the code lenses it changed again.
    pub fn background_work(&mut self) -> Result<Vec<String>, ProtocolError> {
        let messages = self.background_work_messages()?;
        self.serialize_messages(messages)
    }

    pub fn background_work_messages(&mut self) -> Result<Vec<Message>, ProtocolError> {
        if self.lifecycle != Lifecycle::Running {
            return Ok(Vec::new());
        }
        let changed = self.guarded("background work", |this| Ok(this.session.background_work()));
        let mut messages = match changed {
            Ok(changed) => {
                self.lenses_changed |= changed;
                Vec::new()
            }
            Err(error) => vec![log_error(&error.message)],
        };
        if self.lenses_changed && !self.session.has_background_work() {
            self.lenses_changed = false;
            messages.extend(self.code_lens_refresh());
        }
        self.encode_unprompted(messages)
    }

    /// The request to fetch the code lenses again, when the client supports being asked.
    pub(super) fn code_lens_refresh(&mut self) -> Option<Value> {
        if !self.refresh_code_lenses {
            return None;
        }
        self.refresh_request(gen_lsp_types::LspRequestMethod::WorkspaceCodeLensRefresh.as_str())
    }

    pub(super) fn refresh_request(&mut self, method: &str) -> Option<Value> {
        if !self.refresh_pending.insert(method.to_owned()) {
            self.refresh_dirty.insert(method.to_owned());
            return None;
        }
        let request =
            self.requests
                .outgoing
                .register(method.to_owned(), Value::Null, method.to_owned());
        Some(serde_json::to_value(request).expect("standard request serializes"))
    }

    /// A compile-time service answer is pending: poll `service_arrivals` now and then.
    pub fn services_waiting(&self) -> bool {
        self.lifecycle == Lifecycle::Running && self.session.services_waiting()
    }

    /// Checks the open files again when pending service answers came in (publishing, or asking
    /// a pulling client to pull).
    pub fn service_arrivals(&mut self) -> Result<Vec<String>, ProtocolError> {
        let messages = self.service_arrivals_messages()?;
        self.serialize_messages(messages)
    }

    pub fn service_arrivals_messages(&mut self) -> Result<Vec<Message>, ProtocolError> {
        if self.lifecycle != Lifecycle::Running {
            return Ok(Vec::new());
        }
        if !self.session.take_service_arrivals() {
            return Ok(Vec::new());
        }
        let messages = self.edited(true);
        self.encode_unprompted(messages)
    }

    /// An edit is waiting for `flush_publications`.
    pub fn publications_pending(&self) -> bool {
        self.lifecycle == Lifecycle::Running && self.publications_pending
    }

    /// The diagnostics of every open document as they are now (one compile at most), for the
    /// edits since the last flush. Empty for a client that pulls.
    pub fn flush_publications(&mut self) -> Result<Vec<String>, ProtocolError> {
        let messages = self.flush_publications_messages()?;
        self.serialize_messages(messages)
    }

    pub fn flush_publications_messages(&mut self) -> Result<Vec<Message>, ProtocolError> {
        if self.lifecycle != Lifecycle::Running {
            return Ok(Vec::new());
        }
        if !std::mem::take(&mut self.publications_pending) {
            return Ok(Vec::new());
        }
        let messages = self.publications_now();
        self.encode_unprompted(messages)
    }

    /// Like `flush_publications`, but a check that is stopped because input arrived (the
    /// transport calls `jaic::interrupt::request(true)` when it reads some) publishes nothing
    /// and leaves the edits pending, so the transport answers what came in and flushes again
    /// once input is quiet.
    pub fn flush_publications_unless_interrupted(&mut self) -> Result<Vec<String>, ProtocolError> {
        let messages = self.flush_publications_unless_interrupted_messages()?;
        self.serialize_messages(messages)
    }

    pub fn flush_publications_unless_interrupted_messages(
        &mut self,
    ) -> Result<Vec<Message>, ProtocolError> {
        if self.lifecycle != Lifecycle::Running {
            return Ok(Vec::new());
        }
        if !self.publications_pending {
            return Ok(Vec::new());
        }
        let Some(publications) = self.session.publications_unless_interrupted() else {
            return Ok(Vec::new());
        };
        self.publications_pending = false;
        let mut messages = Self::publication_messages(publications);
        messages.extend(self.compile_refreshes());
        self.encode_unprompted(messages)
    }

    /// Browser bridge: decode one bounded JSON body and serialize standard versioned messages.
    pub fn handle_json(&mut self, input: &str) -> Result<Vec<String>, ProtocolError> {
        if input.len() > self.session.limits().message_bytes {
            return Err(ProtocolError::MessageLimit);
        }
        match super::message::decode(input) {
            Ok(Some(message)) => {
                let messages = self.handle_message(message)?;
                self.serialize_messages(messages)
            }
            Ok(None) => Ok(Vec::new()),
            Err(rejected) => self.serialize_rejection(rejected),
        }
    }

    /// Dispatch the library's owned message directly, without a native JSON round trip.
    pub fn handle_message(&mut self, message: Message) -> Result<Vec<Message>, ProtocolError> {
        if self.exit_status().is_some() {
            return Ok(Vec::new());
        }
        match message {
            Message::Request(request) => {
                if !self.requests.incoming.is_completed(&request.id) {
                    return self.checked_messages(vec![
                        Response {
                            id: request.id,
                            response_result: Err(fail(
                                ErrorCode::InvalidRequest,
                                "Duplicate pending request id",
                            )),
                        }
                        .into(),
                    ]);
                }
                self.requests.incoming.register(request.id.clone(), ());
                self.handle_request(request)
            }
            Message::Notification(notification) => {
                self.handle_notification(&notification.method, notification.params)
            }
            Message::Response(response) => {
                if let Some(method) = self.requests.outgoing.complete(response.id) {
                    self.refresh_pending.remove(&method);
                    if self.refresh_dirty.remove(&method)
                        && self.lifecycle == Lifecycle::Running
                        && let Some(request) = self.refresh_request(&method)
                    {
                        return self.encode_unprompted(vec![request]);
                    }
                }
                Ok(Vec::new())
            }
        }
    }

    /// Register requests before cancellation lookahead, then emit each answer immediately.
    /// The caller bounds input count/bytes; native stdio admits at most64 messages per batch.
    pub fn dispatch_messages(
        &mut self,
        messages: Vec<lsp_server::Message>,
        emit: impl FnMut(Vec<lsp_server::Message>) -> Result<(), ProtocolError>,
    ) -> Result<(), ProtocolError> {
        self.dispatch_messages_with_metadata(messages, None, emit)
    }

    /// Dispatch native frames with their transport-only timing identities.
    /// The sidecar never participates in request IDs, admission or protocol payloads.
    pub fn dispatch_messages_traced(
        &mut self,
        messages: Vec<lsp_server::Message>,
        trace: &crate::trace::Trace,
        frames: Vec<crate::trace::Frame>,
        emit: impl FnMut(Vec<lsp_server::Message>) -> Result<(), ProtocolError>,
    ) -> Result<(), ProtocolError> {
        if frames.len() != messages.len() {
            return Err(ProtocolError::Serialization(
                "LSP trace metadata mismatch".into(),
            ));
        }
        self.dispatch_messages_with_metadata(messages, Some((trace, frames)), emit)
    }

    fn dispatch_messages_with_metadata(
        &mut self,
        messages: Vec<lsp_server::Message>,
        frames: Option<(&crate::trace::Trace, Vec<crate::trace::Frame>)>,
        mut emit: impl FnMut(Vec<lsp_server::Message>) -> Result<(), ProtocolError>,
    ) -> Result<(), ProtocolError> {
        if messages.len() > 64 {
            return Err(ProtocolError::MessageLimit);
        }
        let mut batch = PendingBatch {
            session: self,
            registered: std::collections::BTreeSet::new(),
        };
        let mut duplicate = std::collections::BTreeSet::new();
        for (at, message) in messages.iter().enumerate() {
            match message {
                Message::Request(request) => {
                    if !batch.registered.contains(&request.id)
                        && batch.session.requests.incoming.is_completed(&request.id)
                    {
                        batch
                            .session
                            .requests
                            .incoming
                            .register(request.id.clone(), ());
                        batch.registered.insert(request.id.clone());
                    } else {
                        duplicate.insert(at);
                    }
                }
                Message::Notification(notification)
                    if notification.method
                        == gen_lsp_types::LspNotificationMethod::Exit.as_str() =>
                {
                    break;
                }
                Message::Notification(notification)
                    if notification.method
                        == gen_lsp_types::LspNotificationMethod::CancelRequest.as_str() =>
                {
                    if let Ok(params) =
                        serde_json::from_value::<CancelParams>(notification.params.clone())
                    {
                        batch
                            .session
                            .cancel_request(params.id)
                            .expect("standard cancellation id");
                    }
                }
                _ => {}
            }
        }
        for (at, message) in messages.into_iter().enumerate() {
            let _dispatch = frames
                .as_ref()
                .and_then(|(trace, frames)| crate::trace::dispatch_on(trace, frames[at]));
            let output = if let Message::Request(request) = message {
                if duplicate.contains(&at) {
                    batch.session.checked_messages(vec![
                        Response {
                            id: request.id,
                            response_result: Err(fail(
                                ErrorCode::InvalidRequest,
                                "Duplicate pending request id",
                            )),
                        }
                        .into(),
                    ])?
                } else {
                    batch.session.handle_request(request)?
                }
            } else if let Message::Notification(notification) = &message
                && notification.method
                    == gen_lsp_types::LspNotificationMethod::CancelRequest.as_str()
                && serde_json::from_value::<CancelParams>(notification.params.clone()).is_ok()
            {
                // Admission already applied this arrival in order. Applying it again here
                // could cancel a request that arrived later in the same batch.
                Vec::new()
            } else {
                batch.session.handle_message(message)?
            };
            emit(output)?;
            if batch.session.exit_status().is_some() {
                break;
            }
        }
        Ok(())
    }

    /// Dispatch normal messages together while retaining empty ignored-response output slots.
    fn dispatch_batch_slots(
        &mut self,
        pending: Vec<Message>,
        slots: Vec<bool>,
        output: &mut Vec<Vec<String>>,
        bytes: &mut usize,
        limit: usize,
    ) -> Result<(), ProtocolError> {
        let mut slots = slots.into_iter().peekable();
        self.dispatch_messages(pending, |messages| {
            while slots.peek() == Some(&false) {
                slots.next();
                append_batch_text(output, bytes, limit, Vec::new())?;
            }
            debug_assert_eq!(slots.next(), Some(true));
            append_batch_output(output, bytes, limit, messages)
        })?;
        if self.exit_status().is_none() {
            for normal in slots {
                debug_assert!(!normal);
                append_batch_text(output, bytes, limit, Vec::new())?;
            }
        }
        Ok(())
    }

    pub fn handle_batch(&mut self, inputs: &[String]) -> Result<Vec<Vec<String>>, ProtocolError> {
        if inputs.len() > 64
            || inputs
                .iter()
                .try_fold(0usize, |sum, text| sum.checked_add(text.len()))
                .is_none_or(|sum| sum > self.session.limits().message_bytes.saturating_mul(2))
        {
            return Err(ProtocolError::MessageLimit);
        }
        let mut output = Vec::new();
        let mut pending = Vec::new();
        let mut slots = Vec::new();
        let mut bytes = 0usize;
        let limit = self.session.limits().output_bytes;
        for input in inputs {
            if input.len() > self.session.limits().message_bytes {
                return Err(ProtocolError::MessageLimit);
            }
            match super::message::decode(input) {
                Ok(Some(message)) => {
                    pending.push(message);
                    slots.push(true);
                }
                Ok(None) => slots.push(false),
                Err(error) => {
                    self.dispatch_batch_slots(
                        std::mem::take(&mut pending),
                        std::mem::take(&mut slots),
                        &mut output,
                        &mut bytes,
                        limit,
                    )?;
                    if self.exit_status().is_some() {
                        return Ok(output);
                    }
                    let encoded = self.serialize_rejection(error)?;
                    append_batch_text(&mut output, &mut bytes, limit, encoded)?;
                }
            }
        }
        if self.exit_status().is_none() {
            self.dispatch_batch_slots(pending, slots, &mut output, &mut bytes, limit)?;
        }
        Ok(output)
    }

    fn handle_request(
        &mut self,
        request: lsp_server::Request,
    ) -> Result<Vec<Message>, ProtocolError> {
        let lsp_server::Request {
            id,
            method,
            params,
        } = request;
        if let Some(response) = self.cancelled.remove(&id) {
            return self.checked_messages(vec![response.into()]);
        }
        let response_result = self.guarded(&method, |this| {
            debug_panic(&method);
            this.request(&method, params)
        });
        self.requests.incoming.complete(&id);
        let mut output = vec![
            Response {
                id: id.clone(),
                response_result,
            }
            .into(),
        ];
        output.extend(self.messages_from_values(self.shown_messages())?);
        match self.checked_messages(output) {
            Err(ProtocolError::OutputLimit) => self.checked_messages(vec![
                Response {
                    id,
                    response_result: Err(fail(ErrorCode::RequestFailed, OUTPUT_LIMIT)),
                }
                .into(),
            ]),
            other => other,
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Vec<Message>, ProtocolError> {
        let notification = Notification::parse(method);
        match notification {
            Some(Notification::Initialized) => {
                if self.lifecycle == Lifecycle::AwaitInitialized
                    && params.is_object()
                    && serde_json::from_value::<gen_lsp_types::InitializedParams>(params).is_ok()
                {
                    self.lifecycle = Lifecycle::Running;
                }
                return Ok(Vec::new());
            }
            Some(Notification::Cancel) => {
                let result = serde_json::from_value::<CancelParams>(params)
                    .map_err(|_| Error::InvalidEdit("invalid cancellation parameters"))
                    .and_then(|params| self.cancel_request(params.id));
                let errors = result.err().map(|e| log_error(&e.to_string()));
                return self.encode(errors.into_iter().collect(), None);
            }
            Some(Notification::Exit) => {
                let code = u8::from(self.lifecycle != Lifecycle::Shutdown);
                self.lifecycle = Lifecycle::Exited(code);
                return Ok(vec![]);
            }
            _ => {}
        }
        let result = self.guarded(method, |this| {
            debug_panic(method);
            this.notification(notification, params)
        });
        let mut output = match result {
            Ok(messages) => messages,
            // Nothing is logged before `initialize`.
            Err(_) if self.lifecycle != Lifecycle::Running => vec![],
            Err(error) => vec![log_error(&error.message)],
        };
        output.extend(self.shown_messages());
        match self.encode(output, None) {
            Err(ProtocolError::OutputLimit) => self.encode(vec![log_error(OUTPUT_LIMIT)], None),
            other => other,
        }
    }

    /// Run `handler`. A panic in it is that message's error; the server and its documents live on.
    fn guarded<T>(
        &mut self,
        method: &str,
        handler: impl FnOnce(&mut Self) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(self))).unwrap_or_else(
            |payload| {
                let what = payload
                    .downcast_ref::<&str>()
                    .map(std::string::ToString::to_string)
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".into());
                eprintln!("jailsp: internal error in `{method}`: {what}");
                self.session.reset_caches();
                Err(fail(
                    ErrorCode::InternalError,
                    format!("Internal error in `{method}`: {what}"),
                ))
            },
        )
    }

    /// The session's messages for the user, as `window/showMessage` warnings, and its log lines
    /// as `window/logMessage` infos.
    fn shown_messages(&self) -> Vec<Value> {
        if self.lifecycle != Lifecycle::Running {
            return Vec::new();
        }
        let logs = self.session.take_logs().into_iter().map(|message| {
            json!({
                "jsonrpc": "2.0",
                "method": gen_lsp_types::LspNotificationMethod::WindowLogMessage.as_str(),
                "params": { "type": 3, "message": message },
            })
        });
        self.session
            .take_messages()
            .into_iter()
            .map(|message| {
                json!({
                    "jsonrpc": "2.0",
                    "method": gen_lsp_types::LspNotificationMethod::WindowShowMessage.as_str(),
                    "params": { "type": 2, "message": message },
                })
            })
            .chain(logs)
            .chain(self.session.take_limit_notices())
            .collect()
    }

    /// Unknown/completed IDs are ignored; only registered pending requests can be cancelled.
    pub fn cancel_request(&mut self, id: RequestId) -> Result<(), Error> {
        if let Some(response) = self.requests.incoming.cancel(id.clone()) {
            self.cancelled.insert(id, response);
        }
        Ok(())
    }

    fn messages_from_values(&self, messages: Vec<Value>) -> Result<Vec<Message>, ProtocolError> {
        messages
            .into_iter()
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|_| ProtocolError::Serialization("Invalid outgoing message".into()))
            })
            .collect()
    }

    fn checked_messages(&self, messages: Vec<Message>) -> Result<Vec<Message>, ProtocolError> {
        super::message::check_output(&messages, self.session.limits().output_bytes)?;
        Ok(messages)
    }

    fn serialize_messages(&self, messages: Vec<Message>) -> Result<Vec<String>, ProtocolError> {
        self.checked_messages(messages)?
            .iter()
            .map(|message| {
                serde_json::to_string(&super::message::Versioned::new(message))
                    .map_err(|_| ProtocolError::Serialization("Cannot serialize message".into()))
            })
            .collect()
    }

    /// Route rejected complete input through the same lifecycle owner as normal messages.
    pub fn handle_rejected(
        &self,
        rejected: super::RejectedMessage,
    ) -> Option<super::RejectedReply> {
        if self.exit_status().is_some() {
            return None;
        }
        Some(match rejected.id {
            Some(id) => super::RejectedReply::Response(Response {
                id,
                response_result: Err(rejected.error),
            }),
            None => super::RejectedReply::Unidentified(rejected.error),
        })
    }

    fn serialize_rejection(
        &self,
        rejected: super::RejectedMessage,
    ) -> Result<Vec<String>, ProtocolError> {
        match self.handle_rejected(rejected) {
            None => Ok(Vec::new()),
            Some(super::RejectedReply::Response(response)) => {
                self.serialize_messages(vec![response.into()])
            }
            Some(super::RejectedReply::Unidentified(error)) => {
                let text = serde_json::to_string(&super::message::UnidentifiedError(error))
                    .map_err(|_| ProtocolError::Serialization("Cannot serialize error".into()))?;
                if text.len() > self.session.limits().output_bytes {
                    return Err(ProtocolError::OutputLimit);
                }
                Ok(vec![text])
            }
        }
    }

    fn encode_unprompted(&self, messages: Vec<Value>) -> Result<Vec<Message>, ProtocolError> {
        match self.encode(messages, None) {
            Err(ProtocolError::OutputLimit) => self.encode(vec![log_error(OUTPUT_LIMIT)], None),
            other => other,
        }
    }

    fn encode(
        &self,
        messages: Vec<Value>,
        _tail: Option<String>,
    ) -> Result<Vec<Message>, ProtocolError> {
        self.checked_messages(self.messages_from_values(messages)?)
    }

    pub(super) fn ensure_running(&self) -> Result<(), Failure> {
        match self.lifecycle {
            Lifecycle::Running => Ok(()),
            Lifecycle::New | Lifecycle::AwaitInitialized => Err(fail(
                ErrorCode::ServerNotInitialized,
                "Server is not initialized",
            )),
            Lifecycle::Shutdown | Lifecycle::Exited(_) => {
                Err(fail(ErrorCode::InvalidRequest, "Server has shut down"))
            }
        }
    }
}

/// Batch-owned registrations retire even on output failure, exit or caller unwind.
struct PendingBatch<'a> {
    session: &'a mut JsonSession,
    registered: std::collections::BTreeSet<RequestId>,
}

impl Drop for PendingBatch<'_> {
    fn drop(&mut self) {
        for id in &self.registered {
            self.session.requests.incoming.complete(id);
            self.session.cancelled.remove(id);
        }
    }
}

/// Serialize each accepted group before admitting another, rather than retaining large Values.
fn append_batch_output(
    output: &mut Vec<Vec<String>>,
    bytes: &mut usize,
    limit: usize,
    messages: Vec<Message>,
) -> Result<(), ProtocolError> {
    let text = messages
        .iter()
        .map(|message| {
            serde_json::to_string(&super::message::Versioned::new(message))
                .map_err(|_| ProtocolError::Serialization("Cannot serialize message".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    append_batch_text(output, bytes, limit, text)
}

fn append_batch_text(
    output: &mut Vec<Vec<String>>,
    bytes: &mut usize,
    limit: usize,
    text: Vec<String>,
) -> Result<(), ProtocolError> {
    for message in &text {
        *bytes = bytes
            .checked_add(message.len())
            .filter(|n| *n <= limit)
            .ok_or(ProtocolError::OutputLimit)?;
    }
    output.push(text);
    Ok(())
}
