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
    percentage: u32,
    finished: usize,
}

impl IndexProgress {
    /// The `(percentage, message)` for a combined `(finished, total)` count. Both only move
    /// forward within one begin/end, even when a build metaprogram grows the total.
    fn advance(&mut self, finished: usize, total: usize) -> (u32, String) {
        if total == 0 {
            return (self.percentage, "indexing".into());
        }
        self.finished = self.finished.max(finished);
        let total = total.max(self.finished);
        let percentage = u32::try_from(self.finished * 100 / total).unwrap_or(100);
        self.percentage = self.percentage.max(percentage);
        (self.percentage, format!("{}/{total}", self.finished))
    }

    fn restart(&mut self) {
        self.percentage = 0;
        self.finished = 0;
    }
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
        let (finished, total) = self.session.indexing_counts();
        let state = &mut self.index_progress;
        let (percentage, message) = state.advance(finished, total);
        let token = format!("jai-index-{}", state.sequence);
        let messages = if !working {
            if !state.active {
                return Ok((false, Vec::new()));
            }
            state.active = false;
            state.ready = false;
            state.restart();
            self.session.reset_indexing_counts();
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
                    "token":token,"value":{"kind":"report","message":message,"percentage":percentage}}})]
            } else {
                state.active = true;
                state.message.clone_from(&message);
                vec![json!({"jsonrpc":"2.0","method":"$/progress","params":{
                    "token":token,"value":{"kind":"begin","title":"jailsp","message":message,"percentage":percentage,"cancellable":false}}})]
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

#[cfg(test)]
mod tests {
    use super::IndexProgress;

    #[test]
    fn a_growing_total_does_not_move_the_bar_back() {
        let mut progress = IndexProgress::default();
        assert_eq!(progress.advance(0, 0), (0, "indexing".into()));
        assert_eq!(progress.advance(2, 4), (50, "2/4".into()));
        // A build metaprogram added entries: the bar holds, the counter keeps its meaning.
        assert_eq!(progress.advance(3, 12), (50, "3/12".into()));
        // A restarted phase reports fewer finished entries than before.
        assert_eq!(progress.advance(1, 12), (50, "3/12".into()));
        assert_eq!(progress.advance(12, 12), (100, "12/12".into()));
        progress.restart();
        assert_eq!(progress.advance(1, 5), (20, "1/5".into()));
    }
}
