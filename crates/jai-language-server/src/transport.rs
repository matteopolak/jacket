//! Bounded host I/O around the standard library's Message reader and writer.
use crate::protocol::message;
pub use crate::protocol::message::RejectedMessage;
use lsp_server::{Message, RequestId};
use std::io::{self, BufRead, Read};

const HEADER_BYTES: usize = 8192;
const HEADER_LINES: usize = 64;

/// A complete body rejection can be answered and followed by another frame. Framing failures
/// are terminal because a new message boundary cannot be established safely.
#[derive(Debug)]
pub enum Input {
    Ignored {
        bytes: usize,
    },
    Message {
        message: Message,
        bytes: usize,
    },
    Rejected {
        rejected: RejectedMessage,
        bytes: usize,
    },
}

/// Preflight Content-Length before `Message::read` can allocate its body. The library remains
/// responsible for framing consumption and decoding normal messages.
pub struct BoundedReader<R> {
    inner: R,
    limit: usize,
    header_bytes: usize,
    header_lines: usize,
    body_bytes: Option<usize>,
    completed: bool,
    id: Option<RequestId>,
    rejected: Option<RejectedMessage>,
    ignored: bool,
}

impl<R: BufRead> BoundedReader<R> {
    pub fn new(inner: R, limit: usize) -> Self {
        Self {
            inner,
            limit,
            header_bytes: 0,
            header_lines: 0,
            body_bytes: None,
            completed: false,
            id: None,
            rejected: None,
            ignored: false,
        }
    }

    /// Read one frame. Example: `BoundedReader::new(reader, limits.message_bytes).read_frame()`.
    pub fn read_frame(&mut self) -> io::Result<Option<Input>> {
        self.header_bytes = 0;
        self.header_lines = 0;
        self.body_bytes = None;
        self.completed = false;
        self.id = None;
        self.rejected = None;
        self.ignored = false;
        let decoded = Message::read(self);
        let bytes = self.body_bytes.unwrap_or(0);
        if let Some(rejected) = self.rejected.take() {
            return Ok(Some(Input::Rejected {
                rejected,
                bytes,
            }));
        }
        if self.ignored {
            return Ok(Some(Input::Ignored {
                bytes,
            }));
        }
        match decoded {
            Ok(Some(message)) => Ok(Some(Input::Message {
                message,
                bytes,
            })),
            Ok(None) => Ok(None),
            Err(_) if self.completed => Ok(Some(Input::Rejected {
                rejected: message::rejected(
                    self.id.take(),
                    lsp_server::ErrorCode::InvalidRequest,
                    "Invalid request",
                ),
                bytes,
            })),
            // Message::read errors can include the entire payload/header. Never expose Display.
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid or truncated LSP frame",
            )),
        }
    }
}

impl<R: BufRead> Read for BoundedReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.inner.read(output)
    }

    fn read_exact(&mut self, output: &mut [u8]) -> io::Result<()> {
        if self.body_bytes != Some(output.len()) {
            return Err(io::Error::other("Invalid LSP body length"));
        }
        self.inner.read_exact(output)?;
        self.completed = true;
        let result = match std::str::from_utf8(output) {
            Ok(text) => message::validate_metadata(text),
            Err(_) => Err(message::rejected(
                None,
                lsp_server::ErrorCode::ParseError,
                "Parse error",
            )),
        };
        match result {
            Ok(message::MetadataKind::Message(id)) => {
                self.id = id;
                Ok(())
            }
            Ok(message::MetadataKind::UnidentifiedResponse) => {
                self.ignored = true;
                Err(io::Error::other("Unidentified peer response"))
            }
            Err(rejected) => {
                self.rejected = Some(rejected);
                Err(io::Error::other("Rejected LSP body"))
            }
        }
    }
}

impl<R: BufRead> BufRead for BoundedReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }

    fn read_line(&mut self, output: &mut String) -> io::Result<usize> {
        let start = output.len();
        loop {
            let available = self.inner.fill_buf()?;
            if available.is_empty() {
                if output.len() == start && self.header_bytes == 0 {
                    return Ok(0);
                }
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Truncated LSP header",
                ));
            }
            let count = available
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(available.len(), |at| at + 1);
            self.header_bytes = self
                .header_bytes
                .checked_add(count)
                .filter(|n| *n <= HEADER_BYTES)
                .ok_or_else(|| io::Error::other("LSP header limit"))?;
            // Only ASCII header bytes are accepted, so extending String cannot require a
            // temporary unbounded byte buffer or split a multibyte sequence across fill_buf.
            if !available[..count].is_ascii() {
                return Err(io::Error::other("Invalid LSP header"));
            }
            output.push_str(std::str::from_utf8(&available[..count]).expect("ASCII checked"));
            self.inner.consume(count);
            if output.ends_with('\n') {
                break;
            }
        }
        self.header_lines += 1;
        if self.header_lines > HEADER_LINES || !output[start..].ends_with("\r\n") {
            return Err(io::Error::other("Invalid LSP header"));
        }
        let line = &output[start..output.len() - 2];
        if line.is_empty() {
            if self.body_bytes.is_none() {
                return Err(io::Error::other("Missing Content-Length"));
            }
        } else {
            let (name, value) = line
                .split_once(": ")
                .ok_or_else(|| io::Error::other("Invalid LSP header"))?;
            if name.eq_ignore_ascii_case("Content-Length") {
                if self.body_bytes.is_some() {
                    return Err(io::Error::other("Duplicate Content-Length"));
                }
                let length = value
                    .parse::<usize>()
                    .map_err(|_| io::Error::other("Invalid Content-Length"))?;
                if length > self.limit {
                    return Err(io::Error::other("LSP body limit"));
                }
                self.body_bytes = Some(length);
            } else if name.eq_ignore_ascii_case("Content-Type") {
                for parameter in value.split(';').skip(1) {
                    if let Some((key, value)) = parameter.trim().split_once('=')
                        && key.trim().eq_ignore_ascii_case("charset")
                    {
                        let charset = value.trim().trim_matches('"');
                        if !charset.eq_ignore_ascii_case("utf-8")
                            && !charset.eq_ignore_ascii_case("utf8")
                        {
                            return Err(io::Error::other("Unsupported LSP encoding"));
                        }
                    }
                }
            }
        }
        Ok(output.len() - start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn frame(body: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{body}", body.len())
    }

    #[test]
    fn rejected_complete_bodies_preserve_order_and_frame_boundaries() {
        let text = frame("{")
            + &frame(r#"{"jsonrpc":"2.0","id":null,"method":"exit"}"#)
            + &frame(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
            );
        let mut reader = BoundedReader::new(Cursor::new(text), 1024);
        assert!(matches!(
            reader.read_frame().unwrap(),
            Some(Input::Rejected { .. })
        ));
        assert!(matches!(
            reader.read_frame().unwrap(),
            Some(Input::Rejected { .. })
        ));
        assert!(matches!(
            reader.read_frame().unwrap(),
            Some(Input::Message { .. })
        ));
        assert!(reader.read_frame().unwrap().is_none());
    }

    #[test]
    fn ignored_peer_responses_consume_exactly_one_frame_and_reset_state() {
        let ignored = r#"{"jsonrpc":"2.0","id":null,"result":null}"#;
        let text = frame(ignored)
            + &frame(ignored)
            + &frame(r#"{"jsonrpc":"2.0","id":null,"method":"exit"}"#)
            + &frame(r#"{"jsonrpc":"2.0","id":1,"method":"shutdown"}"#);
        let mut reader = BoundedReader::new(Cursor::new(text), 1024);
        for _ in 0..2 {
            assert!(
                matches!(reader.read_frame().unwrap(), Some(Input::Ignored { bytes }) if bytes == ignored.len())
            );
        }
        assert!(matches!(
            reader.read_frame().unwrap(),
            Some(Input::Rejected { .. })
        ));
        assert!(matches!(
            reader.read_frame().unwrap(),
            Some(Input::Message { .. })
        ));
        assert!(reader.read_frame().unwrap().is_none());
    }

    #[test]
    fn header_and_body_caps_precede_body_consumption() {
        for text in [
            "Content-Length: 999999999\r\n\r\nsecret".to_owned(),
            format!("X: {}\r\n", "a".repeat(HEADER_BYTES)),
            "Content-Length: 1\r\nContent-Length: 1\r\n\r\nx".into(),
            "Content-Length: 1\r\n".into(),
            "Content-Length: 4\r\n\r\nx".into(),
        ] {
            let mut reader = BoundedReader::new(Cursor::new(text), 64);
            let error = reader.read_frame().unwrap_err();
            assert_eq!(error.to_string(), "Invalid or truncated LSP frame");
        }
    }
}

/// Ordered routing metadata for the Connection payload channel. A normal message is sent
/// before its token; rejected bodies use this same stream and cannot overtake normal input.
#[derive(Debug)]
pub enum Arrival {
    Ignored {
        bytes: usize,
        frame: Option<crate::trace::Frame>,
    },
    Message {
        bytes: usize,
        frame: Option<crate::trace::Frame>,
    },
    Rejected {
        rejected: RejectedMessage,
        bytes: usize,
        frame: Option<crate::trace::Frame>,
    },
    Closed(io::Result<()>),
}

impl Arrival {
    /// Timing identity stays attached to a held/lookahead routing token.
    pub fn frame(&self) -> Option<crate::trace::Frame> {
        match self {
            Self::Ignored {
                frame, ..
            }
            | Self::Message {
                frame, ..
            }
            | Self::Rejected {
                frame, ..
            } => *frame,
            Self::Closed(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
struct OutputStamp {
    ordinal: u64,
    origin: crate::trace::Origin,
    category: crate::trace::Category,
    bytes: usize,
}

impl OutputStamp {
    fn record(self, stage: crate::trace::Stage) {
        if let Some(trace) = crate::trace::global() {
            trace.output_event(self.origin, self.ordinal, stage, self.category, self.bytes);
        }
    }
}

enum WriteItem {
    Message(Option<OutputStamp>),
    Unidentified(message::UnidentifiedError, Option<OutputStamp>),
    Finish(crossbeam_channel::Sender<()>),
}

#[derive(Default)]
struct TraceCompletion {
    reader: std::sync::atomic::AtomicBool,
    writer: std::sync::atomic::AtomicBool,
}

fn trace_category(message: &Message) -> crate::trace::Category {
    use crate::trace::Category;
    use lsp_types::{notification::Notification as _, request::Request as _};
    match message {
        Message::Request(request) => match request.method.as_str() {
            lsp_types::request::Initialize::METHOD | lsp_types::request::Shutdown::METHOD => {
                Category::Lifecycle
            }
            lsp_types::request::HoverRequest::METHOD => Category::Hover,
            lsp_types::request::References::METHOD => Category::References,
            lsp_types::request::LinkedEditingRange::METHOD => Category::LinkedEditing,
            lsp_types::request::PrepareRenameRequest::METHOD
            | lsp_types::request::Rename::METHOD => Category::Rename,
            lsp_types::request::DocumentDiagnosticRequest::METHOD
            | lsp_types::request::WorkspaceDiagnosticRequest::METHOD => Category::Diagnostics,
            lsp_types::request::Completion::METHOD
            | lsp_types::request::ResolveCompletionItem::METHOD => Category::Completion,
            _ => Category::Request,
        },
        Message::Notification(notification) => match notification.method.as_str() {
            lsp_types::notification::DidOpenTextDocument::METHOD
            | lsp_types::notification::DidChangeTextDocument::METHOD
            | lsp_types::notification::DidCloseTextDocument::METHOD => Category::Change,
            lsp_types::notification::Initialized::METHOD
            | lsp_types::notification::Exit::METHOD => Category::Lifecycle,
            _ => Category::Notification,
        },
        Message::Response(_) => Category::Response,
    }
}

fn transfer(frame: Option<crate::trace::Frame>, stage: crate::trace::Stage) {
    if let (Some(trace), Some(frame)) = (crate::trace::global(), frame) {
        trace.event(
            crate::trace::Origin::Frame(frame.ordinal),
            stage,
            frame.category,
            frame.bytes,
        );
    }
}

/// The standard Connection owns normal message channels. Small ordered control streams add
/// host bounds, arrival interruption and the otherwise unrepresentable null-ID error edge.
/// No compiler state or filesystem object crosses these channels.
pub struct HostTransport {
    connection: lsp_server::Connection,
    pub arrivals: crossbeam_channel::Receiver<Arrival>,
    pub failures: crossbeam_channel::Receiver<()>,
    writes: crossbeam_channel::Sender<WriteItem>,
    output_limit: usize,
    trace_completion: Option<std::sync::Arc<TraceCompletion>>,
}

impl HostTransport {
    /// Open bounded stdio transport. Example: `HostTransport::stdio(Limits::default())`.
    pub fn stdio(limits: crate::Limits) -> Self {
        let (input_tx, input_rx) = crossbeam_channel::bounded(1);
        let (arrival_tx, arrival_rx) = crossbeam_channel::bounded(1);
        let trace_completion =
            crate::trace::global().map(|_| std::sync::Arc::new(TraceCompletion::default()));
        let reader_completion = trace_completion.clone();
        let writer_completion = trace_completion.clone();
        std::thread::Builder::new()
            .name("lsp-reader".into())
            .spawn(move || {
                let mut reader = BoundedReader::new(std::io::stdin().lock(), limits.message_bytes);
                loop {
                    let next = reader.read_frame();
                    let frame = crate::trace::global().and_then(|trace| match &next {
                        Ok(Some(Input::Message {
                            message,
                            bytes,
                        })) => trace.frame(trace_category(message), *bytes),
                        Ok(Some(Input::Ignored {
                            bytes,
                        })) => trace.frame(crate::trace::Category::Ignored, *bytes),
                        Ok(Some(Input::Rejected {
                            bytes, ..
                        })) => trace.frame(crate::trace::Category::Rejected, *bytes),
                        _ => None,
                    });
                    transfer(frame, crate::trace::Stage::SendStart);
                    let event = match next {
                        Ok(Some(Input::Message {
                            message,
                            bytes,
                        })) => {
                            if input_tx.send(message).is_err() {
                                break;
                            }
                            Arrival::Message {
                                bytes,
                                frame,
                            }
                        }
                        Ok(Some(Input::Ignored {
                            bytes,
                        })) => Arrival::Ignored {
                            bytes,
                            frame,
                        },
                        Ok(Some(Input::Rejected {
                            rejected,
                            bytes,
                        })) => Arrival::Rejected {
                            rejected,
                            bytes,
                            frame,
                        },
                        Ok(None) => Arrival::Closed(Ok(())),
                        Err(error) => Arrival::Closed(Err(error)),
                    };
                    let done = matches!(event, Arrival::Closed(_));
                    if done && let Some(completion) = &reader_completion {
                        // No more frame events can be produced after terminal input.
                        completion
                            .reader
                            .store(true, std::sync::atomic::Ordering::Release);
                    }
                    if arrival_tx.send(event).is_err() {
                        break;
                    }
                    // Preserve queue-before-interrupt ordering; logging cannot delay the flag.
                    jaic::interrupt::request(true);
                    transfer(frame, crate::trace::Stage::SendReturned);
                    if done {
                        break;
                    }
                }
                if let Some(completion) = reader_completion {
                    completion
                        .reader
                        .store(true, std::sync::atomic::Ordering::Release);
                }
            })
            .expect("start LSP reader");
        let (output_tx, output_rx) = crossbeam_channel::bounded::<Message>(1);
        let (write_tx, write_rx) = crossbeam_channel::bounded(1);
        let (failed, failures) = crossbeam_channel::bounded(1);
        std::thread::Builder::new()
            .name("lsp-writer".into())
            .spawn(move || {
                use std::io::Write;
                let mut output = std::io::stdout().lock();
                while let Ok(item) = write_rx.recv() {
                    let result = match item {
                        WriteItem::Message(stamp) => match output_rx.recv() {
                            Ok(message) => {
                                if let Some(stamp) = stamp {
                                    stamp.record(crate::trace::Stage::WriteStart);
                                }
                                let result = message.write(&mut output);
                                if let Some(stamp) = stamp {
                                    stamp.record(if result.is_ok() {
                                        crate::trace::Stage::WriteDone
                                    } else {
                                        crate::trace::Stage::WriteFailed
                                    });
                                }
                                result
                            }
                            Err(_) => break,
                        },
                        WriteItem::Unidentified(error, stamp) => {
                            if let Some(stamp) = stamp {
                                stamp.record(crate::trace::Stage::WriteStart);
                            }
                            let result = serde_json::to_vec(&error)
                                .map_err(io::Error::other)
                                .and_then(|bytes| {
                                    write!(output, "Content-Length: {}\r\n\r\n", bytes.len())?;
                                    output.write_all(&bytes)?;
                                    output.flush()
                                });
                            if let Some(stamp) = stamp {
                                stamp.record(if result.is_ok() {
                                    crate::trace::Stage::WriteDone
                                } else {
                                    crate::trace::Stage::WriteFailed
                                });
                            }
                            result
                        }
                        WriteItem::Finish(done) => {
                            if output.flush().is_ok() {
                                if let Some(completion) = &writer_completion {
                                    completion
                                        .writer
                                        .store(true, std::sync::atomic::Ordering::Release);
                                }
                                let _ = done.send(());
                            }
                            break;
                        }
                    };
                    if result.is_err() {
                        let _ = failed.send(());
                        jaic::interrupt::request(true);
                        break;
                    }
                }
                if let Some(completion) = writer_completion {
                    completion
                        .writer
                        .store(true, std::sync::atomic::Ordering::Release);
                }
            })
            .expect("start LSP writer");
        Self {
            connection: lsp_server::Connection {
                sender: output_tx,
                receiver: input_rx,
            },
            arrivals: arrival_rx,
            failures,
            writes: write_tx,
            output_limit: limits.output_bytes,
            trace_completion,
        }
    }

    /// Resolve an ordered arrival token to its library message.
    pub fn take(&self, arrival: Arrival) -> io::Result<Option<Input>> {
        let frame = arrival.frame();
        let input = match arrival {
            Arrival::Ignored {
                bytes, ..
            } => Ok(Some(Input::Ignored {
                bytes,
            })),
            Arrival::Message {
                bytes, ..
            } => self
                .connection
                .receiver
                .recv()
                .map(|message| {
                    Some(Input::Message {
                        message,
                        bytes,
                    })
                })
                .map_err(|_| io::Error::other("LSP input disconnected")),
            Arrival::Rejected {
                rejected,
                bytes,
                ..
            } => Ok(Some(Input::Rejected {
                rejected,
                bytes,
            })),
            Arrival::Closed(result) => result.map(|()| None),
        };
        if input.is_ok() {
            transfer(frame, crate::trace::Stage::OwnerDequeued);
        }
        input
    }

    /// True only after both native threads can produce no further trace events.
    pub fn trace_quiesced(&self) -> bool {
        self.trace_completion.as_ref().is_some_and(|state| {
            state.reader.load(std::sync::atomic::Ordering::Acquire)
                && state.writer.load(std::sync::atomic::Ordering::Acquire)
        })
    }

    /// Queue standard messages under the same body cap as the browser adapter.
    pub fn send(&self, messages: Vec<Message>) -> io::Result<()> {
        let trace = crate::trace::global();
        let mut lengths = Vec::new();
        message::check_output_with_lengths(&messages, self.output_limit, |bytes| {
            if trace.is_some() {
                lengths.push(bytes);
            }
        })
        .map_err(io::Error::other)?;
        for (at, message) in messages.into_iter().enumerate() {
            let stamp = trace.and_then(|trace| {
                Some(OutputStamp {
                    ordinal: trace.output_ordinal()?,
                    origin: crate::trace::current_origin(),
                    category: trace_category(&message),
                    bytes: lengths[at],
                })
            });
            if let Some(stamp) = stamp {
                stamp.record(crate::trace::Stage::OutputEnqueueStart);
            }
            self.connection
                .sender
                .send(message)
                .map_err(|_| io::Error::other("LSP output disconnected"))?;
            self.writes
                .send(WriteItem::Message(stamp))
                .map_err(|_| io::Error::other("LSP output disconnected"))?;
            if let Some(stamp) = stamp {
                stamp.record(crate::trace::Stage::OutputEnqueued);
            }
        }
        Ok(())
    }

    /// Only the semantic owner may answer a rejected body, preserving dispatch/lifecycle order.
    pub fn send_error(&self, reply: crate::RejectedReply) -> io::Result<()> {
        let edge = match reply {
            crate::RejectedReply::Response(response) => return self.send(vec![response.into()]),
            crate::RejectedReply::Unidentified(error) => message::UnidentifiedError(error),
        };
        let bytes = serde_json::to_vec(&edge).map_err(io::Error::other)?;
        if bytes.len() > self.output_limit {
            return Err(io::Error::other("LSP output limit"));
        }
        let stamp = crate::trace::global().and_then(|trace| {
            Some(OutputStamp {
                ordinal: trace.output_ordinal()?,
                origin: crate::trace::current_origin(),
                category: crate::trace::Category::Response,
                bytes: bytes.len(),
            })
        });
        if let Some(stamp) = stamp {
            stamp.record(crate::trace::Stage::OutputEnqueueStart);
        }
        self.writes
            .send(WriteItem::Unidentified(edge, stamp))
            .map_err(|_| io::Error::other("LSP output disconnected"))?;
        if let Some(stamp) = stamp {
            stamp.record(crate::trace::Stage::OutputEnqueued);
        }
        Ok(())
    }

    /// Flush accepted output with a bounded wait. Aborting the core never joins a reader
    /// blocked on an open stdin pipe or a writer whose client stopped reading.
    pub fn finish(&self) -> io::Result<()> {
        let (done, finished) = crossbeam_channel::bounded(1);
        let timeout = std::time::Duration::from_secs(2);
        self.writes
            .send_timeout(WriteItem::Finish(done), timeout)
            .map_err(|_| io::Error::other("LSP output did not drain"))?;
        finished
            .recv_timeout(timeout)
            .map_err(|_| io::Error::other("LSP output did not flush"))
    }
}
