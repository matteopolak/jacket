//! Standard work-done progress for native indexing, created before any work begins.
use super::{JsonSession, ProtocolError};
use serde_json::{Value, json};

#[derive(Default)]
pub(super) struct IndexProgress {
    pub supported: bool,
    pub initialized: bool,
    sequence: u64,
    waiting: Option<String>,
    ready: bool,
    active: bool,
    message: String,
}

impl JsonSession {
    /// Start or finish the native transport's indexing indicator. `true` asks the transport
    /// to serve input until the client acknowledges creation of the progress token.
    pub fn indexing_progress(
        &mut self,
        working: bool,
    ) -> Result<(bool, Vec<String>), ProtocolError> {
        if !self.index_progress.supported
            || !self.index_progress.initialized
            || !working && !self.index_progress.active
        {
            return Ok((false, Vec::new()));
        }
        let message = self.session.indexing_message();
        let state = &mut self.index_progress;
        let token = format!("jai-index-{}", state.sequence);
        let messages = if !working {
            if !state.active {
                return Ok((false, Vec::new()));
            }
            state.active = false;
            state.ready = false;
            vec![json!({"jsonrpc":"2.0","method":"$/progress","params":{
                "token":token,"value":{"kind":"end"}}})]
        } else if state.waiting.is_some() {
            return Ok((true, Vec::new()));
        } else if state.ready {
            if state.active {
                if state.message == message {
                    return Ok((false, Vec::new()));
                }
                state.message.clone_from(&message);
                vec![json!({"jsonrpc":"2.0","method":"$/progress","params":{
                    "token":token,"value":{"kind":"report","message":message}}})]
            } else {
                state.active = true;
                state.message.clone_from(&message);
                vec![json!({"jsonrpc":"2.0","method":"$/progress","params":{
                    "token":token,"value":{"kind":"begin","title":"jailsp","message":message,"cancellable":false}}})]
            }
        } else {
            state.sequence += 1;
            let token = format!("jai-index-{}", state.sequence);
            let id = format!("jai-index-create-{}", state.sequence);
            state.waiting = Some(id.clone());
            vec![
                json!({"jsonrpc":"2.0","id":id,"method":"window/workDoneProgress/create","params":{"token":token}}),
            ]
        };
        Ok((state.waiting.is_some(), self.encode_unprompted(messages)?))
    }

    pub(super) fn progress_reply(&mut self, input: &str) {
        if self.index_progress.waiting.is_none() {
            return;
        }
        let Ok(value) = serde_json::from_str::<Value>(input) else {
            return;
        };
        if value.get("method").is_some()
            || value["id"].as_str() != self.index_progress.waiting.as_deref()
        {
            return;
        }
        self.index_progress.waiting = None;
        if value.get("error").is_some() {
            self.index_progress.supported = false;
        } else {
            self.index_progress.ready = true;
        }
    }
}
