# Stdlib runtime tests

## What it is

Every stdlib runtime test (`tests/stdlib/*.jai`, `stdlib/<Module>/tests/*.jai`, `stdlib/Extensions/<Module>/tests/*.jai` and
`stdlib/tests/**/*.jai`) runs on every CI platform in up to four ways, and a coverage report
says which public stdlib procedures those runs reach, per module, with a ratchet so coverage
cannot drop unnoticed. The point is to catch platform bugs at run time (a missing Windows code
path, an ABI detail of one CPU, a libc function the WASI runtime lacks) rather than only checking
that the stdlib compiles everywhere ([stdlib target check](stdlib-target-check.md) does that).

## How it works

`tools/stdlib_runtime.py` finds the tests itself, so a new test file is picked up without being
registered. Each test runs in the requested modes:

| Mode | Command | Where |
|---|---|---|
| `interp` | `jaic run t.jai` | every host |
| `native` | `jaic build t.jai -o x` then `./x` | every host |
| `wasm-interp` | `jaic run t.jai -os wasm` (the browser playground's sandbox: virtual file system, no threads or native libraries) | macOS, Linux |
| `wasm-native` | `jaic build t.jai -os wasm` then `node tools/wasi_run.mjs x.wasm` (WASI preview 1 with the host's `/` pre-opened: files, a libm and binary128, green threads, but no processes or sockets) | macOS, Linux |

A test passes when it exits with 0; output is not compared (tests `assert` what they check). A
build that writes no executable, or reports `no exported 'main'`, did its work at compile time
and counts as "compile-time only". Tests run in parallel (`-j`, at most 8 by default), each test's
modes one after another, from the test's own directory (`tests/stdlib/`), which is where relative
paths such as `.scratch/<test>/` land (gitignored).

**Skips.** `tests/stdlib-runtime-skips.txt` lists the runs that cannot pass, one per line:

```
<test id> <platforms> <modes> <reason>
simp-window-program windows-arm64* * no OpenGL 3.3 driver (mesa-dist-win has no arm64 build)
```

The test id is the file stem for `tests/stdlib/`, else the path without `.jai`
(`Simp/tests/readback-tests`). Platforms are comma-separated shell patterns over `macos-arm64`,
`macos-x64`, `linux-x64`, `linux-arm64`, `windows-x64`, `windows-arm64` and the MinGW cross builds
`windows-x64-mingw`, `windows-arm64-mingw`; modes are comma-separated or `*`. A skipped run still
runs: with `--strict` (as in CI) a skipped run that passes fails the job, so a line goes away as
soon as its reason does. A line naming a test that does not exist is an error.

**Windows.** The native Windows jobs run the harness directly (`--platform windows-x64` or
`windows-arm64`, modes `interp,native`). The MinGW cross jobs build every stdlib runtime test on
Linux with `tools/windows_cross.py` and run the executables on a Windows runner; its required
programs are the same tests minus the skip-list lines for `windows-<cpu>-mingw` (cross builds run
compile-time code on Linux, so a `#run` that takes Windows-only paths cannot work there).
In `native` mode the harness first builds the third-party libraries for the platform
([native libraries](native-libs.md); FreeType too on Windows) and points `JAIC_NATIVE_LIBS` at
them. The MinGW cross builds link static archives that `tools/windows_cross.py build --stdlib`
cross-builds first (`build_native_libs.py --platform windows-<cpu>-mingw`, with the MinGW-w64 or
llvm-mingw compilers) and hands to `jaic build` as `JAIC_CROSS_LIBS`, so the executables need no
DLL. The window tests need OpenGL 3.3, which the Windows runners lack: the x64 jobs
unpack Mesa's llvmpipe (`tools/install_ci_mesa_windows.sh`, a pinned and checksum-verified
mesa-dist-win release) and put its `opengl32.dll` next to `jaic.exe`, next to each native test
program (`--dlls DIR` copies DIR's DLLs beside every executable before running it) and into the
cross-built programs' directory, with `GALLIUM_DRIVER=llvmpipe`. mesa-dist-win has no arm64
build, so arm64 keeps those skips.

**Coverage.** With `--coverage FILE`, every `interp` and `wasm-interp` run sets `JAIC_COVERAGE`,
which makes the interpreter append `path:line name` for every procedure it executes (also when the
program calls `exit`). `tools/stdlib_coverage.py --record FILE` then lists each module's public
procedures (top-level `name :: (...) {` declarations under `#scope_export`, following `#load`,
skipping `#foreign`, `#compiler`, `#intrinsic` and other bodiless declarations) and counts those
the record names. `#expand` macros are inlined and never "executed", so a macro counts as covered
when a test source mentions its name. Files named for another OS (`windows.jai`, `os/alsa.jai`...)
are left out of a platform's count unless `--all-platforms` is given.

**Ratchet.** `tests/stdlib-coverage.txt` keeps, per platform, the covered count of every module.
`--check` fails when a module covers fewer procedures than its baseline; CI prints the platform's
current section (`--print-baseline`) so a raised baseline can be pasted in. Native runs are not
recorded: native code has no coverage hook, and the interpreter runs the same stdlib code.

**Modules with no coverage entry.** The report counts procedures with a Jai body, so a module
that only declares `#foreign` procedures, types or constants has nothing to count and no entry.
Those modules, and what exercises or blocks them:

| Module | Status |
|---|---|
| `stb_image`, `stb_image_write`, `stb_image_resize` | run by `stb-image-codecs` (PNG/BMP/TGA/JPEG/HDR encode and decode, header queries, resizing, corrupt input); also by the Simp tests. No wasm build |
| `stb_vorbis` | run by the Sound_Player tests |
| `freetype`, `freetype255`, `freetype-2.12.1` | run by the Simp and GetRect tests (system FreeType on macOS and Linux, built on Windows) |
| `rpmalloc` | run by `rpmalloc-allocator` (macOS and Linux; the library is not built for Windows or wasm) |
| `executable_formats`, `generate_c_header`, `Project_Generator` | pure Jai, run by `format-detection-helpers` |
| `Windows`, `Windows_Registry`, `Windows_Resources`, `Windows_Utf8`, `d3d11`, `d3d12`, `d3d_compiler`, `dxc_compiler`, `dxgi`, `debug_info` | Windows-only; the File, Process and Window tests reach the parts the stdlib itself uses on the Windows jobs, the Direct3D and DXC bindings need a GPU driver and the SDK's runtime DLLs that the CI images lack |
| `X11`, `Linux`, `Vulkan` | used by the window, input and clipboard tests on Linux; Vulkan needs a driver and loader the runners lack |
| `Metal`, `Objective_C`, `macos` | used by the window tests on macOS; Metal rendering needs a GPU that virtualised runners do not expose |
| `SDL`, `Gamepad`, `Keymap`, `ImGui`, `Thekla_Atlas`, `Thekla_Baker`, `nvtt`, `nvidia_aftermath`, `telemetry3`, `MojoShader` | bindings to SDKs or libraries that tools/native-libs.json does not build and the runners do not install |
| `lz4` | run by `lz4-roundtrip` (default, fast and HC modes, the compress bound, too-small destinations, truncated and corrupt input, partial decompression); built from the official 1.10.0 release archive |
| `meshoptimizer` | run by `meshoptimizer-indices` (index a shuffled grid, vertex cache, overdraw and fetch optimization, strips, the index and vertex codecs: triangle sets, index ranges and cache statistics are checked); C++ built without a C++ runtime library |
| `pl_mpeg` | run by `pl-mpeg-decode`, which decodes `tests/stdlib/pl-mpeg-clip.mpg`, a 0.4 s clip of a red 32x32 picture and a 440 Hz tone that was made for the test with ffmpeg's synthetic `color` and `sine` sources (no third-party content, so no license to carry) |
| `meow_hash` | would need a pinned source in tools/native-libs.json (a download with a checked hash) to build and link |
| `Curl`, `Mail` | libcurl is a system library whose development files and version vary between the runners, so a `file://` test is not reliable there |
| `Android`, `Toolchains`, `Codex`, `Preload`, `Simple_Package`, `Window_Type`, `GetRect_Common`, `Default_Metaprogram`, `Minimal_Metaprogram` | target-specific, data-only, or metaprograms that other tests drive through the compiler |
| `Runtime_Support_Crash_Handler` | installed by the program entry point; a crash is not something a passing test can provoke |

## Running it locally

Build `jaic` first (`cargo build -p jaic-cli`). The harness defaults to the host platform and the
debug binary in `$CARGO_TARGET_DIR` (or `target/`).

```sh
# macOS and Linux: everything CI runs
python3 tools/stdlib_runtime.py --modes interp,native,wasm-interp,wasm-native --strict \
    --coverage target/stdlib-coverage.txt
python3 tools/stdlib_coverage.py --record target/stdlib-coverage.txt --check

# one test, with full failure output
python3 tools/stdlib_runtime.py --filter file-handles --modes interp,native -v

# what a module still lacks
python3 tools/stdlib_coverage.py --record target/stdlib-coverage.txt --uncovered Process
```

- **Linux**: GUI tests need a display. CI uses `xvfb-run -a -s "-screen 0 1280x1024x24"` with
  Mesa (`libgl1-mesa-dri`), X11, EGL and FreeType development packages, `libxfixes-dev` (the input
  test reads the cursor), `xclip` (clipboard) and `libasound2-dev` with a `~/.asoundrc` that makes
  the default PCM `type null`, so Sound_Player plays without a sound card.
- **macOS**: Simp and GetRect link the system FreeType (`freetype255`), found in Homebrew's library
  directory (`/opt/homebrew/lib`, or `/usr/local/lib` on Intel). The arm64 runner image has it; on
  the Intel one CI runs `brew install freetype`.
- **Windows**: run from a Developer PowerShell (MSVC and the Windows SDK on `PATH`) with
  `--modes interp,native`; the wasm modes are not run there.
- **Native libraries** (stb_image, FreeType, stb_vorbis...): on macOS and Linux the harness builds
  them with `tools/build_native_libs.py` when `JAIC_NATIVE_LIBS` is unset.
- `wasm-native` needs `node` and `wasm-ld`.

## How to change it

**Adding a test.** Drop `tests/stdlib/<name>.jai` (or `stdlib/<Module>/tests/<name>.jai`) with a
`main` that asserts what it checks and exits 0. Make it work in all four modes: compile
OS-specific parts with `#if OS == ...`, and under `OS == .WASM` keep whatever does not need
files, threads, processes or native libraries. Check results, not just that calls return; cover
empty inputs, non-ASCII text and paths, large inputs and failures (missing files must fail
cleanly). Put scratch files under `.scratch/<name>/`. Run it in every mode locally, then
`jaic check t.jai -os windows` and `-os linux` for the other hosts.

**`tests/stdlib/` and the playground.** `tools/check_playground_stdlib.mjs` also runs every
`tests/stdlib/*.jai` in the browser engine. It reads the same skip list: a line whose modes include
`playground` (and whose platforms match `browser`, as `*` does) marks a test that cannot pass
there, and a listed test that passes fails the check as stale. A test that needs what wasm lacks
(sockets, windows, native libraries) is listed for `wasm-interp,wasm-native,playground` rather
than given a `main` that only prints `ok` under `OS == .WASM`.

**When a run cannot pass**, fix the bug if there is one. Otherwise add a skip line whose reason
says what is missing on that platform; never skip a test because it is flaky.

**Raising the baseline.** After adding tests, regenerate the local section with
`stdlib_coverage.py --record ... --update` and paste the other platforms' sections from the CI
logs (`--print-baseline` output of the `stdlib-runtime` and Windows jobs). Lowering a baseline
needs a reason in the commit message (for example, a procedure was removed).

**Gotchas.**
- A two-argument call can pick a different overload than you expect (`Debug.backtrace(frames, 1)`
  is `(trace, skip)`); name or pass the arguments that select the one you mean.
- `jaic run` changes to the main file's directory, as native runs do through the harness.
- Windows `/tmp/x` means `\tmp\x` on the current drive: fine for the File module, but tools that
  resolve paths themselves (clang) need an absolute path.

## Configuration

| Setting | Effect |
|---|---|
| `--jaic PATH` | compiler to test (default: `$CARGO_TARGET_DIR/debug/jaic`) |
| `--modes a,b` | modes to run (default `interp,native`) |
| `--platform NAME` | skip-list platform (default: the host's) |
| `--filter TEXT` | only tests whose id contains TEXT |
| `--coverage FILE` | write the coverage record |
| `--strict` | a listed skip that passes is a failure |
| `--timeout SECONDS` | per run (default 180) |
| `-j N` | tests at once |
| `JAIC_COVERAGE=FILE` | interpreter: append executed procedures to FILE |
| `JAIC_STDLIB_TEST_MODE` | set by the harness for each run (`interp`, `native`, `wasm-interp`, `wasm-native`); a test whose outcome depends on the machine reads it instead of a skip line, which `--strict` would report as stale where the test passes |
| `JAIC_MEMORY_LIMIT` | interpreter memory cap; the harness sets 3G unless set |
| `JAIC_NATIVE_LIBS` | directory of built third-party libraries |

## Dependencies

- `jaic` (interpreter coverage hook in `crates/jaic/src/interp/profile.rs`).
- `node` and `tools/wasi_run.mjs` for `wasm-native` (it names every missing import before running).
- `tools/build_native_libs.py` for native third-party libraries.
- CI: the `stdlib-runtime` job in `.github/workflows/ci.yml` (macOS arm64/x64, Linux x64/arm64)
  and the stdlib steps in `.github/workflows/windows-native.yml` (MSVC x64/arm64, MinGW cross).
