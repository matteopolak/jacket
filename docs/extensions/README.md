# Stdlib extensions (`stdlib/Extensions/`)

`stdlib/Extensions/` holds the modules only jaic has. Everything else in `stdlib/` is a clean-room version of a module the official Jai distribution ships; these are not, no other Jai compiler knows them, and they may change between jaic releases. The import path says so:

```jai
#import "Extensions/Long_Double";
#import "Extensions/WebGPU";
```

| Module | What it is for | Docs |
| --- | --- | --- |
| `Long_Double` | C's `long double` in the target's format, for calling C functions that take or return one | [Long_Double](../language/long-double.md) |
| `Args` | typed, compile-time-generated command-line parser (clap-like); `jaifmt` uses it | [args](args.md) |
| `Regex` | Rust-`regex` syntax and semantics, linear time; `Regex.Static` patterns give typed groups | [regex](regex.md) |
| `Json` | `to_json`/`from_json` generated from your types and `@json` notes | [json](json.md) |
| `Http` | sans-IO HTTP/1.x codec, plus a fiber server and client | [codec](http-codec.md), [server](http-server.md), [client](http-client.md) |
| `SIMD` | portable fixed-width vectors | [simd](simd.md) |
| `Byte_Search` | vectorized `memchr`/`memmem` | [byte-search](byte-search.md) |
| `Jai_Format` | the formatter behind `jaifmt` and the playground's Format button | [jaifmt](../tools/jaifmt.md) |
| `WebGPU` | the standard `webgpu.h` API: wgpu-native natively, the page's WebGPU in the playground | [webgpu](webgpu.md) |
| `SDL3` | the SDL3 C API (`SDL` stays SDL2) | [sdl3](sdl3.md) |
| `Io` | fibers, channels, TCP, files, deadlines over a completion engine | [io-engine](io-engine.md) |
| `Io_Uring` | Linux io_uring bindings and a `Ring` helper | [io-uring](io-uring.md) |
| `Tls` | TLS 1.2/1.3 on vendored mbedTLS over any `Io.Reader`/`Io.Writer` | [tls](tls.md) |
| `Wasi_Runtime` | C library entry points and `_start` for `-os wasm`; jaic adds it itself | [wasm target](../native/wasm-target.md) |

## How modules are found

`#import "Extensions/Long_Double"` is an ordinary module name containing a `/`: `find_module_in` (`crates/jaic/src/sema/modules.rs`) joins it onto each search directory, giving `stdlib/Extensions/Long_Double/module.jai`. `Extensions/` itself is not searched, so `#import "Long_Double";` fails with a help line naming the right import (`Compiler::with_extension_help`). An unknown identifier that an extension declares gets the same import as a jailsp quick fix (`imports_declaring` in `sema/suggestions.rs`), and `#import "` completion lists the folder.

The folder is part of `stdlib/`, so release archives, the browser engine's bundled files (`crates/jai-wasm/build.rs`) and `JAIC_STDLIB` overrides carry it.

## How to change it

- New module: add `stdlib/Extensions/<Name>/module.jai` (one feature per module, first comment says it is a jaic extension), a row above, and a doc. Nothing in the compiler lists modules; help, quick fix and completion list the folder.
- Compiler-backed type (like `Long_Double`): add a name to `jaic_type` in `sema/expr.rs` and export `Name :: #jaic_type name;`.
- Never put a module here whose name official Jai code uses; those belong in `stdlib/` under the official name.
- Test lists that enumerate modules include the folder as `Extensions/<Name>` (`tests/compiler/stdlib-targets.txt`, `tools/stdlib_runtime.py`, `tools/jaic-sweep.py`, `tools/jaic-diff.py` run `stdlib/Extensions/*/tests/*.jai`).

Constants: `jaic::STDLIB_EXTENSIONS_DIR` (`Extensions`), `jaic::build::WASI_RUNTIME_IMPORT`.

## Conventions of Io, Tls, Http and Json

These four follow the same rules; new procedures should too.

- **Handles by value, stateful objects by pointer.** By value: `Tcp_Stream`, `Tcp_Listener`, `Fiber_Handle`, `Io.Reader`, `Io.Writer`. By pointer: `*Io_File` (has a position), `*Tls_Stream`, `*Pipe_Stream`, `*Channel(T)`, `*Wait_Group`, `*Fiber_Mutex`, `*Parser`, `*Client`, `*Client_Response` (body is used up as read). Configs are plain structs with defaults.
- **Type first for free decoders, receiver first for methods.** `from_json(T, text)`, `from_query(T, pairs)`; but `Io.read_json(reader, T)`, `read_json(*response, T)`.
- **`close(x, now := false)`.** Waits for what it must; `now = true` closes at once for code that cannot block. `defer close(x)` is always right. (`close(*Channel(T))` never waits and has no `now`.)
- **Value plus error.** `value, error := f(...)`; the error has a `kind` whose `.NONE` is success (`Io_Error`, `Json_Error_Kind`, `Tls_Error_Kind`, `Client_Error_Kind`); on failure the value is zero. `error_text(error)` gives one log line. Shared kind names: `TIMED_OUT`, `CANCELED`, `CLOSED`.
- **Who frees.** Results come from `context.allocator` and are yours: `read_text`/`read_bytes` (`free` the data), `Io.read_all`, `to_json`, anything `from_json` decodes (`free_json(*value)` takes values, `Json_Value` and `Json_Error`). A `Client_Error` that owns strings is released by `free_error`. `Tls_Error` holds text inline. Message text from `error_text`/`detail` is temporary storage. `error_text(Client_Error)` may return the error's own `detail`: use it before `free_error`.
- **Milliseconds and `NO_DEADLINE`.** Every duration is integer milliseconds. `Io.with_deadline(ms, #code { ... })` is the one deadline primitive; negative (`Io.NO_DEADLINE`) means no limit. `Server_Config` timeouts treat 0 or less as no limit, except `shutdown_timeout`.
- **Diagnostics.** At most one `<module>_statistics()` per module (`Io.io_statistics()`, `Http.http_statistics()`), for tests only.

Keep helpers below `#scope_module` so they stay out of the public surface. Renames of public names go in the `Breaking changes` table of `CHANGELOG.md`.
