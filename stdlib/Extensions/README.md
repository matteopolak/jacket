# jaic extensions

The modules in this folder exist only in jaic, not in the official Jai distribution. Import them by
their path, so the import says the code is jaic-only:

| Import | What it is for |
| --- | --- |
| `#import "Extensions/Long_Double";` | C's `long double` in the target's format, for C functions that take or return one |
| `#import "Extensions/Args";` | A typed command-line parser generated at compile time (clap-style: notes, subcommands, help, completions) |
| `#import "Extensions/Regex";` | Regular expressions like Rust's `regex` crate: Unicode, linear time, and patterns compiled at compile time with typed groups |
| `#import "Extensions/Json";` | Typed JSON like Rust's serde_json: encoders and decoders generated from your types at compile time, `@json` attributes, strict parsing |
| `#import "Extensions/Http";` | HTTP/1.1: a strict sans-IO codec (builds for wasm), plus on fibers a router-based server whose handlers declare typed extractors checked against the route at compile time, and a client with a connection pool, both over TLS for HTTPS |
| `#import "Extensions/SIMD";` | Portable fixed-width vectors (`u8x16`, `f32x4`, ...) with masks, shuffles and table lookups, the same on every target |
| `#import "Extensions/Byte_Search";` | Vectorized byte and substring search (`memchr`/`memmem`), linear time |
| `#import "Extensions/Jai_Format";` | The Jai source formatter behind `jaifmt` (text in, text out) |
| `#import "Extensions/WebGPU";` | The standard WebGPU C API (`webgpu.h`): wgpu-native natively, the page's WebGPU in the browser |
| `#import "Extensions/Io";` | Cooperative fibers: a per-thread scheduler, `spawn`/`sleep`/`cancel`, bounded channels, wait groups (not on wasm) |
| `#import "Extensions/Io_Uring";` | Linux io_uring: kernel constants and structs generated from the UAPI header, the three system calls, and a mapped `Ring` (Linux only) |
| `#import "Extensions/Tls";` | TLS 1.2 and 1.3 on a vendored mbedTLS: client and server handshakes over any `Io` reader/writer pair, system or private roots, ALPN; used by Http for HTTPS (not on wasm) |
| `#import "Extensions/Wasi_Runtime";` | The C library and `_start` a wasm64 program needs as a WASI command (added by `jaic -os wasm`) |

They may change between jaic releases. Details: `docs/extensions/README.md` in the jaic repository.
