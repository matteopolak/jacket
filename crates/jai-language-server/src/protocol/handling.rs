//! Constructing a session and routing incoming messages.
use super::helpers::{fail, failure, log_error, valid_id};
use super::methods::{
    ErrorCode, Failure, Incoming, Notification, OUTPUT_LIMIT, Reply, debug_panic, incoming,
};
use super::params::CancelParams;
use super::{JsonSession, Lifecycle, ProtocolError, RequestId};
use crate::{Error, Limits, MarkupKind, Session};
use serde_json::{Value, json};
use std::collections::VecDeque;

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
            cancelled: VecDeque::new(),
            completed: VecDeque::new(),
            hover_kind: MarkupKind::PlainText,
            signature_kind: MarkupKind::PlainText,
            pull_diagnostics: false,
            defer_publications: false,
            publications_pending: false,
            refresh_diagnostics: false,
            refreshes: 0,
            refresh_code_lenses: false,
            lenses_changed: false,
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
        self.session.warm_auto_import();
    }

    /// Whether `background_work` has something to do.
    pub fn has_background_work(&self) -> bool {
        self.session.has_background_work()
    }

    /// One piece of work a request left for later (see `Session::background_work`), for a
    /// transport to do while no input waits. Once none is left, a client that supports it is
    /// asked to fetch the code lenses it changed again.
    pub fn background_work(&mut self) -> Result<Vec<String>, ProtocolError> {
        let changed = self.guarded("background work", |this| Ok(this.session.background_work()));
        let mut messages = match changed {
            Ok(changed) => {
                self.lenses_changed |= changed;
                Vec::new()
            }
            Err((_, message)) => vec![log_error(&message)],
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
        self.refreshes += 1;
        Some(json!({
            "jsonrpc": "2.0",
            "id": format!("jai-code-lens-refresh-{}", self.refreshes),
            "method": "workspace/codeLens/refresh",
        }))
    }

    /// A compile-time service answer is pending: poll `service_arrivals` now and then.
    pub fn services_waiting(&self) -> bool {
        self.session.services_waiting()
    }

    /// Checks the open files again when pending service answers came in (publishing, or asking
    /// a pulling client to pull).
    pub fn service_arrivals(&mut self) -> Result<Vec<String>, ProtocolError> {
        if !self.session.take_service_arrivals() {
            return Ok(Vec::new());
        }
        let messages = self.edited(true);
        self.encode_unprompted(messages)
    }

    /// An edit is waiting for `flush_publications`.
    pub fn publications_pending(&self) -> bool {
        self.publications_pending
    }

    /// The diagnostics of every open document as they are now (one compile at most), for the
    /// edits since the last flush. Empty for a client that pulls.
    pub fn flush_publications(&mut self) -> Result<Vec<String>, ProtocolError> {
        if !std::mem::take(&mut self.publications_pending) {
            return Ok(Vec::new());
        }
        let messages = self.publications_now();
        self.encode_unprompted(messages)
    }

    pub fn handle_json(&mut self, input: &str) -> Result<Vec<String>, ProtocolError> {
        if input.len() > self.session.limits().message_bytes {
            return Err(ProtocolError::MessageLimit);
        }
        match incoming(input) {
            Err(error) => self.encode(vec![error], None),
            // The client's answer to a request of ours (a refresh): nothing to do.
            Ok(None) => Ok(vec![]),
            Ok(Some(Incoming {
                id: Some(id),
                method,
                params,
            })) => self.handle_request(id, &method, params),
            Ok(Some(Incoming {
                id: None,
                method,
                params,
            })) => self.handle_notification(&method, params),
        }
    }

    /// Handle messages that arrived together, one answer list per message in order, up to the
    /// one that ends the session. Cancellations among them are noted first, so a request
    /// queued behind the work in front of it can still be cancelled.
    pub fn handle_batch(&mut self, inputs: &[String]) -> Result<Vec<Vec<String>>, ProtocolError> {
        for input in inputs {
            self.note_cancellation(input);
        }
        let mut answers = Vec::with_capacity(inputs.len());
        for input in inputs {
            answers.push(self.handle_json(input)?);
            if self.exit_status().is_some() {
                break;
            }
        }
        Ok(answers)
    }

    fn note_cancellation(&mut self, input: &str) {
        if input.len() > self.session.limits().message_bytes || !input.contains("$/cancelRequest") {
            return;
        }
        if let Ok(Some(Incoming {
            id: None,
            method,
            params,
        })) = incoming(input)
            && Notification::parse(&method) == Some(Notification::Cancel)
            && let Ok(params) = serde_json::from_value::<CancelParams>(params)
        {
            let _ = self.cancel_request(params.id);
        }
    }

    fn handle_request(
        &mut self,
        id: RequestId,
        method: &str,
        params: Value,
    ) -> Result<Vec<String>, ProtocolError> {
        let identifier = json!(id);
        if let Some(at) = self.cancelled.iter().position(|cancelled| *cancelled == id) {
            self.cancelled.remove(at);
            self.remember(id);
            let cancelled = failure(identifier, ErrorCode::RequestCancelled, "Request cancelled");
            return self.encode(vec![cancelled], None);
        }
        let result = self.guarded(method, |this| {
            debug_panic(method);
            this.request(method, params)
        });
        self.remember(id);
        let (mut output, tail) = match result {
            Ok(Reply::Value(result)) => (
                vec![json!({ "jsonrpc": "2.0", "id": identifier, "result": result })],
                None,
            ),
            Ok(Reply::Raw(raw)) => (
                vec![],
                Some(format!(
                    "{{\"jsonrpc\":\"2.0\",\"id\":{identifier},\"result\":{raw}}}"
                )),
            ),
            Err((code, message)) => (vec![failure(identifier.clone(), code, &message)], None),
        };
        output.extend(self.shown_messages());
        match self.encode(output, tail) {
            // A response over the output cap is that request's error, not the end of the session.
            Err(ProtocolError::OutputLimit) => self.encode(
                vec![failure(identifier, ErrorCode::RequestFailed, OUTPUT_LIMIT)],
                None,
            ),
            other => other,
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Vec<String>, ProtocolError> {
        let notification = Notification::parse(method);
        match notification {
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
            Err(_) if self.lifecycle == Lifecycle::New => vec![],
            Err((_, message)) => vec![log_error(&message)],
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
        let logs = self.session.take_logs().into_iter().map(|message| {
            json!({
                "jsonrpc": "2.0",
                "method": "window/logMessage",
                "params": { "type": 3, "message": message },
            })
        });
        self.session
            .take_messages()
            .into_iter()
            .map(|message| {
                json!({
                    "jsonrpc": "2.0",
                    "method": "window/showMessage",
                    "params": { "type": 2, "message": message },
                })
            })
            .chain(logs)
            .chain(self.session.take_limit_notices())
            .collect()
    }

    pub fn cancel_request(&mut self, id: RequestId) -> Result<(), Error> {
        if !valid_id(&id) {
            return Err(Error::InvalidEdit("invalid cancellation id"));
        }
        if self.completed.contains(&id) {
            return Ok(());
        }
        if self.cancelled.contains(&id) {
            return Ok(());
        }
        // A cancellation whose request never comes would sit here for good: the oldest goes.
        let limit = self.session.limits().cancelled_requests;
        if limit == 0 {
            return Ok(());
        }
        if self.cancelled.len() >= limit {
            self.cancelled.pop_front();
        }
        self.cancelled.push_back(id);
        Ok(())
    }

    fn remember(&mut self, id: RequestId) {
        let limit = self.session.limits().cancelled_requests;
        if limit == 0 {
            return;
        }
        if self.completed.len() >= limit {
            self.completed.pop_front();
        }
        self.completed.push_back(id);
    }

    /// `messages` that no request asked for. Over the output cap they are replaced by a log line:
    /// there is no request to fail, and the server keeps running.
    fn encode_unprompted(&self, messages: Vec<Value>) -> Result<Vec<String>, ProtocolError> {
        match self.encode(messages, None) {
            Err(ProtocolError::OutputLimit) => self.encode(vec![log_error(OUTPUT_LIMIT)], None),
            other => other,
        }
    }

    /// The messages as JSON text, then `tail` (already text: the response to a large request).
    fn encode(
        &self,
        messages: Vec<Value>,
        tail: Option<String>,
    ) -> Result<Vec<String>, ProtocolError> {
        let mut out = vec![];
        let mut bytes = 0usize;
        let texts = messages
            .into_iter()
            .map(|message| {
                serde_json::to_string(&message)
                    .map_err(|e| ProtocolError::Serialization(e.to_string()))
            })
            .chain(tail.map(Ok));
        for text in texts {
            let text = text?;
            bytes = bytes
                .checked_add(text.len())
                .filter(|n| *n <= self.session.limits().output_bytes)
                .ok_or(ProtocolError::OutputLimit)?;
            out.push(text);
        }
        Ok(out)
    }

    pub(super) fn ensure_running(&self) -> Result<(), Failure> {
        match self.lifecycle {
            Lifecycle::Running => Ok(()),
            Lifecycle::New => Err(fail(
                ErrorCode::ServerNotInitialized,
                "Server is not initialized",
            )),
            Lifecycle::Shutdown | Lifecycle::Exited(_) => {
                Err(fail(ErrorCode::InvalidRequest, "Server has shut down"))
            }
        }
    }
}
