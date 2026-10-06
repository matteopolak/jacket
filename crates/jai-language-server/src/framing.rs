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

pub struct FrameDecoder {
    buffer: Vec<u8>,
    expected: Option<usize>,
    body_limit: usize,
}

const HEADER_LIMIT: usize = 8192;

impl FrameDecoder {
    pub fn new(body_limit: usize) -> Self {
        Self {
            buffer: vec![],
            expected: None,
            body_limit,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, FrameError> {
        let limit = self
            .body_limit
            .checked_add(HEADER_LIMIT)
            .ok_or(FrameError::BodyLimit)?;
        if self
            .buffer
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > limit)
        {
            return Err(FrameError::BodyLimit);
        }
        self.buffer.extend_from_slice(bytes);
        let mut messages = vec![];
        loop {
            if self.expected.is_none() {
                let Some(at) = self.buffer.windows(4).position(|w| w == b"\r\n\r\n") else {
                    if self.buffer.len() > HEADER_LIMIT {
                        return Err(FrameError::HeaderLimit);
                    }
                    break;
                };
                if at + 4 > HEADER_LIMIT {
                    return Err(FrameError::HeaderLimit);
                }
                let header = &self.buffer[..at];
                if !header.is_ascii() {
                    return Err(FrameError::InvalidHeader);
                }
                let text = std::str::from_utf8(header).map_err(|_| FrameError::InvalidHeader)?;
                let mut length = None;
                for line in text.split("\r\n") {
                    let (name, value) = line.split_once(':').ok_or(FrameError::InvalidHeader)?;
                    let value = value.trim();
                    if name.eq_ignore_ascii_case("Content-Length") {
                        if length.is_some()
                            || value.is_empty()
                            || !value.bytes().all(|c| c.is_ascii_digit())
                        {
                            return Err(FrameError::InvalidLength);
                        }
                        let number = value
                            .parse::<usize>()
                            .map_err(|_| FrameError::InvalidLength)?;
                        if number > self.body_limit {
                            return Err(FrameError::BodyLimit);
                        }
                        length = Some(number);
                    } else if name.eq_ignore_ascii_case("Content-Type") {
                        for option in value.split(';').skip(1) {
                            if let Some((key, encoding)) = option.trim().split_once('=')
                                && key.eq_ignore_ascii_case("charset")
                                && !encoding.eq_ignore_ascii_case("utf-8")
                                && !encoding.eq_ignore_ascii_case("utf8")
                            {
                                return Err(FrameError::UnsupportedEncoding);
                            }
                        }
                    }
                }
                self.expected = Some(length.ok_or(FrameError::InvalidLength)?);
                self.buffer.drain(..at + 4);
            }
            let length = self.expected.expect("parsed Content-Length");
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
        Ok(messages)
    }

    pub fn finish(&self) -> Result<(), FrameError> {
        if self.buffer.is_empty() && self.expected.is_none() {
            Ok(())
        } else {
            Err(FrameError::Incomplete)
        }
    }
}

pub fn encode(message: &str) -> Vec<u8> {
    let mut output = format!("Content-Length: {}\r\n\r\n", message.len()).into_bytes();
    output.extend_from_slice(message.as_bytes());
    output
}
