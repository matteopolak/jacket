# Http: the HTTP/1 codec (`Extensions/Http`)

## What it is

`#import "Extensions/Http";` gives a sans-IO HTTP/1.0 and HTTP/1.1 codec: an incremental parser for requests and responses, and writers for heads and chunked bodies. It reads and writes byte buffers and never touches a socket, so the [server](http-server.md) and [client](http-client.md) sit on top of it, tests drive it directly, and it builds for every target including wasm. A jaic extension, not official Jai. Strict RFC 9112 framing is the point: ambiguous messages are refused, which is what stops request smuggling.

```jai
parser: Parser;
parser_init(*parser, .REQUEST);

while true {
    bytes := read_from_socket();                    // any size, down to one byte
    used, event := parse(*parser, bytes);

    if event.kind == {
        case .HEAD;      print("% %\n", parser.head.method_text, parser.head.target);
        case .BODY;      collect(event.data);       // a slice of `bytes`
        case .END;       respond(*parser.head);
        case .NEED_MORE; // read more
        case .ERROR;     reply_error(error_status(parser.error.kind)); break;
    }
}
```

## How it works

**Events.** `parse(*parser, data) -> consumed, event` uses some of `data` and returns one event. After `HEAD`, `BODY` and `END` call it again with `advance(data, consumed)`; stop at `NEED_MORE`, which means all the input was used. `HEAD` and `END` can come with zero bytes used, so loop on the event, not on the byte count. `parse_eof(*parser)` says the connection closed: it ends a body that runs until close, reports `CLOSED` between messages and an `UNEXPECTED_EOF` error inside one.

| Event | Meaning |
| --- | --- |
| `HEAD` | `parser.head` is filled in: method or status, target, version, headers, `body` kind, `keep_alive`, `expect_continue`, `upgrade` |
| `BODY` | `event.data` is a piece of the decoded body, a slice of the caller's input (no copy) |
| `END` | the message is complete; `parser.trailers` has chunked trailers |
| `SWITCHED` | a 101 response, or a 2xx response to CONNECT: the rest of the stream is not HTTP |
| `ERROR` | `parser.error` has a `kind`, a static `message` and the byte offset; the parser stays failed |

**Memory.** The head is copied into a buffer the parser owns, so header strings (`Header_Field.name`, `.value`, `head.target`...) stay valid however the input was chunked, until the next message's first byte is parsed. Body bytes are never copied. Pipelined requests therefore work like this: take what you need from `parser.head` before parsing past `END`. Buffers are reused across messages; `parser_free` releases them, `parser_reset` starts a new connection.

**Lines, not bytes.** The head is read a line at a time: the next CR or LF is found with a vector scan, the line is appended to the buffer and checked when complete, so feeding one byte at a time costs no more than feeding everything. A bad first byte of a line (an obs-fold), a lone CR or LF and an over-long line are refused before the line ends.

**Vector scans.** `scan.jai` has the vector loops (16 bytes at a time with `Extensions/SIMD`'s `u8x16`) with scalar twins: the end of a line (CR or LF), bytes a header value or a target may not hold. `http_use_simd` switches them; wasm always uses the scalar loops. Results are identical (the tests run both).

**Framing** (RFC 9112 6.3), decided when the head is complete:

| Message | Body |
| --- | --- |
| response to HEAD, 1xx, 204, 304, 2xx to CONNECT | none (`Content-Length` is still checked and kept in `head.content_length`) |
| `Transfer-Encoding: chunked` | chunked: sizes, extensions, trailers |
| `Content-Length: n` | n bytes |
| a response with neither | until the connection closes (`keep_alive` is false) |
| a request with neither | none |

Set `parser.response_to` to the method of the request a response answers (needed for HEAD and CONNECT). A 1xx other than 101 is a message of its own (`head.informational`); the final response follows.

**Connection handling.** `head.keep_alive` is the HTTP/1.1 default (on unless `Connection: close`) or the HTTP/1.0 one (off unless `Connection: keep-alive`). After a message with `keep_alive` false the parser takes no more bytes: any that follow are a `DATA_AFTER_CLOSE` error, so stop reading and close after replying. `head.upgrade` is `Connection: upgrade` with an `Upgrade` header; the parser keeps going afterwards, since a server may ignore `Upgrade` (RFC 9110 7.8). When you do switch protocols, stop calling `parse`; the unused bytes of the buffer are yours. Pipelining needs nothing special: after `END` the next message begins in the same input.

**Expect: 100-continue.** `head.expect_continue` is set for an HTTP/1.1 request with a body that sends it. Answer with `write_continue(*out)` (or a final status) before reading the body; the parser itself does not wait. Any other expectation is a 417 error.

### Strictness

What is refused (each gets a `Parse_Error_Kind`; `error_status(kind)` maps it to the status to send):

| Refused | Kind | Status |
| --- | --- | --- |
| an unparsable start line, bytes a target or method may not hold, a fragment, `GET` with an authority or `*` target, `CONNECT` without `host:port` | `BAD_START_LINE`, `BAD_METHOD`, `BAD_TARGET` | 400 |
| a version that is not `HTTP/d.d`; one that is not 1.0 or 1.1 | `BAD_VERSION`; `VERSION_NOT_SUPPORTED` | 400; 505 |
| a lone CR or LF, a line not ending in CRLF | `BAD_LINE_ENDING` | 400 |
| obs-fold (a header line starting with SP or HTAB), whitespace before the colon, a name that is not a token, NUL or other controls or DEL in a value | `OBS_FOLD`, `WHITESPACE_BEFORE_COLON`, `BAD_HEADER_NAME`, `BAD_HEADER_VALUE` | 400 |
| no `Host` (HTTP/1.1, unless `limits.require_host` is off), two `Host`s, userinfo or a comma in it | `MISSING_HOST`, `DUPLICATE_HOST`, `BAD_HEADER_VALUE` | 400 |
| `Content-Length` that is not `1*DIGIT` below 2^63, a list (`5, 5`), a sign, two `Content-Length` lines even when equal | `BAD_CONTENT_LENGTH`, `DUPLICATE_CONTENT_LENGTH` | 400 |
| `Content-Length` together with `Transfer-Encoding` | `CONTENT_LENGTH_WITH_TRANSFER_ENCODING` | 400 |
| `Transfer-Encoding` that is empty, has an empty element, repeats `chunked`, has `chunked` before another coding, does not end in `chunked`, or appears in HTTP/1.0 | `BAD_TRANSFER_ENCODING` | 400 |
| another coding before `chunked` (`gzip, chunked`): the codec only decodes the chunking | `UNSUPPORTED_TRANSFER_ENCODING` | 501 |
| a chunk size with no digits, a sign, `0x`, spaces, junk; an extension that is not `;name[=token\|"quoted"]`; missing CRLFs; more than 63 bits of size | `BAD_CHUNK`, `CHUNK_SIZE_OVERFLOW` | 400 |
| `Content-Length`, `Transfer-Encoding` or `Host` in the trailers, folded or malformed trailers | `BAD_TRAILER` | 400 |
| `Expect` other than `100-continue` | `BAD_EXPECT` | 417 |
| the request line over `max_line_size` | `URI_TOO_LONG` | 414 |
| a header line over `max_line_size`, the head or trailers over their size, over `max_headers` | `HEADERS_TOO_LARGE`, `TOO_MANY_HEADERS` | 431 |
| a body over `max_body_size` (declared or counted) | `BODY_TOO_LARGE` | 413 |
| bytes after a message that ended the connection; the connection closing inside a message | `DATA_AFTER_CLOSE`, `UNEXPECTED_EOF` | 400 |

Accepted although unusual: any method token (the server answers 501 for methods it does not know), empty lines before the start line (RFC 9112 2.2; `skip_leading_empty_lines`), a response status line without reason phrase, obs-text (0x80 and up) in values, leading zeros in lengths and chunk sizes, upper and lower case hex.

`Limits` (set it in `parser_init(*parser, kind, limits)`):

| Field | Default | Refuses with |
| --- | --- | --- |
| `max_line_size` | 8192 | 414 (request line), 431 (a header line) |
| `max_head_size` | 65536 | 431 |
| `max_headers` | 100 | 431 |
| `max_trailer_size` | 8192 | 431 |
| `max_chunk_extension` | 1024 | 400 |
| `max_body_size` | 16 MiB (-1: none) | 413 |
| `require_host` | true | 400 |
| `skip_leading_empty_lines` | true | |

### Writers

All append to a `[..] u8` the caller owns.

- `write_response_head(*out, status, headers, body_length, options) -> Body_Kind, ok` and `write_request_head(*out, method, target, headers, body_length)`. The framing follows from `body_length`: a non-negative length adds `Content-Length` and the body is sent as is; `-1` (unknown) adds `Transfer-Encoding: chunked` on HTTP/1.1, or for an HTTP/1.0 response sends until close. 1xx, 204 and CONNECT 2xx responses get no framing headers, a HEAD response gets the `Content-Length` of the body it does not send. The returned `Body_Kind` says how to send the body. `Response_Options` has `version`, `reason`, `request_method`, `keep_alive` (false adds `Connection: close`), `date` and `server`.
- Header names must be tokens and values free of control bytes; a CR or LF in a value (header injection) makes the call return `ok = false` with nothing written. Setting `Content-Length` or `Transfer-Encoding` by hand is refused too: the writer owns the framing.
- `write_chunk(*out, data)` (an empty chunk is skipped, since a zero-size chunk ends the body), `write_last_chunk(*out, trailers)`, `write_continue(*out)`.
- `Date_Cache` and `http_date(*cache, seconds)` format an explicit seconds value once per second; `format_http_date`, `parse_http_date` (IMF-fixdate only) convert; `status_reason(status)`. These explicit-value date codecs are portable, including WASM.

`http_date_now(*cache)` obtains wall-clock time through `Basic.current_time_consensus()`. That Basic API is declared only for Windows, Linux, Android, and macOS; the reviewed public surface provides no WASM clock ABI or failure contract. Consequently `http_date_now` belongs to the existing native client/server adapter surface, while explicit-seconds formatting and parsing remain available to portable codec users.

### Names

`Method` (`GET`, `HEAD`, `POST`... plus `OTHER`; the spelling is in `head.method_text`) and `Header_Name`, an enum of the ~70 common headers. Parsed headers carry `id`, so code compares enums: `find_header(head.headers, .CONTENT_TYPE)`, or by any-case name `find_header(head.headers, "content-type")`. `has_token(value, "upgrade")` checks comma-separated values. The id table is an open-addressing hash table built at compile time (`header_table`).

## How to change it

- **A new strictness rule:** header-level rules go in `note_header` (`parser.jai`; it runs as each header line arrives, so bad messages are refused early); rules that need the whole head go in `finish_head`. Add the `Parse_Error_Kind` and its status in `types.jai` (`error_status`), a row in the tables above, and a case in `tests/smuggling.jai`.
- **A new well-known header:** add it to `Header_Name` and `HEADER_NAMES` (same order; `#assert` checks the count, and `tests/codec.jai` checks that each name maps back to its enum member). Raise `HEADER_TABLE_SIZE` if the table gets crowded.
- **A new scan:** write the vector loop in `scan.jai` with its scalar twin and run `tests/codec.jai` (it compares both on messages with bytes at every 16-byte boundary).
- **Chunk framing:** `chunked.jai` is a byte-at-a-time state machine (`Parser_State` CHUNK_*, `Ext_State` for extensions); body data is the only part handled in bulk.
- **Gotchas.** Head slices point into the parser's buffer and die at the next message's first byte. `parse` must be called again after `HEAD` even with no input left (the body or `END` is next). `keep_alive = false` stops the parser, by design.
- **llhttp deviations.** Run `python3 tools/gen_http_corpus.py` after changing `tools/http_deviations.json` or the vendored fixtures, and commit `tests/data/llhttp_cases.jai`. A deviation must carry a reason; the test fails on a listed deviation that no longer differs.

## Conformance and tests

`python3 tools/http_conformance.py [--modes interp,native,wasm-interp,wasm-native]` checks the generated corpus is current, runs the tests through `tools/stdlib_runtime.py` and prints the llhttp summary. The GitHub workflow `http-conformance.yml` runs it on Linux and macOS when `stdlib/Extensions/Http/**` or the tools change (and by hand); the same tests also run with all stdlib runtime tests on every push.

| Test | What it checks |
| --- | --- |
| `tests/llhttp.jai` | the 229 request and response fixtures of llhttp (MIT; `tests/data/llhttp/`, commit in `SOURCE.txt`): heads, headers, body bytes, completion and errors must equal llhttp's, except where listed. |
| `tests/smuggling.jai` | 108 messages: CL.TE and TE.CL, TE obfuscations, length spellings, header syntax, line endings, request-line forms, chunk and trailer faults; plus ones that look odd and must be accepted. |
| `tests/chunking.jai` | every llhttp fixture and 24 random mutations of each, fed whole, byte by byte and cut in random places, with the vector and scalar scans: all runs must report the same. |
| `tests/codec.jai` | heads, keep-alive, bodies, Expect, pipelining, responses (HEAD, 204, 304, 1xx, 101, until-close), EOF, limits, writers, dates, name tables. |
| `tests/stdlib/http-codec-roundtrip.jai` | pipelined writer-to-parser round trips at several piece sizes. |

The llhttp fixtures: 229 run (pausing fixtures and ones with inline JavaScript are not applicable), 165 match, 29 differ on purpose, 35 ran with an llhttp leniency flag (`lenient-headers`, `lenient-optional-lf-after-cr`...) and may differ because the codec has no lenient mode. The deliberate deviations and why:

- **Not HTTP / versions:** `RTSP/` and `ICE/` start lines; HTTP/0.9 and request lines without a version (RFC 9112 3).
- **Methods and targets:** llhttp has a fixed method list, the codec accepts any token (RFC 9110 9.1); `M-SEARCH *` (asterisk form is OPTIONS only); a fragment in the target.
- **CONNECT:** llhttp stops after the request as an upgrade; the codec reports the request and any framed body and leaves the tunnel to the server.
- **Upgrade requests and 101:** llhttp fails on any byte after them; the codec keeps parsing requests (a server may ignore `Upgrade`) and reports `SWITCHED` after a 101.
- **Transfer-Encoding:** codings other than `chunked` are refused with 501 as the header is read (llhttp accepts `deflate, chunked` and decodes the chunking); `chunked` not last, or a misspelling like `chunkedchunked`, is refused where RFC 9112 6.1 would read the response until close (a smuggling risk).
- **Timing:** `GET / HTTP/5.6` without its CRLF is refused when the line ends, llhttp refuses at the first bad byte.
- Where llhttp keeps blanks around a header value in the value, the tests compare the trimmed value (RFC 9110 5.5).

http-garden (Docker, differential testing of whole HTTP servers) is optional: `http_conformance.py --garden DIR` detects Docker and a checkout, and points at `tools/http_garden/server.jai`, the echo server to put in its image; the script itself probes that server over TCP (smuggling shapes, limits, SIGINT shutdown) without Docker, see [the server](http-server.md). h2spec and HPACK suites are for the HTTP/2 milestone.

## Performance

`bench/bench.jai` (`jaic stdlib/Extensions/Http/bench/bench.jai -O2 -exe /tmp/http-bench`), Apple silicon, native `-O2`, fed in 4 KiB reads, best of three:

| Stream | scalar scans | vector scans |
| --- | --- | --- |
| browser-like requests (540 bytes, 9 headers) | 637 MB/s, 1.19 M requests/s | 959 MB/s, 1.79 M requests/s |
| requests with large headers (a 3 KB cookie, a bearer token, a long target) | 1178 MB/s, 373 k/s | 7394 MB/s, 2.34 M/s |
| chunked bodies | zero-copy: cost per chunk, not per byte | same |

The vector scans pay where lines are long (value validation and line end search cost 16 bytes per step); short requests are dominated by per-header work (name token check, id lookup).

## Configuration

| Name | Effect |
| --- | --- |
| `Limits` | per parser; see the table above |
| `http_use_simd` (global, default true) | false forces the scalar scan loops; wasm always uses them |
| `Response_Options` | writer options |

## Dependencies

- `Basic`, `String`, `Extensions/SIMD`.
- Nothing from `Io`, `POSIX` or `Socket`: the module is pure and builds for wasm.
- Tests: the vendored llhttp fixtures (MIT, `tests/data/llhttp/LICENSE`) and `tools/gen_http_corpus.py`.
