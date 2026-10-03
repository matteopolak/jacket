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
    for (frame,error) in [(b"Content-Length: 1\r\ncontent-length: 1\r\n\r\na".as_slice(),FrameError::InvalidLength),
        (b"Content-Length: 999\r\n\r\n",FrameError::BodyLimit),
        (b"Content-Length: 1\r\nContent-Type: application/vscode-jsonrpc; charset=utf-16\r\n\r\na",FrameError::UnsupportedEncoding),
        (b"Content-Length: 1\r\n\r\n\xff",FrameError::InvalidUtf8)] {
        assert_eq!(FrameDecoder::new(128).push(frame),Err(error));
    }
    let mut incomplete = FrameDecoder::new(128);
    incomplete.push(b"Content-Length: 3\r\n\r\na").unwrap();
    assert_eq!(incomplete.finish(), Err(FrameError::Incomplete));
}
