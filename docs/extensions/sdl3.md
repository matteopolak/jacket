# SDL3

`#import "Extensions/SDL3"` binds the SDL3 desktop C API with C names. `SDL` (SDL2) is unchanged. Targets are macOS, Linux and Windows on x64 and arm64. The SDL version is pinned in `tools/bindings/sdl3_gen.py`; the smoke tests check `SDL_GetVersion()` against it. Licences are in `stdlib/Extensions/SDL3/`.

## How it works

- `tools/bindings/sdl3_gen.py` reads every declaration reached by `SDL.h`, `SDL_main.h` and `SDL_vulkan.h` for each target and writes `<os>-<cpu>.jai`. Structs keep field order, enums use full C member names, callbacks are `#c_call`, C strings are `*u8`, variadics use `args: ..Any`.
- Event `.type` is a `u32` while `SDL_EventType` is an enum: write `cast(u32) SDL_EVENT_USER`.
- Macros and inline helpers go through `JAI_SDL3_` wrappers built into the native library (`tools/native/sdl3/macros.json`).
- Not bound: `va_list` procedures, preprocessor macros, `SDL_main` (`SDL_RunApp` is available), mobile/console/WebAssembly declarations, `SDL3_test`. Vulkan has forward declarations only.
- Clang computes constants and emits size/alignment/offset `#assert`s per target.
- Native libraries: `PYTHONPATH=tools python3 tools/native/build_native_libs.py SDL3 --platform <platform>` (CMake builder `tools/bindings/sdl3_native.py`). `jaic` finds the static archive via `JAIC_NATIVE_LIBS` or `target/native-libs/<platform>`; `link-<os>.jai` names the OS libraries. `jaic -interp` loads the shared library by absolute path.
- Windows builds need Visual Studio C tools; set `CMAKE_GENERATOR*` to choose a toolchain.

## How to change it

- Upgrade SDL: change the source pin and hash, the generator's version checks, the wrapper manifest and the runtime version expectation together, then regenerate.
- Regenerate: `PYTHONPATH=tools python3 tools/bindings/sdl3_native.py --platform macos-arm64`, then `PYTHONPATH=tools python3 tools/bindings/sdl3_gen.py --source target/sdl3/<unpacked source>`. `--check` fails on a stale file. Generated files are UTF-8 with LF only.
- New macro wrapper: add it to `macros.json` and regenerate.
- Tests: `PYTHONPATH=tools python3 tools/bindings/run_sdl3.py` and `python3 tools/run_tests.py -p 'test_sdl3*.py'`; CI's `sdl3` job builds the libraries per platform.
