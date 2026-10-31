# Http codec

A sans-IO HTTP/1.0 and 1.1 codec: an incremental, strict parser for requests and responses, and writers for heads and chunked bodies. It works on byte buffers only, so the [server](http-server.md) and [client](http-client.md) sit on top, tests drive it directly, and it builds for wasm. Strict RFC 9112 framing is the point: ambiguous messages are refused, which stops request smuggling.

```jai
parser: Parser;
parser_init(*parser, .REQUEST);

while true {
    bytes := read_from_socket();                    // any size, down to one byte
    used, event := parse(*parser, bytes);
    if event.kind == {
        case .HEAD;      print("% %\n", parser.head.method_text, parser.head.target);
        case .BODY;      collect(event.data);       // a slice of `bytes`, not copied
        case .END;       respond(*parser.head);
        case .NEED_MORE; // read more
        case .ERROR;     reply_error(error_status(parser.error.kind)); break;
    }
}
```

## How it works

- `parse(*parser, data) -> consumed, event`. After `HEAD`, `BODY` and `END`, call again with the rest; stop at `NEED_MORE`. `HEAD` and `END` can consume zero bytes, so loop on the event.
- The head is copied into a parser-owned buffer and stays valid until the next message's first byte. Body bytes are never copied.
- Lines are found with a vector scan for CR/LF (`scan.jai`, with scalar twins; `http_use_simd` switches). One-byte feeds cost the same as whole feeds.
- Framing follows RFC 9112 6.3. Set `parser.response_to` to the request method for HEAD/CONNECT answers. After a `keep_alive = false` message the parser takes no more bytes.
- Each refusal has a `Parse_Error_Kind`; `error_status(kind)` gives the status to send (400, 413, 414, 431, 417, 501, 505).
- `Limits` (passed to `parser_init`) cap line, head, header count, trailer, chunk extension and body sizes (16 MiB body default).
- Writers append to a caller-owned `[..] u8`: `write_response_head`, `write_request_head`, `write_chunk`, `write_last_chunk`, `write_continue`. A CR/LF in a header value, or a hand-set `Content-Length`/`Transfer-Encoding`, returns `ok = false` with nothing written.
- `http_date_now` needs a clock wasm lacks; portable code passes explicit seconds.
- Known headers are a `Header_Name` enum: `find_header(head.headers, .CONTENT_TYPE)`.

## How to change it

- New strictness rule: per-header rules go in `note_header` (`parser.jai`), whole-head rules in `finish_head`. Add the `Parse_Error_Kind` and status in `types.jai` and a case in `tests/smuggling.jai`.
- New well-known header: add to `Header_Name` and `HEADER_NAMES` in the same order; raise `HEADER_TABLE_SIZE` if the hash table crowds.
- New scan: write the vector loop in `scan.jai` with a scalar twin; `tests/codec.jai` compares both.
- Chunk framing is a byte-at-a-time state machine in `chunked.jai`.
- Deliberate differences from the vendored llhttp fixtures live in `tools/corpus/http_deviations.json`, each with a reason. After editing it run `PYTHONPATH=tools python3 tools/corpus/gen_http_corpus.py` and commit `tests/fixtures/llhttp_cases.jai`.
- Tests: `llhttp.jai`, `smuggling.jai`, `chunking.jai`, `codec.jai` in `stdlib/Extensions/Http/tests/`; `PYTHONPATH=tools python3 tools/stdlib/http_conformance.py` runs them, and `http-conformance.yml` does so when the module changes.
