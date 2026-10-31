# Crash runtime build

`stdlib/internal/runtime/crash-handler.jai` binds a private C library, `crash-native/portable_crash` (source `runtime.c`), on macOS and Linux. `cargo build -p jaic-cli` compiles it and embeds it in the `jaic` binary; nothing is run by hand and nothing is written to the source tree. Windows has its own DLL path ([Windows](windows.md)) and embeds nothing.

## How it works

- `crates/jaic-cli/build.rs` compiles `runtime.c` once (`CC`, default `cc`) into `libportable_crash.a` and `.dylib`/`.so` under `OUT_DIR`. It reruns only when the C sources change. A compile failure on a supported target fails the build; unsupported or cross targets embed nothing and warn.
- `crash_runtime.rs` embeds the files with `include_bytes!` and a digest. `crates/jaic-cli/src/crash_runtime.rs` unpacks them on first use to `<cache>/crash-runtime/<digest>/` (`JAIC_CACHE_DIR`, else `$XDG_CACHE_HOME/jaic`, else `~/Library/Caches/jaic` or `~/.cache/jaic`) and puts that folder first among native library directories, so a stale copy in `JAIC_NATIVE_LIBS` never shadows it. Files are written then renamed, so concurrent runs are safe.
- A relative `#library "dir/name"` is looked for beside the source, then in the library directories. The static `.a` wins when linking, so executables carry the runtime and need no cache at run time.

The macOS file watcher backend (`stdlib/internal/watching/native/watch.c`) is built and unpacked the same way by `build_watcher` in `build.rs`.

## How to change it

Edit `runtime.c` and rebuild. If the compile recipe changes, change `build.rs` and `tools/portable/build_portable.py runtime` together; `tools/portable/test_portable_crash_build.py` fails if their flags drift. The script is a helper for tools that need the libraries as files and never writes to `stdlib/`.

Gotcha: a stale `libportable_crash.*` left in `stdlib/internal/runtime/crash-native/` by an older checkout is found first and shadows the embedded copy. Delete it.

## Configuration

`CC`, `AR`, `MACOSX_DEPLOYMENT_TARGET` at build time; `JAIC_CACHE_DIR`, `XDG_CACHE_HOME` for the unpack location.

## Dependencies

A C compiler and `ar` at build time, `pthread` and `dl`.
