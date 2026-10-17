# SDL3 (`Extensions/SDL3`)

## What it is

`#import "Extensions/SDL3"` binds the official SDL3 desktop C API with C names. `SDL` (SDL2) is unchanged. Targets: macOS, Linux and Windows on x64 and arm64.

Source: SDL 3.4.18 (`SDL3-3.4.18.tar.gz` from the libsdl-org release, SHA256 `9c75cf16330322c217dedd2e0609f1124f1b54b8633e763467b4684d0f4334a3`). The runtime smoke tests require `SDL_GetVersion() == 3004018`. License zlib: `stdlib/Extensions/SDL3/LICENSE.txt`, `NOTICES.txt` (public header notices), `THIRD-PARTY-NOTICES.txt` (embedded HIDAPI and YUV code). Declarations come from the pinned headers; the SDL wiki is the usage reference.

## How it works

- `tools/sdl3_gen.py` reads every declaration reached by `SDL.h`, `SDL_main.h` and `SDL_vulkan.h` for each target (about 1,250 procedures per target plus macro wrappers) and writes `<os>-<cpu>.jai`; exact per-target contents are in `inventory.json`. Structs keep field order and overlap; opaque types are empty declarations used behind pointers; enums use full C member names; `_Bool` is `bool`, `int` is `s32`, `size_t` is `u64`, `long`/`wchar_t` use the target's widths; callbacks are `#c_call`; C strings are `*u8`, string macros are Jai string constants (pass `.data`). Variadics use `args: ..Any`.
- Event `.type` is a `u32` as in C while `SDL_EventType` is an enum: write `cast(u32) SDL_EVENT_USER`. Backtick names keep C fields that are Jai keywords (`SDL_StorageInterface.remove`).
- Macros and inline helpers (version arithmetic, FourCC, pixel/audio packing, endian conversion, refcounts, UTF-8 helpers, thread creation, `SDL_INIT_INTERFACE` overloads) call the official C code through `JAI_SDL3_` wrappers built into the native library (manifest `tools/native/sdl3/macros.json`).
- Not bound: the seven `va_list` procedures (their variadic twins are bound), preprocessor and intrinsic/generic macros, `SDL_main` (the module does not rewrite a Jai entry point; `SDL_RunApp` is available), Android/iOS/console/WebAssembly declarations, standalone OpenGL headers and `SDL3_test`. Vulkan uses the forward declarations from `SDL_vulkan.h`; import a Vulkan binding for full definitions.
- Clang evaluates constants and emits about 1,490 size/alignment/offset `#assert`s per target; the C smoke programs compile the same assertions against real platform headers.
- Native libraries: `python3 tools/build_native_libs.py SDL3 --platform <platform>` (CMake builder `tools/sdl3_native.py`, source and build cache in `target/sdl3/`, archive hash-checked, offline mode with `--cache DIR --offline`). Artifacts: static `libSDL3.a`/`SDL3.lib`, shared `libSDL3.dylib`/`.so`/`SDL3.dll`, import library on Windows, and the installed SDK under `sdl3/sdk`. macOS uses system frameworks (CoreHaptics and UniformTypeIdentifiers weak-linked); Linux enables X11, Wayland, ALSA, PulseAudio and PipeWire with dynamic backend loading (CI installs the dev packages); Windows needs Visual Studio C tools and the Windows SDK (found with vswhere; VS 2026 needs CMake 4.2+; Ninja and MinGW are rejected). `CMAKE_GENERATOR`, `CMAKE_GENERATOR_INSTANCE`, `CMAKE_GENERATOR_PLATFORM`, `CMAKE_GENERATOR_TOOLSET` may select an installed toolchain and must match the target.
- `jaic` finds the static archive through `JAIC_NATIVE_LIBS` or `target/native-libs/<platform>` (`native-libs/<platform>` in a release package); `link-<os>.jai` names the transitive OS libraries, so executables need no SDL3 file beside them. `jaic -interp` loads the shared library by absolute path. A missing runtime backend returns SDL's own error.

## How to change it

- **Upgrade SDL:** update the source pin and hash, the generator's version checks, the wrapper manifest, generated files and the runtime version expectation together, then regenerate and run the checks below.
- **Regenerate:** `python3 tools/sdl3_native.py --platform macos-arm64`, then `python3 tools/sdl3_gen.py --source target/sdl3/SDL3-3.4.18` (add `--check` to fail on a stale file). Needs only Python, Clang's C frontend and the verified archive; nothing from SDL is executed.
- Generated files are UTF-8 with LF only (scoped Git attributes, byte I/O); `--check` rejects CRLF and stale bytes.
- **New macro wrapper:** add it to `tools/native/sdl3/macros.json` and regenerate.

## Configuration

`JAIC_NATIVE_LIBS`, and the CMake variables above for native builds.

## Dependencies and tests

Clang (generation only), CMake and a platform C toolchain (native builds). Tests: `stdlib/Extensions/SDL3/tests/` (`layout*.jai`, `headless.jai`, `callbacks.jai`, `entry.jai`, C layout headers) run with `python3 tools/test_sdl3.py` and `python3 -m unittest discover -s tools -p 'test_sdl3*.py'`; the `sdl3` job in `.github/workflows/ci.yml` builds the native libraries per platform, audits them and runs the generator `--check` (passing `--jaifmt`); `release.yml` builds SDL3 for all six targets and tests the packaged libraries.
