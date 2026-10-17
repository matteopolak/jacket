# Crash runtime build

## What it is

`stdlib/internal/runtime/crash-handler.jai` binds a private C library, `crash-native/portable_crash`
(source: `stdlib/internal/runtime/crash-native/runtime.c`), on macOS and Linux. `cargo build -p jaic-cli`
compiles it and embeds it in the `jaic` binary; nothing is run by hand and nothing is written to the
source tree. Windows has its own DLL path ([Windows](windows.md)) and embeds nothing.

## How it works

- `crates/jaic-cli/build.rs` compiles `runtime.c` once (`CC`, default `cc`) and makes
  `libportable_crash.a` and `libportable_crash.{dylib,so}` from that one object in `OUT_DIR`
  (under `target/`). `cargo:rerun-if-changed` on the `crash-native` directory means it is rebuilt only
  when the C sources change. A compile failure on a supported target fails the build; unsupported or
  cross targets embed nothing and print a `cargo:warning`.
- It generates `crash_runtime.rs` (the files via `include_bytes!` and a content digest).
  `crates/jaic-cli/src/crash_runtime.rs` unpacks them on first use to
  `<cache>/crash-runtime/<digest>/` (`JAIC_CACHE_DIR`, else `$XDG_CACHE_HOME/jaic`, else
  `~/Library/Caches/jaic` or `~/.cache/jaic`, else the temp directory) and appends that directory to the
  native library directories. Files are written then renamed, so concurrent runs are safe.
- A relative `#library "dir/name"` is looked for beside the source first; if absent, the interpreter
  (`interp/native.rs`) and the Unix linker (`jaic-llvm` `library_args`) fall back to the library
  directories by file name. Static `.a` wins when linking, so executables carry the runtime and
  need no cache at run time.
- Release archives, the Nix package and plain `cargo build` need no extra packaging: the runtime
  is inside the binary.

## Also embedded: the macOS file watcher backend

`stdlib/internal/watching/native/watch.c` (bound by `watching/mac.jai` as `native/portable_watch`) is built
the same way by `build_watcher` in `build.rs` (a universal arm64+x86_64 dylib linked with CoreServices,
macOS only) and unpacked into the same cache directory. `tools/build_portable_watcher.py` is its helper
twin (`--output-dir`, default `target/native-helpers/macos-watch`).

## How to change it

Edit `runtime.c` and rebuild. If the compile recipe changes, change `build.rs` and
`tools/build_portable_runtime.py` together; `tools/test_portable_crash_build.py` fails if their flags
drift. The Python script is an internal helper for tools that need the libraries and a hash receipt
as files (`tools/upstream_native.py`, `tools/check_portable_crash_init.py`); it takes
`--output-dir` (default `target/native-helpers/<platform>/crash-runtime`) and never writes to `stdlib/`.

Gotchas: a stale `libportable_crash.*` left in `stdlib/internal/runtime/crash-native/` by an older
checkout is found first (source-relative lookup) and shadows the embedded copy; delete it.

## Configuration

`CC`, `AR`, `MACOSX_DEPLOYMENT_TARGET` (build); `JAIC_CACHE_DIR`, `XDG_CACHE_HOME` (unpack location).

## Dependencies

A C compiler and `ar` at build time (already needed for LLVM), `pthread` and `dl`.
