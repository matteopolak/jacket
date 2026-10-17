# SDL3

`#import "Extensions/SDL3"` binds the official SDL3 desktop C API. `SDL` remains SDL2 and is unchanged. This is a new extension, with C names rather than an inferred legacy Jai facade.

The source is **SDL 3.4.18**, the stable release identified by SDL's official release API on 2026-10-14 (published 2026-10-02). The downloaded source's `SDL.h`, `SDL_version.h` and CMake version were checked; the runtime smoke tests require `SDL_GetVersion() == 3004018`.

- Source: <https://github.com/libsdl-org/SDL/releases/download/release-3.4.18/SDL3-3.4.18.tar.gz>
- SHA256: `9c75cf16330322c217dedd2e0609f1124f1b54b8633e763467b4684d0f4334a3` (also matches the official release asset digest).
- License: zlib, [unaltered license](../../stdlib/Extensions/SDL3/LICENSE.txt). [NOTICES.txt](../../stdlib/Extensions/SDL3/NOTICES.txt) retains the public headers' original notices. Generated translations are marked as altered. Release native artifacts ship `licenses/sdl3/LICENSE.txt`, the original HIDAPI and YUV license files, and [embedded third-party notices](../../stdlib/Extensions/SDL3/THIRD-PARTY-NOTICES.txt) extracted from the verified native source. HIDAPI is used under its BSD-style option.
- API documentation: <https://wiki.libsdl.org/SDL3/FrontPage>. The pinned headers, rather than the moving wiki, determine the declarations.

## API scope

The checked-in generator reads **all declarations reached by `SDL.h`, `SDL_main.h` and `SDL_vulkan.h`** for all six supported release targets, including audio, async IO, cameras, clipboard, dialogs, events, filesystem, gamepads, the entire GPU API, haptics, HID, hints, initialization, IO streams, keyboard/keycodes, dynamic loading, locale, logging, Metal window integration, mouse, synchronization, pen, pixels, processes, properties, rectangles, rendering, scancodes, sensors, storage, surfaces, desktop system integration, threads, time, timers, tray, touch, version and video. Each target has roughly 1,250 C procedures plus typed macro wrappers; exact per-target declarations and exclusions are in [inventory.json](../../stdlib/Extensions/SDL3/inventory.json).

C structs/unions retain field order and storage overlap. Opaque C types are empty Jai declarations used behind pointers; do not allocate them by value. Typedefs alias their canonical target without making another nominal type. Enums use full C member names and `using`; the compiler preserves the enum's canonical type name. C `_Bool` is Jai `bool` (one byte), `int` is `s32`, `size_t` is `u64`, and `long`/`wchar_t` use the target's actual C widths. Callbacks use `#c_call`; foreign procedures use the C ABI. C strings use `*u8`; string-valued macros are Jai string constants (pass `.data` to a C procedure). SDL allocation, ownership, threading restrictions and error conventions remain those documented by SDL.

SDL3 event `.type` is a `u32`, as in the C header, while `SDL_EventType` is an enum: assign `cast(u32) SDL_EVENT_USER`. This is intentional ABI fidelity. Backtick declaration syntax preserves C field names such as `SDL_StorageInterface.remove` where the statement parser would otherwise consume a keyword; reflected names and storage offsets are unchanged.

Supported macro wrappers execute the official C macro/inline helper in the native library, with `JAI_SDL3_` symbol names: version arithmetic, FourCC, pixel/audio/colorspace packing and queries, endian conversion, timing/window-position helpers, surface locking, atomic reference counts, UTF-8 conversion helpers, error helpers, thread creation, initialization of the three versioned interface structs, and all public inline helpers reached by the headers. This avoids declaring nonexistent SDL exports such as `SDL_CreateThread`. The wrapper manifest is `tools/native/sdl3/macros.json`. Functions that take `...` use real C variadics (`args: ..Any`), with the compiler's C promotion rules.

Explicitly unsupported surfaces:

- Seven procedures taking platform-specific `va_list`: `SDL_IOvprintf`, `SDL_LogMessageV`, `SDL_SetErrorV`, `SDL_vasprintf`, `SDL_vsnprintf`, `SDL_vsscanf`, `SDL_vswprintf`. Their corresponding variadic procedures are bound. No fake `va_list` layout is supplied.
- C preprocessor configuration/annotations, compiler intrinsics, generic type-dependent or statement/control-flow macros (including assertion-site macros, stack allocation, array-count/zero/copy macros, and compiler/platform detection). Every observed function-like macro is inventoried. `SDL_INIT_INTERFACE` has typed pointer overloads for `SDL_IOStreamInterface`, `SDL_StorageInterface` and `SDL_VirtualJoystickDesc`; they invoke the official C macro.
- `SDL_main` is an application entry point, not a foreign library symbol. SDL's exported `SDL_RunApp`, callback-entry and main-ready procedures are available; the module does not rewrite a Jai program's entry point.
- Platform-specific declarations for Android, iOS, consoles, GDK and WebAssembly, and standalone OpenGL/OpenGL ES/intrinsic headers and the separate `SDL3_test` auxiliary testing library. These are additional graphics/platform APIs rather than SDL3 library exports. Vulkan integration uses the public forward declarations from `SDL_vulkan.h`; import a Vulkan binding for complete Vulkan definitions.

## Regeneration

Only Python's standard library, Clang's C frontend and the hash-checked official archive are needed. No SDL program or reference compiler is executed during generation. The generator supplies a minimal ISO C typedef environment using Clang's target builtin widths, so all six target headers can be parsed without installing six SDKs. It does not replace any SDL declaration. It compares every input header with the verified archive before generating.

```sh
python3 tools/sdl3_native.py --platform macos-arm64   # fetch/check cache and build
python3 tools/sdl3_gen.py --source artifacts/sdl3/SDL3-3.4.18
python3 tools/sdl3_gen.py --source artifacts/sdl3/SDL3-3.4.18 --check
python3 -m unittest discover -s tools -p 'test_sdl3*.py'
```

The generated target files, native inline wrappers, macro wrappers, original notices, inventory and ABI assertions are checked in. Clang evaluates constants and supplies 1,486 size/alignment/member-offset assertions per macOS/Linux target and 1,488 per Windows target, covering aggregates, enums and aliases. The C smoke programs compile the same assertions against the real platform SDK headers; Jai checks the translated layouts. The layout test also forces every typedef and procedure signature to type-check. All targets are little-endian 64-bit; unsupported target selection fails explicitly. A binding version upgrade requires updating the source pin, generator version checks, wrapper inventory, generated files and runtime expectations together.

Generated assets have an exact UTF-8/LF byte contract, enforced by scoped SDL3 Git attributes and explicit byte I/O. `--check` rejects CRLF and any stale byte; it never normalizes a ledger before accepting its hash.

## Native builds and release packaging

`tools/build_native_libs.py SDL3 --platform <platform>` integrates the dedicated `tools/sdl3_native.py` CMake builder with `native-libs.json`. SDL3 uses the durable `artifacts/sdl3/` source/build cache. `--out` writes directly into a release package. An independent invocation supports `--cache DIR --offline --jobs 2`. Cached archives are always SHA256-checked; every extracted source file is compared with the verified archive before reuse. Missing/corrupt offline caches fail without a network request. Extraction rejects absolute/parent paths and links. Downloads never use a mutable source tag or unchecked content.

| Target | Artifacts | Target/toolchain baseline |
|---|---|---|
| macOS x64 | `libSDL3.a`, `libSDL3.dylib`, `libSDL3.0.dylib` | explicit `x86_64`, macOS 10.12, Xcode command-line tools/SDK |
| macOS arm64 | same | explicit `arm64`, macOS 11.0, Xcode tools/SDK |
| Linux x64 | `libSDL3.a`, `libSDL3.so` and SONAME files | native x86-64 Linux runner, distro C toolchain and backend development packages |
| Linux arm64 | same | native arm64 Linux runner; a mismatched host is rejected |
| Windows x64 | `SDL3.lib` (static), `SDL3-import.lib`, `SDL3.dll` | discovered stable VS/MSVC C tools + Windows SDK, explicit `-A x64 -T host=x64`, static CRT |
| Windows arm64 | same | discovered stable native ARM64 VS/MSVC tools + Windows SDK, explicit `-A ARM64 -T host=ARM64`, static CRT |

Both static and shared C headless smoke programs are built through SDL's own CMake targets and run on native matching runners. CMake options retain video, audio, events, gamepads/joysticks, rendering, GPU, haptics and HID, with SDL tests/examples off. Linux explicitly enables X11, Wayland, ALSA, PulseAudio and PipeWire with dynamic backend loading; CI installs the development packages (see `.github/workflows/sdl3.yml`). Runtime machines need their platform drivers/backends for those features, but the headless smoke needs neither a display nor manually installed SDL3. macOS uses Apple's system frameworks; optional CoreHaptics and UniformTypeIdentifiers remain weak-linked even for the static archive, through private source-built jaic workspace linker-option primitives. Windows uses system SDK libraries listed in `link-windows.jai`. SDL's own error is returned when a runtime backend is unavailable.

Windows discovery uses the installed [Microsoft vswhere](https://github.com/microsoft/vswhere/wiki/Examples) with MSBuild and target-specific VC tools component requirements, then matches each actual installation's numeric major version to `cmake -E capabilities`. It chooses the newest complete, launchable compatible stable instance, including Build Tools, and passes its exact `CMAKE_GENERATOR_INSTANCE`. No fixed release-year mapping or developer command prompt is required. VS 2026 needs [CMake 4.2 or newer](https://cmake.org/cmake/help/latest/generator/Visual%20Studio%2018%202026.html); older CMake can select an installed compatible VS 2022 instance, otherwise discovery fails with upgrade/install instructions. Windows release builds require a matching native CPU runner. CMake and the C smoke's architecture macros reject wrong compilers/architectures. Both real static/shared targets are required; the distinct official static and import libraries retain their identities in package metadata.

Windows declarations use the pinned header's native MSVC compiler-feature profile. The undocumented compiler-conditional `_builtin` overflow helpers are exposed only on the Clang/GNU profiles where SDL defines them; the documented portable `SDL_size_add_check_overflow` and `SDL_size_mul_check_overflow` are available everywhere with real native wrappers. Generation compiles every emitted wrapper against its header profile.

`CMAKE_GENERATOR` and `CMAKE_GENERATOR_INSTANCE` may select an installed compatible VS explicitly; an active prompt's `VSINSTALLDIR` selects that instance when no explicit instance is given. `CMAKE_GENERATOR_PLATFORM` must match the target, and `CMAKE_GENERATOR_TOOLSET`, if set, must equal the selected native `host=x64` or `host=ARM64`. Conflicting selections, Ninja and MinGW fail explicitly. When a previous build cache names another generator, instance, architecture or host toolset, only the generated target build directory is removed; verified source caches and packaged outputs are preserved. Offline discovery never downloads a toolchain.

`jaic` resolves the static archive through `JAIC_NATIVE_LIBS` or the packaged `artifacts/native-libs/<platform>` directory. The module names the transitive OS libraries, so linked executables carry SDL3 without needing an SDL3 DLL/dylib/so alongside them. `jaic -interp` opens the packaged unversioned runtime library by absolute path; versioned aliases preserve its install name/SONAME. macOS runtime install names use `@rpath`, and the directory includes the actual `libSDL3.0.dylib`. Windows runtime/import artifacts ship separately from the static archive. The complete official installed SDK now ships under `sdl3/sdk`, including public headers, original library names, relocatable CMake/pkg-config targets and installed licenses. Its actual CMake directory comes from the install manifest (MSVC defaults to `cmake`, Unix to `lib/cmake/SDL3`), preserving metadata's relative layout. Real relocated C/C++ static/shared consumers run in native builds. `sdl3/build.json` records the source, target, options, complete SDK hashes, artifact identities and actual consumer evidence; the package audit validates them.

The existing release workflow builds SDL3 on all six current targets and runs `tools/test_sdl3.py` with the packaged compiler/libraries outside the checkout. It audits every declared runtime symbol and artifact checksum, then compiles/runs all ABI, headless and callback/macro tests.
