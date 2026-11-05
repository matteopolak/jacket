# Third-party native libraries

The stdlib binds libraries no system ships: stb, lz4, meshoptimizer, FreeType, Dear ImGui, MojoShader, mbedTLS, SDL3, xxHash and a few of our own. `tools/native/build_native_libs.py` builds them from pinned, hash-checked sources into `target/native-libs/<os>-<arch>/`. `jaic` searches that directory when resolving a library name, for foreign calls under `-interp` and when linking. Release archives ship the libraries for their platform with licences under `licenses/<source>/`.

```sh
PYTHONPATH=tools python3 tools/native/build_native_libs.py                  # all
PYTHONPATH=tools python3 tools/native/build_native_libs.py stb_image        # one
PYTHONPATH=tools python3 tools/native/build_native_libs.py --platform windows-arm64
PYTHONPATH=tools python3 tools/native/build_native_libs.py --missing        # only what is not built yet
```

Not shipped (the OS's or the user's to install): SDL2 on most platforms, libcurl, Vulkan, GL/EGL, Metal and Apple frameworks, X11, ALSA, Windows system libraries. wgpu-native is downloaded, not built (`tools/bindings/fetch_wgpu_native.py`, see [WebGPU](../extensions/webgpu.md)).

## How it works

- `tools/native/native-libs.json` pins each source (a repository and revision with a sha256 per file, or an archive URL) and lists the licence files. A library is a C unit or a list of `units` with `include`, `defines` and `cflags`; extra fields cover support glue, repo-local files, `platforms`/`except` limits and `prebuilt` downloads.
- Units compile once with `cc -O2 -fPIC` into `lib<name>.a` and `lib<name>.dylib`/`.so`. On Windows it uses clang with the static CRT and writes `<name>.lib` plus `<name>.dll`. MinGW cross builds produce static archives only, linked from `JAIC_CROSS_LIBS`.
- The interpreter loads `<dir>/lib<name>.<dylib|so>` before the system search. The linker links `<dir>/lib<name>.a` by path, so executables are self-contained.
- C++ libraries (meshoptimizer, ImGui) build without exceptions, RTTI or the C++ runtime. `check_no_cxx_runtime` fails the build if an object needs it.
- mbedTLS is one library (crypto, X.509, TLS) with our config and glue in `tools/native/mbedtls/`. SDL3 is built by CMake. SDL2 on Windows x64 is the official release.
- Output goes to the main checkout's `target/` even from a worktree, and `jaic` looks there too.

### `@native` notes

Every other library the stdlib binds says who provides it with a note on the line that binds it:

```jai
libm :: #library,system "m"; @native(external)
native_runtime :: #library "crash-native/platform_io_windows_crash"; @native(runtime,windows)
```

`@native(<kind>[,<os>...])`; the OS names limit which archives it applies to. A library `native-libs.json` builds needs no note.

| Kind | Meaning |
| --- | --- |
| `embedded` | compiled into `jaic` |
| `fetched` | pinned, checksum-verified download (wgpu-native) |
| `runtime` | C built into `stdlib/` while packaging |
| `packaged` | built by a script that places several files |
| `developer` | built by the developer checks next to their sources |
| `external` | the OS's, the SDK's or the user's |

`tools/ci/check_native_libs.py` requires every `#library`/`#system_library`/`#foreign_library` name under `stdlib/` to be built or carry a note, and notes of one name to agree. Files written by generators get their notes from the generator (`tools/native/provided_libs.py`); never add one by hand. The recipe for a `fetched`, `runtime` or `packaged` library is its entry in `RECIPES` there; `package_native_libs.py --dry-run` prints the commands.

The Windows crash provider (`tools/stdlib/build_platform_io_runtime.py`) writes its DLL, `.lib` import or static library, and `.exp` linker metadata under `stdlib/internal/runtime/crash-native/`. That directory ignores these generated outputs so building the provider leaves the source checkout clean. Keep new provider output suffixes in its local `.gitignore`.

## How to change it

- **Bind a system library:** put `@native(external)` after the `;` of its `#library` line. The check names any undeclared file.
- **New library:** add the source under `sources` (with `shasum -a 256` and licence files) and an entry under `libraries`. The name must match the Jai library name without `lib`. Add a runtime test under `tests/compiler/regressions/`.
- **Library written in this repo** (`jai_audio_output`, `portable_clipboard`): `"source": "repo"` with `local` and `local_units`. No download or licence entry.
- **rpmalloc** profiles come from `ci-rpmalloc-profiles.json`; a new profile needs no change here.

## Configuration

`JAIC_NATIVE_LIBS` replaces the default directory. `JAIC_CROSS_LIBS` lists MinGW directories for cross builds.

## Dependencies

`cc`, `c++`, `ar` on macOS and Linux; `clang`, `llvm-lib`, `llvm-nm` on Windows; network access on first build.
