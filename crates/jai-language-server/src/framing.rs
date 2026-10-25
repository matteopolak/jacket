//! Incremental LSP framing. This module has no stdio or filesystem operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameError {
    HeaderLimit,
    BodyLimit,
    InvalidHeader,
    InvalidLength,
    UnsupportedEncoding,
    InvalidUtf8,
    Incomplete,
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = match self {
            FrameError::HeaderLimit => "a message header is too long",
            FrameError::BodyLimit => "a message is larger than the size limit",
            FrameError::InvalidHeader => {
                "a message header is not of the form `Name: value` (expected `Content-Length: N`)"
            }
            FrameError::InvalidLength => "a `Content-Length` header is not a number",
            FrameError::UnsupportedEncoding => "a message asks for a charset other than UTF-8",
            FrameError::InvalidUtf8 => "a message is not valid UTF-8",
            FrameError::Incomplete => "the input ended in the middle of a message",
        };
        write!(f, "invalid LSP input: {what}")
    }
}

impl std::error::Error for FrameError {
}

/// A message that was over the size limit and thrown away unread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Oversized {
    pub length: usize,
    /// The first bytes of its body, enough to find the request id.
    pub head: Vec<u8>,
}

pub struct FrameDecoder {
    buffer: Vec<u8>,
    expected: Option<usize>,
    body_limit: usize,
    /// Bytes of an oversized message still to be thrown away as they arrive.
    discarding: usize,
    /// The message being thrown away: its declared length and the start of its body so far.
    skipping: Option<Oversized>,
    /// The messages thrown away since `take_oversized` was last called.
    oversized: Vec<Oversized>,
}

const HEADER_LIMIT: usize = 8192;

/// How much of an oversized body is kept to find its request id.
const HEAD_LIMIT: usize = 4096;

impl FrameDecoder {
    pub fn new(body_limit: usize) -> Self {
        Self {
            buffer: vec![],
            expected: None,
            body_limit,
            discarding: 0,
            skipping: None,
            oversized: vec![],
        }
    }

    /// The messages that were over the limit and thrown away unread (their bytes are skipped as
    /// they arrive, so the stream stays in step), each reported once.
    pub fn take_oversized(&mut self) -> Vec<Oversized> {
        std::mem::take(&mut self.oversized)
    }

    /// Throw away what is left of an oversized body from the front of `bytes`; how many bytes
    /// that took.
    fn skip(&mut self, bytes: &[u8]) -> usize {
        let skipped = self.discarding.min(bytes.len());
        self.discarding -= skipped;
        if let Some(message) = &mut self.skipping {
            let room = HEAD_LIMIT.saturating_sub(message.head.len());
            message.head.extend_from_slice(&bytes[..skipped.min(room)]);
            if message.head.len() >= HEAD_LIMIT || self.discarding == 0 {
                self.oversized.extend(self.skipping.take());
            }
        }
        skipped
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, FrameError> {
        let skipped = self.skip(bytes);
        let bytes = &bytes[skipped..];
        self.buffer.extend_from_slice(bytes);
        let mut messages = vec![];
        loop {
            let length = match self.expected {
                Some(length) => length,
                None => match self.header()? {
                    None => break,
                    Some(length) if length > self.body_limit => {
                        self.discarding = length;
                        self.skipping = Some(Oversized {
                            length,
                            head: vec![],
                        });
                        let buffered = std::mem::take(&mut self.buffer);
                        let skipped = self.skip(&buffered);
                        self.buffer = buffered[skipped..].to_vec();
                        continue;
                    }
                    Some(length) => *self.expected.insert(length),
                },
            };
            if self.buffer.len() < length {
                break;
            }
            let body = std::str::from_utf8(&self.buffer[..length])
                .map_err(|_| FrameError::InvalidUtf8)?
                .to_owned();
            self.buffer.drain(..length);
            self.expected = None;
            messages.push(body);
        }
        // What is left is one incomplete message: its body (at most the limit, checked when its
        // header was read) or its header. Complete messages in the same chunk do not count.
        let limit = self.body_limit.saturating_add(HEADER_LIMIT);
        if self.buffer.len() > limit {
            return Err(FrameError::BodyLimit);
        }
        Ok(messages)
    }

    /// The `Content-Length` of the header at the start of the buffer, which is consumed; `None`
    /// while the header is incomplete.
    fn header(&mut self) -> Result<Option<usize>, FrameError> {
        let Some(at) = self.buffer.windows(4).position(|w| w == b"\r\n\r\n") else {
            if self.buffer.len() > HEADER_LIMIT {
                return Err(FrameError::HeaderLimit);
            }
            return Ok(None);
        };
        if at + 4 > HEADER_LIMIT {
            return Err(FrameError::HeaderLimit);
        }
        // Blank lines between messages are skipped, not an empty header.
        let blanks = self.buffer.len() - self.buffer.trim_ascii_start().len();
        if blanks > 0 {
            self.buffer.drain(..blanks);
            return self.header();
        }
        let header = &self.buffer[..at];
        if !header.is_ascii() {
            return Err(FrameError::InvalidHeader);
        }
        let text = std::str::from_utf8(header).map_err(|_| FrameError::InvalidHeader)?;
        let length = content_length(text)?;
        self.buffer.drain(..at + 4);
        Ok(Some(length))
    }

    pub fn finish(&self) -> Result<(), FrameError> {
        if self.buffer.is_empty() && self.expected.is_none() {
            Ok(())
        } else {
            Err(FrameError::Incomplete)
        }
    }
}

/// The body length a header declares, once and as plain digits. A `Content-Type` may only name
/// UTF-8.
fn content_length(header: &str) -> Result<usize, FrameError> {
    let mut length = None;
    for line in header.split("\r\n") {
        let (name, value) = line.split_once(':').ok_or(FrameError::InvalidHeader)?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("Content-Length") {
            let digits = !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit());
            if length.is_some() || !digits {
                return Err(FrameError::InvalidLength);
            }
            length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| FrameError::InvalidLength)?,
            );
        } else if name.eq_ignore_ascii_case("Content-Type") && !utf8_charset(value) {
            return Err(FrameError::UnsupportedEncoding);
        }
    }
    length.ok_or(FrameError::InvalidLength)
}

/// The `charset` options of a `Content-Type` value name UTF-8, if any.
fn utf8_charset(content_type: &str) -> bool {
    content_type.split(';').skip(1).all(|option| {
        option.trim().split_once('=').is_none_or(|(key, encoding)| {
            !key.eq_ignore_ascii_case("charset")
                || encoding.trim_matches('"').eq_ignore_ascii_case("utf-8")
                || encoding.trim_matches('"').eq_ignore_ascii_case("utf8")
        })
    })
}

pub fn encode(message: &str) -> Vec<u8> {
    let mut output = format!("Content-Length: {}\r\n\r\n", message.len()).into_bytes();
    output.extend_from_slice(message.as_bytes());
    output
}
