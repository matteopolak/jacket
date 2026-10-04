# Third-party native libraries

## What it is

The stdlib binds some C libraries that no system ships (`stb_image`, `stb_image_write`,
`stb_image_resize`, `stb_vorbis`, and `rpmalloc` 1.4.5 built with first-class heaps). `tools/build_native_libs.py` builds them from pinned, hash-checked
sources into `artifacts/native-libs/<os>-<arch>/`, and `jaic` searches that directory when it resolves a
library name, both for foreign calls at compile time / under `jaic run` and when linking `jaic build`
output. Without it, programs that use those modules check fine but cannot call into them or link.

## How it works

- `tools/native-libs.json` pins each source (repository, revision, sha256 per file) and gives each
  library a one-line C translation unit (`#define STB_IMAGE_IMPLEMENTATION` + `#include`).
- The tool downloads the files into `artifacts/native-libs/sources/` (verifying hashes), compiles each
  unit once with `cc -O2 -fPIC`, and writes `lib<name>.a` and `lib<name>.dylib`/`.so` (macOS dylibs get
  an `@rpath/` install name).
- `jaic-cli` (`native_lib_dirs`) passes the directory to `jaic::interp::set_library_dirs` at startup.
  - Interpreter (`interp/native.rs` `Library::open`): `<dir>/lib<name>.<dylib|so>` is tried before the
    system's search, with a leading `lib` in the Jai name stripped.
  - Linker (`jaic-llvm` `library_args`): `<dir>/lib<name>.a` is linked by path, so executables are
    self-contained.

```bash
python3 tools/build_native_libs.py              # all libraries
python3 tools/build_native_libs.py stb_image    # just one
```

## Slang (sgpu)

The sgpu examples (`corpus/upstream/roeyb1--sgpu`) link `modules/slang/mac/libslang`, `Vulkan_With_VMA/libs/mac/libvulkan` and `Vulkan_With_VMA/mac/VkMemAlloc`, which upstream ships as prebuilt binaries that we do not download. `tools/build_slang.py` produces them:

- Clones `shader-slang/slang` at the pinned tag (`TAG`, currently `v2025.24.2`, with submodules) into `artifacts/thirdparty/slang`, builds a Release shared library with CMake (tests, examples, gfx, slangd, slangi, replayer, CUDA/DXIL off; glslang on), and installs `libslang.dylib` plus the modules Slang `dlopen`s beside it (`libslang-glslang-*`, `libslang-glsl-module-*`) into `modules/slang/mac/`, ad-hoc signed.
- Compiles `vk_mem_alloc.cpp` from the module's own sources into `libVkMemAlloc.a` / `libVkMemAlloc_DEBUG.a`, and copies Homebrew's Vulkan loader to `libs/mac/libvulkan.dylib`.
- Idempotent: reuses the clone/build directory and skips installed outputs (`--force` reinstalls). Paths resolve through the git common dir, so it works from a worktree and writes into the main checkout's `artifacts/` and `corpus/`.

Then build and run the examples (shaders and `sample.png` come from `tools/fetch_upstreams.py`; stb libraries from `build_native_libs.py`):

```bash
brew install cmake ninja molten-vk vulkan-loader vulkan-validationlayers
python3 tools/build_slang.py
python3 tools/build_native_libs.py
cd corpus/upstream/roeyb1--sgpu/examples && jaic build build.jai
export VK_ICD_FILENAMES=/opt/homebrew/etc/vulkan/icd.d/MoltenVK_icd.json \
       VK_LAYER_PATH=/opt/homebrew/share/vulkan/explicit_layer.d \
       DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/vulkan-validationlayers/lib
cd bin && ./02_compute          # run from bin/: shader paths are ../shaders/...
```

Examples request `VK_LAYER_KHRONOS_validation` (hence the layers). `04_mesh_shaders` needs `VK_EXT_mesh_shader`, which MoltenVK does not provide. `JAIC_NATIVE_LIBS` must point at the `build_native_libs.py` output when building from a worktree (the default is relative to the stdlib).

## Vk-Engine (ImGui, VMA, JoltC)

`tools/build_vk_engine_libs.py` builds the C++ libraries Vk-Engine's `Build.jai` expects under `Modules/<M>/Libs/MacOS/` (`libImGui.dylib`, `libVkMemAlloc.a`, `libJoltC.dylib`, the last from JoltC + Jolt Physics v5.6.0 through CMake). Details and options: [Vk-Engine](../native/vk-engine.md).

## How to change it

- New library: add its sources under `sources` (compute sha256 with `shasum -a 256`) and an entry under
  `libraries` whose `code` is a C unit that compiles the implementation. The name must match the Jai
  `#system_library` name (without `lib`).
- Libraries a project builds itself (focus-editor's `LightweightRenderingView` via its `build.jai`) are
  not listed here; `tools/fetch_upstreams.py` fetches C-family sources (`NATIVE_SOURCE_SUFFIXES`) for
  that, never prebuilt binaries.

## Configuration

- `JAIC_NATIVE_LIBS`: path list replacing the default directory
  (`<stdlib>/../artifacts/native-libs/<os>-<arch>`, `os` in `macos`/`linux`, `arch` in `arm64`/`x64`).

## Dependencies

`cc` and `ar` on the host; network access to `raw.githubusercontent.com` on first build.
