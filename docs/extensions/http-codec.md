# Http: the HTTP/1 codec (`Extensions/Http`)

## What it is

A sans-IO HTTP/1.0 and 1.1 codec: an incremental, strict parser for requests and responses and writers for heads and chunked bodies. It works on byte buffers only, so the [server](http-server.md) and [client](http-client.md) sit on top, tests drive it directly, and it builds for wasm. Strict RFC 9112 framing is the point: ambiguous messages are refused, which stops request smuggling.

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

- `parse(*parser, data) -> consumed, event`. After `HEAD`, `BODY` and `END` call it again with `advance(data, consumed)`; stop at `NEED_MORE`. `HEAD` and `END` can use zero bytes, so loop on the event, not the byte count. `parse_eof` reports a closed connection. `SWITCHED` follows a 101 or a 2xx to CONNECT: the rest is not HTTP.
- The head is copied into a parser-owned buffer, so header strings stay valid until the next message's first byte; body bytes are never copied. Take what you need from `parser.head` before parsing past `END`. `parser_free` releases buffers, `parser_reset` starts a new connection.
- The head is read a line at a time with a vector scan for CR/LF (`scan.jai`, 16 bytes via `Extensions/SIMD`, scalar twins; `http_use_simd` switches, wasm is always scalar). One-byte feeds cost the same as whole feeds.
- Framing (RFC 9112 6.3): no body for HEAD responses, 1xx, 204, 304; chunked; `Content-Length`; a response with neither reads until close (`keep_alive` false); a request with neither has no body. Set `parser.response_to` to the request method for HEAD/CONNECT answers.
- `head.keep_alive` follows 1.1/1.0 defaults. After a `keep_alive = false` message the parser takes no more bytes (`DATA_AFTER_CLOSE`). `head.upgrade` does not stop parsing (a server may ignore `Upgrade`); if you switch protocols, stop calling `parse`.
- `head.expect_continue`: answer with `write_continue(*out)` before reading the body; any other `Expect` is a 417.
- Refusals each have a `Parse_Error_Kind`; `error_status(kind)` maps it to the status to send (400 for syntax, framing and smuggling shapes; 413 body too large; 414 request line too long; 431 headers too large; 417; 501 for codings other than `chunked`; 505). The categories: bad start line, method or target, lone CR/LF, obs-fold, whitespace before the colon, bad header name or value, missing or duplicate `Host`, malformed or duplicate `Content-Length`, `Content-Length` with `Transfer-Encoding`, malformed `Transfer-Encoding`, bad chunk sizes or extensions, forbidden trailers. Accepted although unusual: any method token, leading empty lines, no reason phrase, obs-text in values, leading zeros.
- `Limits` (pass to `parser_init`): `max_line_size` 8192, `max_head_size` 65536, `max_headers` 100, `max_trailer_size` 8192, `max_chunk_extension` 1024, `max_body_size` 16 MiB (-1 none), `require_host` true, `skip_leading_empty_lines` true.
- Writers append to a caller-owned `[..] u8`: `write_response_head(*out, status, headers, body_length, options) -> Body_Kind, ok` and `write_request_head`. A non-negative `body_length` adds `Content-Length`; `-1` adds chunked (1.1) or sends until close (1.0 response). A CR/LF in a header value, or a hand-set `Content-Length`/`Transfer-Encoding`, returns `ok = false` with nothing written. Also `write_chunk`, `write_last_chunk`, `write_continue`, `Date_Cache`/`http_date`, `format_http_date`, `parse_http_date`, `status_reason`. `http_date_now` needs `Basic.current_time_consensus()`, which has no wasm clock, so portable code passes explicit seconds.
- Names: `Method` (plus `OTHER`, spelling in `head.method_text`) and the `Header_Name` enum of about 70 headers. `find_header(head.headers, .CONTENT_TYPE)` or by any-case string; `has_token(value, "upgrade")`.

## How to change it

- **New strictness rule:** header-level rules go in `note_header` (`parser.jai`, runs per header line so bad messages fail early), whole-head rules in `finish_head`. Add the `Parse_Error_Kind` and its status in `types.jai` (`error_status`) and a case in `tests/smuggling.jai`.
- **New well-known header:** add to `Header_Name` and `HEADER_NAMES` in the same order (`#assert` checks the count); raise `HEADER_TABLE_SIZE` if the compile-time hash table crowds.
- **New scan:** write the vector loop in `scan.jai` with a scalar twin; `tests/codec.jai` compares both at every 16-byte boundary.
- **Chunk framing** is a byte-at-a-time state machine in `chunked.jai`; only body data is bulk.
- Gotchas: `parse` must be called again after `HEAD` even with no input left. Head slices die at the next message's first byte.
- **llhttp deviations:** after changing `tools/http_deviations.json` or the vendored fixtures run `python3 tools/gen_http_corpus.py` and commit `tests/data/llhttp_cases.jai`. Each deviation needs a reason; a listed deviation that no longer differs fails the test. Deliberate deviations: any method token, no HTTP/0.9, `CONNECT`/`Upgrade` handling (codec keeps parsing, reports `SWITCHED` after 101), codings other than `chunked` refused with 501, `chunked` not last refused.

## Configuration

`Limits` per parser, `http_use_simd` (global, default true), `Response_Options` (`version`, `reason`, `request_method`, `keep_alive`, `date`, `server`) for writers.

## Dependencies and tests

`Basic`, `String`, `Extensions/SIMD`; nothing from `Io`, so it is pure. Tests in `stdlib/Extensions/Http/tests/`: `llhttp.jai` (229 vendored llhttp fixtures, MIT, `tests/data/llhttp/`), `smuggling.jai`, `chunking.jai` (fixtures fed whole, byte by byte and cut at random, vector vs scalar), `codec.jai`. `python3 tools/http_conformance.py` regenerates and runs them; the `http-conformance.yml` workflow runs it when `stdlib/Extensions/Http/**` or its harness changes. `bench/bench.jai` measures parse throughput.
