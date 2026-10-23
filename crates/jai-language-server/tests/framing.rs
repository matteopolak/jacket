use jai_language_server::framing::{FrameDecoder, FrameError, encode};

#[test]
fn fragmented_and_coalesced_frames_count_utf8_bytes() {
    let messages = ["{\"text\":\"🦀é\"}", "{\"next\":2}"];
    let wire = messages
        .iter()
        .flat_map(|message| encode(message))
        .collect::<Vec<_>>();
    let mut decoder = FrameDecoder::new(1024);
    let mut output = vec![];
    for chunk in wire.chunks(3) {
        output.extend(decoder.push(chunk).unwrap());
    }
    decoder.finish().unwrap();
    assert_eq!(output, messages);
}

#[test]
fn invalid_framing_never_accepts_conflicting_lengths_or_encodings() {
    let cases: [(&[u8], FrameError); 3] = [
        (
            b"Content-Length: 1\r\ncontent-length: 1\r\n\r\na",
            FrameError::InvalidLength,
        ),
        (
            b"Content-Length: 1\r\n\
              Content-Type: application/vscode-jsonrpc; charset=utf-16\r\n\r\na",
            FrameError::UnsupportedEncoding,
        ),
        (b"Content-Length: 1\r\n\r\n\xff", FrameError::InvalidUtf8),
    ];
    for (frame, error) in cases {
        assert_eq!(FrameDecoder::new(128).push(frame), Err(error));
    }
    let mut incomplete = FrameDecoder::new(128);
    incomplete.push(b"Content-Length: 3\r\n\r\na").unwrap();
    assert_eq!(incomplete.finish(), Err(FrameError::Incomplete));
}

#[test]
fn an_oversized_message_is_skipped_and_the_stream_stays_in_step() {
    let big = "x".repeat(300);
    let wire = [encode("{\"a\":1}"), encode(&big), encode("{\"b\":2}")].concat();
    let mut decoder = FrameDecoder::new(128);
    let mut output = vec![];
    for chunk in wire.chunks(7) {
        output.extend(decoder.push(chunk).unwrap());
    }
    decoder.finish().unwrap();
    assert_eq!(output, ["{\"a\":1}", "{\"b\":2}"]);
    assert_eq!(decoder.take_oversized(), [300]);
    assert_eq!(decoder.take_oversized(), [] as [usize; 0]);
}

/// A message just under the limit, then the next one, arriving in one chunk that is larger
/// than the room left: both are read.
#[test]
fn a_message_near_the_limit_followed_by_another_in_one_chunk() {
    let frame = |body: &str| format!("Content-Length: {}\r\n\r\n{body}", body.len());
    let big = "x".repeat(20_000);
    let small = "y".repeat(10_000);
    let stream = format!("{}{}", frame(&big), frame(&small));
    let mut decoder = FrameDecoder::new(20_000);
    // The second chunk holds the last 1000 bytes of the first body and all of the second.
    let first_cut = frame(&big).len() - 1000;
    let mut messages = decoder.push(&stream.as_bytes()[..first_cut]).unwrap();
    messages.extend(decoder.push(&stream.as_bytes()[first_cut..]).unwrap());
    assert_eq!(messages, vec![big, small]);
}
