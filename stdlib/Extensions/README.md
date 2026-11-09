# jaic extensions

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

The modules in this folder exist only in jaic, not in the official Jai distribution. Import them by
their path, so the import says the code is jaic-only:

| Import | What it is for |
| --- | --- |
| `#import "Extensions/Time";` | API declarations for civil time, exact instants, zones and calendar spans ([docs](../../docs/extensions/time.md)) |
| `#import "Extensions/Archive";` | ZIP/ZIP64 and TAR codecs with bounded decoding and safe extraction ([docs](../../docs/extensions/archive.md)) |
| `#import "Extensions/Long_Double";` | C's `long double` in the target's format, for C functions that take or return one |
| `#import "Extensions/Args";` | A typed command-line parser generated at compile time (clap-style: notes, subcommands, help, completions) |
| `#import "Extensions/Regex";` | Regular expressions like Rust's `regex` crate: Unicode, linear time, and patterns compiled at compile time with typed groups |
| `#import "Extensions/Uuid";` | RFC 9562 UUIDs: strict text forms, v3/v4/v5/v7, hashing and Json field handlers ([docs](../../docs/extensions/uuid.md)) |
| `#import "Extensions/Gltf";` | glTF/GLB typed codecs, bounded accessor reads and core validation ([docs](../../docs/extensions/gltf.md)) |
| `#import "Extensions/Csv";` | Streaming CSV records, bounded writing and compile-time typed rows ([docs](../../docs/extensions/csv.md)) |
| `#import "Extensions/Json";` | Typed JSON like Rust's serde_json: encoders and decoders generated from your types at compile time, `@json` attributes, strict parsing |
| `#import "Extensions/Vt";` | Streaming DEC ANSI terminal byte parser; [UTF-8, CSI, OSC and DCS callbacks](../../docs/extensions/vt.md), without a screen |
| `#import "Extensions/Variant";` | A tagged union with named cases, like a Rust enum (`Variant(struct { email: string; phone: string; })`, `Variant(Leaf, *Branch)`, a `void` case for "nothing", `.which`, `set`, `get_pointer`); Json encodes and decodes it |
| `#import "Extensions/WebSocket";` | RFC 6455 framing, permessage-deflate and Http upgrades on Io fibers ([docs](../../docs/extensions/websocket.md)) |
| `#import "Extensions/Http";` | HTTP/1.1: a strict sans-IO codec (builds for wasm), plus on fibers a router-based server whose handlers declare typed extractors checked against the route at compile time, and a client with a connection pool, both over TLS for HTTPS |
| `#import "Extensions/SIMD";` | Portable fixed-width vectors (`u8x16`, `f32x4`, ...) with masks, shuffles and table lookups, the same on every target |
| `#import "Extensions/Byte_Search";` | Vectorized byte and substring search (`memchr`/`memmem`), linear time |
| `#import "Extensions/Hash_Map";` | An opt-in hash map that stores each key once and probes sixteen slots per step (`Map.insert`, `find`, `get`, `entry`, `delete`) |
| `#import "Extensions/Jai_Format";` | The Jai source formatter behind `jaifmt` (text in, text out) |
| `#import "Extensions/Web";` | Compile-time HTML components, reusable page buffers, concurrent static-site writing and internal-link checks |
| `#import "Extensions/WebGPU";` | The standard WebGPU C API (`webgpu.h`): wgpu-native natively, the page's WebGPU in the browser |
| `#import "Extensions/Unicode_Text";` | [Unicode normalization and segmentation](../../docs/extensions/unicode-text.md): NFC/NFD/NFKC/NFKD and grapheme, word and sentence iterators |
| `#import "Extensions/Text_Buffer";` | An editable text buffer: piece table, line index, undo and redo, grapheme and word movement |
| `#import "Extensions/Iprof_Trace";` | Optional Iprof invocation timelines, nested frame recording and portable JIPT trace files |
| `#import "Extensions/UI";` | A native UI library: windows, WebGPU rendering, FreeType text, layout, a retained widget tree and a text editing widget; testable without a window |
| `#import "Extensions/Io";` | Cooperative fibers: a per-thread scheduler, `spawn`/`sleep`/`cancel`, bounded channels, wait groups (not on wasm) |
| `#import "Extensions/Io_Uring";` | Linux io_uring: kernel constants and structs generated from the UAPI header, the three system calls, and a mapped `Ring` (Linux only) |
| `#import "Extensions/Tls";` | TLS 1.2 and 1.3 on a vendored mbedTLS: client and server handshakes over any `Io` reader/writer pair, system or private roots, ALPN; used by Http for HTTPS (not on wasm) |
| `#import "Extensions/Service";` | Compile-time services: long-lived programs that compile-time code asks questions (a database connection kept open between compiles), answered without blocking the editor |
| `#import "Extensions/Postgres";` | PostgreSQL client on Io fibers: TLS with SCRAM-SHA-256(-PLUS), MD5 and cleartext auth, Unix sockets, extended and simple queries, structured server errors, text and binary codecs (not on wasm) |
| `#import "Extensions/String_Literal";` | The source location of a byte of a string literal's value, so compile-time checks of queries or patterns can point at the exact character |
| `#import "Extensions/Wasi_Runtime";` | The C library and `_start` a wasm64 program needs as a WASI command (added by `jaic -os wasm`) |

They may change between jaic releases. Details: `docs/extensions/README.md` in the jaic repository.
