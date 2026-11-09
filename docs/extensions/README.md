# Stdlib extensions (`stdlib/Extensions/`)

`stdlib/Extensions/` holds the modules only Jacket has. Import them with the `Extensions/` prefix:

```jai
#import "Extensions/Long_Double";
#import "Extensions/WebGPU";
```

| Module | What it is for | Docs |
| --- | --- | --- |
| `Environment` | reading environment variables, UTF-8 on every platform | [environment](environment.md) |
| `Testing` | `@test` procedures, checks, subtests and a leak-checking allocator, run by `jacket test` | [testing](testing.md) |
| `Fuzz` | deterministic argument generators and procedure fuzz runners | [stdlib fuzzing](../tools/stdlib-fuzzing.md) |
| `Long_Double` | C's `long double` in the target's format, for calling C functions that take or return one | [Long_Double](../language/long-double.md) |
| `Args` | typed, compile-time-generated command-line parser (clap-like); `jaifmt` uses it | [args](args.md) |
| `Regex` | Rust-`regex` syntax and semantics, linear time; `Regex.Static` patterns give typed groups | [regex](regex.md) |
| `Toml` | TOML 1.1 values, parsing and semantic writing; formatter configuration | [toml](toml.md), [typed](toml-typed.md), [performance](toml-performance.md) |
| `Uuid` | RFC 9562 UUID parsing, formatting, v3/v4/v5/v7 generation and Json field handlers | [uuid](uuid.md) |
| `Json` | `to_json`/`from_json` generated from your types and `@json` notes | [json](json.md) |
| `Protobuf` | binary protobuf: types, encoders and decoders generated at compile time from a `.proto` schema | [protobuf](protobuf.md) |
| `Variant` | tagged union with named cases and a `which` tag: `variant`, `set`, `get_pointer`, equality, printing; Json encodes it | [variant](variant.md) |
| `Image` | decode PNG, JPEG, QOI, BMP, PNM and ICO entries; encode PNG, QOI, BMP and PPM; convert, resize, crop, flip, rotate | [image](image.md) |
| `Deflate` | DEFLATE compression with zlib and gzip framing, CRC-32 and Adler-32 | [deflate](deflate.md) |
| `Audio_Stream` | bounded, seekable decoding of WAV (PCM, IMA ADPCM) and Ogg Vorbis files into `s16` frames, with Vorbis tags; `@experimental` | [audio-stream](audio-stream.md) |
| `Vt` | sans-IO streaming DEC ANSI parser with UTF-8, CSI, OSC and DCS callbacks | [vt](vt.md) |
| `Pty` | nonblocking pseudoterminal sessions on macOS and Linux: spawn, read, write, resize, reap; `@experimental` | [pty](pty.md) |
| `Iprof_Trace` | records Iprof zones as nested per-frame timelines and saves/loads them in a small binary format (`.jipt`); `@experimental` | [iprof-trace](iprof-trace.md) |
| `Meow_Portable` | `hash_bytes`: the Meow hash in portable scalar code, same lanes as `meow_hash` on CPUs without AES-NI; `@experimental` | — |
| `Assets` | `load_asset`: a program's files from disk natively, over `fetch` in the playground | [assets](assets.md) |
| `Grpc` | TLS HTTP/2 RPC client/server, all four modes and typed generated services | [gRPC](grpc.md), [independent interop](../tools/grpc-interop.md) |
| `Http` | sans-IO HTTP/1.x codec, plus a fiber server and client | [codec](http-codec.md), [server](http-server.md), [client](http-client.md), [HTTP/2](http2.md) |
| `SIMD` | portable fixed-width vectors | [simd](simd.md) |
| `Hash_Map` | opt-in SwissTable hash map: single-key, 16-slot SIMD group probing | [hash-map](hash-map.md) |
| `Byte_Search` | vectorized `memchr`/`memmem` | [byte-search](byte-search.md) |
| `Jai_Format` | the formatter behind `jaifmt` and the playground's Format button | [jaifmt](../tools/jaifmt.md) |
| `WebGPU` | the standard `webgpu.h` API: wgpu-native natively, the page's WebGPU in the playground | [webgpu](webgpu.md) |
| `Text_Buffer` | piece-table text buffer with lines, undo and redo, grapheme and word movement | [text-buffer](text-buffer.md) |
| `UI` | native windows with our own WebGPU rendering, FreeType text, layout and a retained widget tree declared with `for` containers; headless for tests | [ui](ui.md), [editor example](ui-editor.md) |
| `Desktop` | native menus, file dialogs and IME composition for `Window_Creation` windows on macOS and Windows; `@experimental` | [desktop](desktop.md) |
| `SDL3` | the SDL3 C API (`SDL` stays SDL2) | [sdl3](sdl3.md) |
| `Io` | fibers, channels, TCP, files, deadlines over a completion engine | [io-engine](io-engine.md) |
| `Io_Uring` | Linux io_uring bindings and a `Ring` helper | [io-uring](io-uring.md) |
| `Tls` | TLS 1.2/1.3 on vendored mbedTLS over any `Io.Reader`/`Io.Writer` | [tls](tls.md) |
| `Service` | compile-time services: long-lived worker programs `#run` code asks (pending answers in the editor) | [service](service.md) |
| `Postgres` | PostgreSQL client on fibers: TLS, SCRAM(-PLUS)/MD5 auth, extended and simple queries, text and binary codecs | [postgres](postgres.md) |
| `Web` | static sites: compile-time HTML templates, components, a parallel page writer and link check; `jacket doc`'s theme | [web](web.md) |
| `String_Literal` | the source location of a byte of a string literal's value, for exact compile-time error spans | [string-literal](string-literal.md) |
| `Wasi_Runtime` | C library entry points and `_start` for `-os wasm`; jaic adds it itself | [wasm target](../native/wasm-target.md) |

## How modules are found

`#import "Extensions/Long_Double"` is a module name containing a `/`, joined onto each search directory (`find_module_in`, `crates/jaic/src/sema/modules.rs`). `Extensions/` itself is not searched, so `#import "Long_Double";` fails with a help line naming the right import. An unknown identifier an extension declares gets the import as a quick fix, and `#import "` completion lists the folder.

## How to change it

- New module: add `stdlib/Extensions/<Name>/module.jai`, a row above, and a page here if it needs design notes. Nothing in the compiler lists modules.
- Compiler-backed type (like `Long_Double`): add a name to `jaic_type` in `sema/expr.rs` and export `Name :: #jaic_type name;`.
- Don't use a name official Jai code uses; those belong in `stdlib/`.
- Tests enumerate modules as `Extensions/<Name>` (`tests/compiler/stdlib-targets.txt`).

## Conventions of Io, Tls, Http and Json

New procedures in these modules follow the same rules.

- **Handles by value, stateful objects by pointer.** By value: `Tcp_Stream`, `Tcp_Listener`, `Fiber_Handle`, `Io.Reader`, `Io.Writer`. By pointer: `*Io_File`, `*Tls_Stream`, `*Channel(T)`, `*Client`, `*Parser`, and the like.
- **Type first for free decoders, receiver first for methods.** `from_json(T, text)`, but `Io.read_json(reader, T)`.
- **`close(x, now := false)`.** Waits for what it must; `now = true` closes at once. `defer close(x)` is always right.
- **Value plus error.** `value, error := f(...)`; the error's `kind` is `.NONE` on success and the value is zero on failure. `error_text(error)` gives one log line.
- **Who frees.** Results come from `context.allocator` and are yours. Text from `error_text` is temporary storage.
- **Milliseconds.** Every duration is integer milliseconds; negative (`Io.NO_DEADLINE`) means no limit.
- **Diagnostics.** At most one `<module>_statistics()` per module, for tests only.

Keep helpers below `#scope_module`. Public renames go in the `Breaking changes` table of `CHANGELOG.md`.
