# Third-party native libraries

## What it is

The stdlib binds some third-party libraries that no system ships: `stb_image`, `stb_image_write`,
`stb_image_resize`, `stb_vorbis`, `lz4`, `meshoptimizer`, `pl_mpeg`, FreeType, Dear ImGui, MojoShader,
`rpmalloc` (built with first-class heaps; macOS and Linux only), mbedTLS (for [`Extensions/Tls`](../specs/stdlib/README.md)) and, on Windows, SDL2.
`tools/build_native_libs.py` builds them from pinned, hash-checked
sources into `artifacts/native-libs/<os>-<arch>/`. `jaic` searches that directory when it resolves a
library name, both for foreign calls at compile time or under `jaic -interp` and when linking `jaic`
output. Without it, programs that use those modules type-check but can't call into them or link.
Release archives ship the libraries for their platform in `artifacts/native-libs/<platform>/`, built by
`release.yml` from the same pinned sources, so an installed toolchain runs and builds Simp, GetRect and
Sound_Player programs, and the modules for every library below, with no setup. Each source's licence
files go to `licenses/<source>/` beside the libraries (the pinned manifest names them in `license`).

## What ships where

Every row is built by `build_native_libs.py` into `artifacts/native-libs/<platform>/` and into the release
archive of the same name. "static" is `lib<name>.a` / `<name>.lib` (what `jaic` links, so executables
carry the library), "shared" the `.dylib` / `.so` / `.dll` that `jaic -interp` loads.

| Library | macOS arm64/x64 | Linux x64/arm64 | Windows x64/arm64 (MSVC) | MinGW cross (`windows-*-mingw`) |
|---|---|---|---|---|
| `stb_image`, `stb_image_write`, `stb_image_resize`, `stb_vorbis`, `lz4`, `meshoptimizer`, `pl_mpeg` | built | built | built | built (static) |
| FreeType (`freetype`) | built (was the system's) | built (was the system's) | built | built (static) |
| MojoShader (`mojoshader`) | built | built | built | built (static) |
| Dear ImGui (`imgui`, C++) | built | built | built | not built (see below) |
| `rpmalloc` | built | built | not built | not built |
| mbedTLS (`mbedtls`) | built | built | built | built (static) |
| SDL2 (`SDL2`) | system copy | system copy | official prebuilt release, x64 only | official prebuilt release, x64 only |
| wgpu-native | downloaded (`tools/fetch_wgpu_native.py`), static and shared | same | same | not shipped |

Not shipped, from the system or not available: SDL2 on macOS, Linux and Windows arm64 (arm64
MinGW included), `nvtt` and `thekla_baker` (proprietary binaries nobody redistributes), libcurl, Vulkan, GL / OpenGL / EGL, Metal and the other
Apple frameworks, X11, ALSA, and the Windows system libraries. Those are the operating system's, or the
user's to install; `jaic` finds them the usual way.

## How it works

- `tools/native-libs.json` pins each source: a GitHub repository and revision with a sha256 per
  file, or an `archive` URL (`.tar.gz` or `.zip`) with its sha256 and top-level directory, and the
  `license` files to copy out of it (a path, or `{"file", "head", "as"}` when the licence is only
  the head of a file, as in `pl_mpeg.h`). Each library has a one-line C translation unit (`code`:
  `#define STB_IMAGE_IMPLEMENTATION` + `#include`) or a list of `units` with `include` directories,
  `defines` and extra compiler `cflags`, and optionally a `support` unit of glue code, a `local` directory of the
  repository with `local_units` compiled with it (mbedTLS's configuration and glue), and a `lemon`
  parser generation step; `platforms` limits it to some OSes (`["windows"]`), `except` names
  platform directories it is not built for, and `prebuilt` installs files of an official binary
  release per platform instead of compiling (SDL2). A run that builds every library then checks that
  each file `shipped_files` expects is in the output, so a library or licence cannot go missing from a
  package unnoticed.
- The tool downloads the files into `artifacts/native-libs/sources/` (verifying hashes), compiles each
  unit once with `cc -O2 -fPIC`, and writes `lib<name>.a` and `lib<name>.dylib`/`.so` (macOS dylibs get
  an `@rpath/` install name).
- On Windows it compiles with `clang --target=<x86_64|aarch64>-pc-windows-msvc -fms-runtime-lib=static -fms-omit-default-lib`
  (static-CRT code that names no runtime library, so the archive links into executables with either
  runtime: Clang's driver links `libcmt`, `link.exe` gets `/DEFAULTLIB:msvcrt`; code built for the DLL runtime
  references `__imp_fopen` and the like, which MSVC's `link.exe` cannot resolve against `libcmt`) and writes `<name>.lib` (`llvm-lib`) and `<name>.dll`, whose
  exports are every external symbol of the objects (`llvm-nm` into a `.def` file). `--platform
  windows-arm64` picks the CPU explicitly; `tools/stdlib_runtime.py` passes its `--platform`, since
  the Python on an arm64 runner may be an x64 build.
- `jaic-cli` (`native_lib_dirs`) passes the directory to `jaic::interp::set_library_dirs` at startup.
  - Interpreter (`interp/native.rs` `Library::open`): `<dir>/lib<name>.<dylib|so>` (Windows:
    `<dir>/<name>.dll`) is tried before the system's search, with a leading `lib` in the Jai name
    stripped.
  - Linker (`jaic-llvm` `library_args`): `<dir>/lib<name>.a` (Windows MSVC: `<dir>/<name>.lib`) is
    linked by path, so executables are self-contained. Cross builds (`-os windows` from macOS or
    Linux) ignore the directory.

- MinGW cross builds (`--platform windows-x64-mingw` or `windows-arm64-mingw`, on Linux or macOS):
  the same sources compiled with `<x86_64|aarch64>-w64-mingw32-gcc`/`-g++` into `lib<name>.a`
  (static only, no DLL) under `artifacts/native-libs/windows-<cpu>-mingw/`. `jaic -os windows`
  links them from the directories in `JAIC_CROSS_LIBS` (`library_args` in `jaic-llvm`); it never uses
  the host's native-libs directory for a cross build, whose archives are for the host.
  `tools/windows_cross.py build --stdlib` builds and sets it, as `windows-native.yml`'s cross jobs run it.
- FreeType, Dear ImGui and MojoShader come from pinned source archives (FreeType from its repository
  revision, ImGui from the 1.89.6 tag the bindings were written for, MojoShader from the last revision
  before upstream removed its HLSL compiler, which `MOJOSHADER_compile` and the assembler still need).
  FreeType is the same source on every platform, with its own copies of zlib for gzip fonts and
  no PNG, bzip2 or HarfBuzz support. The shipped FreeType wins over the system's: `Library::open` tries
  the native-libs directory before the system's search (and Homebrew's directory), and `library_args`
  links `lib<name>.a` from it before falling back to `-lfreetype`, so an executable built on macOS or
  Linux no longer depends on Homebrew's or the distribution's FreeType being installed.
- lz4 comes from its official release archive and meshoptimizer and pl_mpeg from pinned GitHub
  revisions (`pl_mpeg.h` is one file with an implementation define). meshoptimizer is C++: its
  units compile with `-fno-exceptions -fno-rtti -fno-threadsafe-statics`, and the manifest's
  `support` unit supplies `operator new`/`delete` over `malloc`, so neither the archive nor the
  shared library needs a C++ runtime library.

### C++ libraries (meshoptimizer, Dear ImGui)

Both are compiled with `-fno-exceptions -fno-rtti -fno-threadsafe-statics -std=c++11`, so neither the
static archive nor the shared library needs libstdc++ or libc++, on any platform. meshoptimizer's
`support` unit supplies `operator new` and `delete` over `malloc`; ImGui allocates with its own
allocator and needs none. The script checks this on macOS and Linux (`check_no_cxx_runtime`) and stops
the build if an object still needs an `operator new`, `__cxa_*`, `std::` or guard-variable symbol, so a
future upstream change cannot silently add a runtime dependency. If one ever must, link `-lc++` or
`-lstdc++` from the module (`#system_library "c++"`) rather than from `jaic`.

The ImGui bindings (`stdlib/ImGui/unix.jai`, `windows.jai`) call C++ symbols directly by their
decorated names: Itanium names (`_ZN5ImGui8NewFrameEv`) on macOS and Linux and MSVC names
(`?NewFrame@ImGui@@YAXXZ`) on Windows. There is no C wrapper (cimgui-style), so the library is plain
upstream Dear ImGui (`imgui.cpp`, `imgui_draw.cpp`, `imgui_tables.cpp`, `imgui_widgets.cpp`,
`imgui_demo.cpp`, default `imconfig.h`, 16-bit `ImWchar` and indices, the layout `CreateContext` checks
with `DebugCheckVersionAndDataLayout`), and every name the bindings use was checked against the built
library. Consequences:

- Windows (MSVC) builds it with Clang for the MSVC ABI, and the DLL's `.def` file exports the decorated
  names too (`cxx_exports` in the manifest; C libraries keep only plain names). A `support` unit names
  `user32`, `imm32`, `shell32` and `gdi32` with `#pragma comment(lib, ...)`, so a program that links
  `imgui.lib` needs no extra flags.
- MinGW cross builds (`windows-*-mingw`) do not build it (`except` in the manifest): GCC and Clang for
  MinGW emit Itanium names, which `windows.jai` does not name. A MinGW program that imports ImGui
  type-checks and fails to link. Fixing that means giving `windows.jai` an Itanium variant for MinGW (or a
  C wrapper) first.
- `jaic` links nothing else for it: the archive is self-contained on every platform it is built for.

### MojoShader

`mojoshader` is C, built with the OpenGL, D3D-bytecode, GLSL and ARB1 profiles and effect support. The
HLSL compiler needs a parser that Lemon generates from `mojoshader_parser_hlsl.lemon`; the manifest's
`lemon` entry makes the script compile `misc/lemon.c` for the build machine (also when cross-building),
run it in a scratch directory and add that directory to the include path. The Direct3D 11, SDL GPU,
Metal and SPIR-V profiles are off (`SUPPORT_PROFILE_*=0`), as the bindings do not use them. `cflags` turn
off the C diagnostics that newer compilers promote to errors in this code. At this revision
`MOJOSHADER_compile` stops at an intermediate form that the library prints to standard output and
returns no assembly; the bindings and upstream both behave that way.

### mbedTLS

`mbedtls` is the TLS library behind [`Extensions/Tls`](../specs/stdlib/README.md): mbedTLS 3.6.7 (the 3.6 LTS; Apache-2.0 OR GPL-2.0-or-later, `LICENSE` ships under `licenses/mbedtls/`) from its release tarball (pinned URL and sha256), built as one library from 107 of the 109 `library/*.c` units: crypto, X.509 and TLS together, so the Jai side has one `#system_library "mbedtls"`. Left out are `net_sockets.c` (the Io module does the sockets) and `psa_its_file.c` (persistent keys). The library's own sources are not edited. What is ours lives in the repository:

- `tools/native/mbedtls/jai_mbedtls_config.h`, given to the library with `MBEDTLS_USER_CONFIG_FILE` (a `defines` entry in the manifest) on top of the defaults, which already give TLS 1.2 and 1.3, ALPN, SNI, PSA, X25519, P-256, P-384, ECDSA, RSA, AES-GCM, ChaCha20-Poly1305 and PEM. It turns on threading (`MBEDTLS_THREADING_PTHREAD`, or `MBEDTLS_THREADING_ALT` on Windows), sets the output record buffer to 8 KiB (see [memory](../specs/stdlib/README.md)) and removes the net, timing and persistent-key code; on Windows arm64 it also turns off the ARM crypto instruction paths, which need compiler support that is not checked there.
- `tools/native/mbedtls/threading_alt.h` and `jai_mbedtls.c`: the Windows mutex (`CRITICAL_SECTION`), starting the library once per process, allocating its opaque objects, reading the system's root certificates on Unix, and a few field reads the Jai bindings cannot do.

Two entries of the manifest make this possible: `local` is a directory of the repository that is added to the include path, and `local_units` are files in it that are compiled with the library. MinGW cross builds compile it the same way with `x86_64-w64-mingw32-gcc`; a program that links it on Windows also needs `bcrypt` (named by the module with `#library,system,link_always`). The Windows (clang for MSVC, and MinGW) and macOS builds have not been run here; CI builds them.

### SDL2

SDL2 is not built from source here. It has 100+ source files spread over platform backends (Cocoa and
Objective-C on macOS, X11, Wayland, KMS, ALSA, PulseAudio and more on Linux, each optional and found by
`configure`), and a static archive would also need those backends' frameworks and libraries named at
link time, which `jaic` has no way to learn from a plain `libSDL2.a`. So:

- **Windows x64 (MSVC and MinGW)**: the official `SDL2-devel-2.32.10` release (pinned URL and sha256 in
  the manifest, kept apart for the VC and MinGW archives) is unpacked and its files are copied:
  `SDL2.lib` (import library) and `SDL2.dll` for MSVC, `libSDL2.a` (the release's `libSDL2.dll.a` import
  library under the name `jaic` looks up) and `SDL2.dll` for MinGW. The release's static MinGW `libSDL2.a`
  is not used because it needs a dozen Windows system libraries named at link time. Programs that use
  SDL2 therefore need `SDL2.dll` next to the executable (or on `PATH`); `jaic -interp` loads it from the
  native-libs directory. `stdlib_runtime.py` and `windows_cross.py` copy it beside the test executables.
- **Windows arm64**: the official release has no arm64 binaries, so SDL2 is not shipped there.
- **macOS and Linux**: the system's SDL2 (`brew install sdl2`, `libsdl2-dev`), as before. CI installs it.

```bash
python3 tools/build_native_libs.py              # all libraries
python3 tools/build_native_libs.py stb_image    # just one (an explicit list skips the completeness check)
python tools/build_native_libs.py --platform windows-arm64   # Windows: needs clang, llvm-lib, llvm-nm
python3 tools/build_native_libs.py --out jai-linux-x64/artifacts/native-libs/linux-x64   # into a package
```

The output goes to the main checkout's `artifacts/` even when run from a git worktree (resolved through
the git common dir), so worktrees share one build. `tools/jaic-sweep.py` builds any missing library
before it starts and sets `JAIC_NATIVE_LIBS` to that directory, so sweeps from any checkout or worktree
can call and link these libraries without setup. `--out DIR` writes the libraries somewhere else (the
downloaded sources stay in the shared cache); `release.yml` uses it to build them into each archive, with
`MACOSX_DEPLOYMENT_TARGET` set to the oldest macOS `jaic` supports (11.0 on arm64, 10.12 on Intel) so the
dylibs load there too. macOS dylibs are linked with `-headerpad_max_install_names`, so Homebrew can rewrite
their install names when it installs an archive.

## Slang (sgpu)

The sgpu examples (`corpus/upstream/roeyb1--sgpu`) link `modules/slang/mac/libslang`, `Vulkan_With_VMA/libs/mac/libvulkan` and `Vulkan_With_VMA/mac/VkMemAlloc`, which upstream ships as prebuilt binaries that we do not download. `tools/build_slang.py` produces them:

- Clones `shader-slang/slang` at the pinned `TAG`, with submodules, into `artifacts/thirdparty/slang`, builds a Release shared library with CMake (tests, examples, gfx, slangd, slangi, replayer, CUDA/DXIL off; glslang on), and installs `libslang.dylib` plus the modules Slang `dlopen`s beside it (`libslang-glslang-*`, `libslang-glsl-module-*`) into `modules/slang/mac/`, ad-hoc signed.
- Compiles `vk_mem_alloc.cpp` from the module's own sources into `libVkMemAlloc.a` / `libVkMemAlloc_DEBUG.a`, and copies Homebrew's Vulkan loader to `libs/mac/libvulkan.dylib`.
- Idempotent: reuses the clone/build directory and skips installed outputs (`--force` reinstalls). Paths resolve through the git common dir, so it works from a worktree and writes into the main checkout's `artifacts/` and `corpus/`.

Then build and run the examples (shaders and `sample.png` come from `tools/fetch_upstreams.py`; stb libraries from `build_native_libs.py`):

```bash
brew install cmake ninja molten-vk vulkan-loader vulkan-validationlayers
python3 tools/build_slang.py
python3 tools/build_native_libs.py
cd corpus/upstream/roeyb1--sgpu/examples && jaic build.jai
export VK_ICD_FILENAMES=/opt/homebrew/etc/vulkan/icd.d/MoltenVK_icd.json \
       VK_LAYER_PATH=/opt/homebrew/share/vulkan/explicit_layer.d \
       DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/vulkan-validationlayers/lib
cd bin && ./02_compute          # run from bin/: shader paths are ../shaders/...
```

Examples request `VK_LAYER_KHRONOS_validation` (hence the layers). `04_mesh_shaders` needs `VK_EXT_mesh_shader`, which MoltenVK does not provide. `JAIC_NATIVE_LIBS` must point at the `build_native_libs.py` output when building from a worktree (the default is relative to the stdlib).

## Vk-Engine (ImGui, VMA, JoltC)

`tools/build_vk_engine_libs.py` builds the C++ libraries Vk-Engine's `Build.jai` expects under `Modules/<M>/Libs/MacOS/` (`libImGui.dylib`, `libVkMemAlloc.a`, `libJoltC.dylib`, the last from JoltC + Jolt Physics v5.6.0 through CMake). Details and options: [Vk-Engine](../native/vk-engine.md).

## How to change it

- New library: add its sources under `sources` (compute sha256 with `shasum -a 256`, and list the
  licence files) and an entry under `libraries` whose `code` is a C unit that compiles the
  implementation, or whose `units` are its source files. The name must match the Jai `#system_library`
  name (without `lib`). Add a runtime test that calls it (`tests/stdlib/<name>-....jai`) and, if it has
  no wasm build or is missing on a platform, lines in `tests/stdlib-runtime-skips.txt`. A C++ library
  must build without the C++ runtime (see above) or its bindings need a link requirement documented
  here.
- Libraries a project builds itself (focus-editor's `LightweightRenderingView` via its `build.jai`) are
  not listed here; `tools/fetch_upstreams.py` fetches C-family sources (`NATIVE_SOURCE_SUFFIXES`) for
  that, never prebuilt binaries.

## Configuration

- `JAIC_NATIVE_LIBS`: path list replacing the default directory
  (`<stdlib>/../artifacts/native-libs/<os>-<arch>`, `os` in `macos`/`linux`/`windows`, `arch` in
  `arm64`/`x64`).

## Dependencies

`cc`, `c++` and `ar` on macOS and Linux (`cc` also compiles Lemon when MojoShader builds); `clang`, `llvm-lib` and `llvm-nm` (the official LLVM release) and the
MSVC libraries on Windows; network access to `raw.githubusercontent.com` and `github.com` on first
build. Release archives include these libraries (`release.yml`'s Package step; the smoke test runs
`tests/native-libs/simp-image.jai` and the FreeType, ImGui and MojoShader runtime tests (plus SDL2's on Windows
x64) from the unpacked archive with `jaic -interp` and as built executables). Archives
up to 0.4.2 did not, so Simp programs stopped with ``foreign procedure `stbi_load` is not available here``
under `jaic -interp` and with a missing `stb_image` library when linking.

## wgpu-native (WebGPU)

wgpu-native is not built here: `tools/fetch_wgpu_native.py [--platform P] [--out DIR] [--force]` downloads the
release zip pinned in `tools/webgpu.json` (sha256 per platform), checks that its bundled `webgpu.yml` is the
revision the bindings were generated from, and puts the static and shared library (`libwgpu_native.a` +
`.dylib`/`.so`, or `wgpu_native.lib` + `.dll`) into the same `artifacts/native-libs/<os>-<arch>/`. Release
archives ship it next to the libraries above, in `artifacts/native-libs/<platform>/` beside `stdlib/`, which is
the default search directory. See [WebGPU](../specs/stdlib/README.md). The release zip has no licence files, so
the script also downloads wgpu-native's `LICENSE.MIT` and `LICENSE.APACHE` from the pinned tag (sha256 in
`tools/webgpu.json`) into `licenses/wgpu-native/`. The static library contains Rust dependencies under their
own licences, which the zip does not list; upstream's repository is the place to find them.
