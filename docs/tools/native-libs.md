# Third-party native libraries

## What it is

The stdlib binds libraries no system ships: `stb_image`, `stb_image_write`, `stb_image_resize`, `stb_vorbis`, `lz4`, `meshoptimizer`, `pl_mpeg`, FreeType, Dear ImGui, MojoShader, the allocator profile libraries `rpmalloc` binds (`binding_rpmalloc_sdk*` and `binding_rpmalloc_callbacks*`, macOS and Linux only), the callback context (`binding_callback_context`), the audio output behind `Sound_Player` (`jai_audio_output`), the Linux X11 clipboard (`portable_clipboard`), xxHash 0.7.0 (`xxhash070`), mbedTLS (for [`Extensions/Tls`](../extensions/tls.md)), SDL3 and, on Windows, SDL2. `tools/native/build_native_libs.py` builds them from pinned, hash-checked sources into `target/native-libs/<os>-<arch>/`. `jaic` searches that directory when resolving a library name, for foreign calls under `-interp` and when linking. Without it, programs using those modules type-check but cannot call or link. Release archives ship the libraries for their platform (built by `release.yml` from the same sources) with licence files under `licenses/<source>/`.

Every other library the stdlib binds says who provides it in an `@native(<kind>)` note on the line that binds it ([below](#what-the-native-notes-declare)); the notes live in the sources, so editing the declarations never invalidates the CI cache of built libraries. `tools/ci/check_native_libs.py` ties it together: every `#library`, `#system_library`, `#foreign_library` and `#foreign_system_library` name under `stdlib/` (not `tests/`) must be built by `libraries` or carry a note, notes of one name must agree, and with `--native DIR` the built, fetched and packaged ones must be in the archive. `tools/native/package_native_libs.py` fills an archive from the same declarations, and `tools/ci/package_release.sh` (run by `release.yml`) calls it.

Not shipped (the OS's or the user's to install): SDL2 on macOS, Linux and Windows arm64, `nvtt`, `thekla_baker`, libcurl, Vulkan, GL/EGL, Metal and Apple frameworks, X11, ALSA, Windows system libraries. wgpu-native is downloaded rather than built (below).

## How it works

```sh
PYTHONPATH=tools python3 tools/native/build_native_libs.py              # all libraries
PYTHONPATH=tools python3 tools/native/build_native_libs.py stb_image    # one (skips the completeness check)
PYTHONPATH=tools python3 tools/native/build_native_libs.py --platform windows-arm64   # needs clang, llvm-lib, llvm-nm
PYTHONPATH=tools python3 tools/native/build_native_libs.py --out pkg/native-libs/linux-x64   # into a package
PYTHONPATH=tools python3 tools/native/build_native_libs.py --missing    # only those not yet in target/native-libs (CI, after restoring its cache)
```

- `tools/native/native-libs.json` pins each source: a GitHub repository and revision with a sha256 per file, or an `archive` URL with sha256 and top-level directory, plus the `license` files to copy. Each library is a one-line C unit (`code`) or a list of `units` with `include`, `defines`, `cflags`; optional `support` glue unit, `local`/`local_units` (repository files compiled with the library), `lemon` parser generation, `platforms`/`except` limits, `prebuilt` (install an official binary release). A full run checks that every file `shipped_files` expects was produced.
- Sources go to `target/native-libs/sources/` (hashes verified). Units compile once with `cc -O2 -fPIC` into `lib<name>.a` and `lib<name>.dylib`/`.so`. On Windows it uses `clang --target=<cpu>-pc-windows-msvc -fms-runtime-lib=static -fms-omit-default-lib` (static-CRT code naming no runtime, so archives link with either runtime) and writes `<name>.lib` plus `<name>.dll` whose exports come from `llvm-nm`.
- MinGW cross builds (`--platform windows-x64-mingw`/`windows-arm64-mingw`) produce static `lib<name>.a` only. `jaic -os windows` links them from `JAIC_CROSS_LIBS` and never uses the host's native-libs for a cross build. `tools/native/windows_cross.py build --stdlib` builds and sets it.
- `jaic-cli` (`native_lib_dirs`) passes the directory to `jaic::interp::set_library_dirs`. The interpreter (`interp/native.rs`, `Library::open`) tries `<dir>/lib<name>.<dylib|so>` (Windows `<name>.dll`) before the system search, stripping a leading `lib`. The linker (`jaic-llvm`, `library_args`) links `<dir>/lib<name>.a` (MSVC `<name>.lib`) by path so executables are self-contained. The shipped FreeType wins over the system's.
- Output goes to the main checkout's `target/` even from a worktree (via the git common dir). `jaic` looks there too: after `<stdlib>/../target/native-libs/<platform>` it tries the main checkout's, found through the worktree's `.git` file (`main_checkout` in `crates/jaic-cli/src/cli.rs`). `tools/differential/jaic_sweep.py` builds missing libraries and sets `JAIC_NATIVE_LIBS`. `release.yml` builds into each archive with `MACOSX_DEPLOYMENT_TARGET` at the oldest supported macOS; macOS dylibs use `-headerpad_max_install_names` so Homebrew can rewrite install names.

Per-library notes:

- **C++ (meshoptimizer, Dear ImGui)** compile with `-fno-exceptions -fno-rtti -fno-threadsafe-statics -std=c++11`, so neither archive needs libstdc++/libc++. meshoptimizer's `support` unit supplies `operator new`/`delete` over `malloc`. `check_no_cxx_runtime` fails the build if an object needs `operator new`, `__cxa_*`, `std::` or guard variables.
- **ImGui** bindings (`stdlib/ImGui/unix.jai`, `windows.jai`) call decorated C++ names directly (Itanium on Unix, MSVC on Windows; no C wrapper), so the library is plain upstream 1.89.6 with default config. Windows exports decorated names (`cxx_exports`). MinGW does not build it (`except`): GCC emits Itanium names that `windows.jai` lacks, so a MinGW program importing ImGui type-checks and fails to link.
- **MojoShader** is built at the last revision with its HLSL compiler; a `lemon` step compiles `misc/lemon.c` for the build machine to generate the parser. D3D11, SDL GPU, Metal and SPIR-V profiles are off. `MOJOSHADER_compile` stops at an intermediate form printed to stdout, as upstream does.
- **mbedTLS** 3.6.x is built as one library (crypto, X.509, TLS) without `net_sockets.c` and `psa_its_file.c`. Ours: `tools/native/mbedtls/jai_mbedtls_config.h` (threading, 8 KiB output records, no net/timing/persistent keys, no ARM crypto paths on Windows arm64) and `jai_mbedtls.c` + `threading_alt.h` (Windows mutex, once-per-process init, opaque-object allocation, Unix root certificates, the verification callback for names, self-signed roots and CRLs, CRL objects, the alert a peer sent). Windows programs also need `bcrypt`. The library sources are not edited.
- **xxhash070** on Windows ARM64 gets a hash-checked build-local fix replacing x86-only `__emulu` with the header's portable multiply branch.
- **SDL3** is built from a pinned source by a CMake builder (`build_native_libs.py SDL3`), see [SDL3](../extensions/sdl3.md).
- **SDL2** is not built (many optional platform backends cannot be described by a plain `libSDL2.a`). Windows x64 copies the official `SDL2-devel` release (`SDL2.lib`+`SDL2.dll`; MinGW's import library under the name `libSDL2.a`), so programs need `SDL2.dll` beside the executable. macOS and Linux use the system SDL2.
- **wgpu-native**: `tools/bindings/fetch_wgpu_native.py [--platform P] [--out DIR] [--force]` downloads the zip pinned in `tools/bindings/webgpu.json`, checks its `webgpu.yml` matches the revision the bindings were generated from, and installs the static and shared library plus the licences. See [WebGPU](../extensions/webgpu.md).
- **Slang for the sgpu examples**: `tools/native/build_slang.py` builds `libslang` (pinned `TAG`, CMake) and `libVkMemAlloc.a` into the upstream project's `modules/` and `libs/mac/`, and copies Homebrew's Vulkan loader. Idempotent; `--force` reinstalls. Examples need MoltenVK and the validation layers via `VK_ICD_FILENAMES`, `VK_LAYER_PATH`, `DYLD_FALLBACK_LIBRARY_PATH`.
- **Vk-Engine**: `tools/native/build_vk_engine_libs.py` builds ImGui, VMA and JoltC for its `Build.jai`.

### What the native notes declare

A binding names its library after the folder and `lib` prefix are dropped (`../internal/developer-tools/clang-bindings/clang` is `clang`, `libm` is `m`). One note per name is enough (the stdlib puts one on every line), and notes of a name must not disagree:

```jai
libm :: #library,system "m"; @native(external)
backend :: #library "native/portable_watch"; @native(embedded)
native_runtime :: #library "crash-native/platform_io_windows_crash"; @native(runtime,windows)
```

`@native(<kind>[,<os>...])` takes one kind; the OS names (`macos`, `linux`, `windows`) limit the archives it applies to. A library `libraries` builds carries no note.

| Kind | Meaning | Packaging |
| --- | --- | --- |
| `embedded` | compiled into `jaic` (`portable_crash`, `portable_watch`, [crash runtime build](../native/crash-runtime-build.md)) | none |
| `fetched` | a pinned, checksum-verified prebuilt download (wgpu-native) | runs its recipe |
| `runtime` | C built into `stdlib/` while packaging (the Windows crash runtime), for the OSes listed | runs its recipe, before the rest |
| `packaged` | built by a script that places several files (Bindings_Generator's helper, `generator_platform`); the archive must hold its static and shared library | runs its recipe |
| `developer` | built by the developer checks (`tools/developer_checks`) next to their sources (`developer_tools_check`, `developer_tools_debug`, `clang`) | none |
| `external` | the operating system's, the SDK's or the user's (system and Apple frameworks, X11, libcurl, GPU APIs, Android, NVIDIA SDKs) | none |

Files written by a generator (`tools/native_bindings/generate.py`, `generate_appkit.py`, `tools/generators/generate_socket_abi.py`, `tools/developer_checks/generate_developer_binding_model.py`) get their notes from the generator, through `note`, `EXTERNAL`, `DEVELOPER` and `external_note` in `tools/native/provided_libs.py`; never add a note to such a file by hand. `tools/generators/check_generated.py` runs `generate_appkit.py --check`, so `stdlib/Objective_C/AppKit/generated.jai` and `module.jai` must match its output. `external_note` leaves the note off a library `native-libs.json` builds.

The recipe of a `fetched`, `runtime` or `packaged` library is its entry in `RECIPES` in `tools/native/provided_libs.py`: a `script` and its `args`, which may use `{platform}`, `{cpu}`, `{stdlib}`, `{out}` and `{llvm_prefix}`. `package_native_libs.py --dry-run` prints the commands for a platform. A note without a recipe fails the check.

## How to change it

- **A library written in this repository** (`jai_audio_output`, `binding_callback_context`, `portable_clipboard`) has `"source": "repo"`: `local` is its directory under the repository, `local_units` the C files, `platform_units` those only one OS compiles (`focus.m`), and `link` the extra arguments per OS (`macos`, `linux`, `windows`) for its shared library. It needs no download or licence entry.
- **rpmalloc** has `"builder": "rpmalloc"`: it calls `tools/native_bindings/build_rpmalloc.py` for every profile in `ci-rpmalloc-profiles.json` and copies only the `lib<profile>.a`/shared libraries. A new profile needs no change here. Windows has no rpmalloc archive in the release.
- **A new binding of a system library** (a framework, a Windows DLL, an OS library): put `@native(external)` after the `;` of its `#library` line. `check_native_libs.py` names the file that binds anything undeclared. `jaifmt` keeps the note on the declaration's line.
- **New library:** add the source under `sources` (`shasum -a 256`, licence files) and an entry under `libraries` whose `code` or `units` compile the implementation. The name must match the Jai `#system_library` name without `lib`. Add a runtime test (`tests/compiler/regressions/<name>-....jai`). A C++ library must build without the C++ runtime or document its link requirement here.
- Libraries a project builds itself are not listed here; `tools/fetch_upstreams.py` fetches C-family sources for that, never prebuilt binaries.

## Configuration

`JAIC_NATIVE_LIBS` replaces the default directory (`<stdlib>/../native-libs/<os>-<arch>` in a release package, else `<stdlib>/../target/native-libs/<os>-<arch>`, os in `macos`/`linux`/`windows`, arch in `arm64`/`x64`). `JAIC_CROSS_LIBS` lists MinGW directories for cross builds.

## Dependencies

`cc`, `c++`, `ar` on macOS and Linux; `clang`, `llvm-lib`, `llvm-nm` and the MSVC libraries on Windows; network access to GitHub on first build. The release smoke test runs `stdlib/Simp/tests/image.jai` and the FreeType, ImGui and MojoShader runtime tests from the unpacked archive.
