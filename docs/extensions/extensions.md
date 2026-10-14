# Stdlib extensions (`stdlib/Extensions/`)

## What it is

`stdlib/Extensions/` holds the modules only jaic has. Everything else in `stdlib/` is a clean-room version of a module the official Jai distribution ships; these are not, no other Jai compiler knows them, and they may change between jaic releases.

A program imports them by path, so the import itself says the code is jaic-only:

```jai
#import "Extensions/Long_Double";
#import "Extensions/WebGPU";
```

| Module | Import | What it is for | Docs |
| --- | --- | --- | --- |
| `Long_Double` | `#import "Extensions/Long_Double";` | C's `long double` in the target's format (`Long_Double`, `LONG_DOUBLE_IS_WIDE`), for calling C functions that take or return one. `Bindings_Generator` output imports it when it uses the type. | [Long_Double](../language/long-double.md) |
| `Args` | `#import "Extensions/Args";` | A typed, compile-time-generated command-line parser in the spirit of clap: field notes, subcommands, generated help, typo suggestions and shell completions. `jaifmt` uses it. | [Args](args.md) |
| `Regex` | `#import "Extensions/Regex";` | Regular expressions with the syntax and semantics of Rust's `regex` crate: full Unicode, linear-time matching, and `Regex.Static` patterns compiled at compile time whose named groups become typed fields. | [Regex](regex.md) |
| `Json` | `#import "Extensions/Json";` | Typed JSON in the style of serde_json: `to_json`, `from_json` (text, or a stream through `Io.read_json`), `free_json`, with encoders and decoders generated at compile time from your types and `@json` attributes. | [Json](json.md) |
| `Http` | `#import "Extensions/Http";` | A sans-IO HTTP/1.0 and 1.1 codec: an incremental, strict request and response parser (chunked bodies, trailers, pipelining, `100-continue`, smuggling defenses), and writers that pick the framing. No sockets; builds for wasm. | [Http](http-codec.md) |
| `SIMD` | `#import "Extensions/SIMD";` | Portable fixed-width vectors with lane-wise arithmetic, masks, shuffles and byte table lookups; the same results on every target, vector instructions in optimized native builds. | [SIMD](simd.md) |
| `Byte_Search` | `#import "Extensions/Byte_Search";` | Vectorized byte and substring search (the job of `memchr`/`memmem`), linear time even for adversarial needles. | [Byte_Search](byte-search.md) |
| `Jai_Format` | `#import "Extensions/Jai_Format";` | The Jai source formatter behind `jaifmt` and the playground's Format button: text in, text out. | [jaifmt](../tools/jaifmt.md) |
| `WebGPU` | `#import "Extensions/WebGPU";` | The standard WebGPU C API (`webgpu.h`), generated from `webgpu.yml`: wgpu-native natively, the page's WebGPU in the playground. | [WebGPU](webgpu.md) |
| `Io` | `#import "Extensions/Io";` | Cooperative fibers on a per-thread scheduler: `spawn`, `sleep`, `cancel`, bounded channels, wait groups and fiber mutexes, plus fiber TCP, files, name lookups, deadlines and cancellation over a completion engine (io_uring, epoll, kqueue). Not available on wasm; no IO on Windows yet. | [Io](io-fibers.md), [engine](io-engine.md) |
| `Io_Uring` | `#import "Extensions/Io_Uring";` | Linux io_uring: constants and structs generated from the kernel header, `io_uring_setup/enter/register`, and a mapped `Ring` with submit, completion and probe helpers. Linux only. | [Io_Uring](io-uring.md) |
| `Tls` | `#import "Extensions/Tls";` | TLS 1.2 and 1.3 on a vendored mbedTLS: `tls_connect`/`tls_accept` over any `Io.Reader`/`Io.Writer`, a config with PEM certificates, roots and ALPN, system trust (certificate bundles on Linux, Security.framework on macOS, CryptoAPI on Windows). Not available on wasm. | [Tls](tls.md) |
| `Wasi_Runtime` | `#import "Extensions/Wasi_Runtime";` | The C library entry points (threads included, as green threads) and `_start` a wasm64 program needs to run as a WASI command. jaic adds it to `-os wasm` builds itself; a program rarely imports it. | [wasm target](../native/wasm-target.md) |

## How it works

`stdlib/Extensions/` is not a module search directory. `#import "Extensions/Long_Double"` is an ordinary module name containing a `/`: `find_module_in` (`crates/jaic/src/sema/modules.rs`) joins it onto each search directory like any other name, so it resolves to `stdlib/Extensions/Long_Double/module.jai`. Nothing special happens for the folder, and the same path form works for a user's own `modules/Folder/Name`.

Because `Extensions/` is not searched, `#import "Long_Double";` is an unknown module. `Compiler::with_extension_help` turns that error into a fix naming the right import (and for the folder alone, or for the old `Jaic_Extensions`, a pointer to the modules):

```
error: module `Long_Double` not found
help: `Long_Double` is a jaic extension (not official Jai), imported by its path: `#import "Extensions/Long_Double";`
```

An unknown identifier that an extension module declares gets the same import as its help and as jailsp's "Add `#import`" quick fix (`imports_declaring` in `sema/suggestions.rs`, which lists `stdlib/Extensions/*` as `Extensions/Name` after the official modules). In `#import "` completion, jailsp shows `Extensions` as a folder of "jaic extension modules" and the modules inside it as "jaic extension module".

The folder is part of `stdlib/`, so everything that copies or embeds the stdlib carries it: release archives (`cp -R stdlib`), the browser engine's bundled files (`crates/jai-wasm/build.rs`) and `JAIC_STDLIB` overrides.

## How to change it

- New extension module: add `stdlib/Extensions/<Name>/module.jai` (one feature per module, named after it, starting with a comment that says it is a jaic extension), add a row to the table above, and document it. Nothing in the compiler lists the modules: the help, quick fix and completion find it by listing the folder.
- A compiler-backed type (like `Long_Double`): add a name to `jaic_type` in `sema/expr.rs` and export it from the module as `Name :: #jaic_type name;`.
- Never put a module here whose name the public source files uses: those belong in `stdlib/` itself, under the official name.
- Tests that enumerate modules include the folder: `crates/jaic-cli/tests/stdlib_targets.rs` checks each as `Extensions/<Name>` (keys in `tests/stdlib-targets.txt`), `tools/stdlib_coverage.py` keys them the same way in `tests/stdlib-coverage.txt`, and `tools/stdlib_runtime.py`, `tools/jaic-sweep.py` and `tools/jaic-diff.py` run `stdlib/Extensions/*/tests/*.jai` (test ids `Extensions/<Name>/tests/<file>`).

## Configuration

- `jaic::STDLIB_EXTENSIONS_DIR` (`crates/jaic/src/lib.rs`): the folder name, `Extensions`.
- `jaic::build::WASI_RUNTIME_IMPORT`: the import jaic adds for WASI targets.

## Dependencies

- `crates/jaic/src/sema/modules.rs` (resolution and the missing-module help), `crates/jaic/src/sema/suggestions.rs` (import suggestions), `crates/jai-language-server/src/session.rs` (completion).
