#!/usr/bin/env python3
"""Build Slang from source and install it where the sgpu corpus project expects it.

    python3 tools/build_slang.py [--jobs N] [--force]

Clones https://github.com/shader-slang/slang at a pinned tag into
<repo>/artifacts/thirdparty/slang, builds a Release shared library with CMake, and installs
`libslang.dylib` (macOS) / `libslang.so` (Linux) plus the runtime modules Slang dlopens next to it
(glslang, glsl-module) into <sgpu>/modules/slang/<mac|linux>/. Idempotent: reruns reuse the clone and
build directory, and skip the install when the files are already present (use --force to redo it).
"""
import argparse
import glob
import os
import platform
import shutil
import subprocess
import sys

TAG = "v2025.24.2"
URL = "https://github.com/shader-slang/slang"
SGPU = os.path.join("corpus", "upstream", "roeyb1--sgpu")


def run(cmd, **kw):
    print("+", " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, **kw)


def main_checkout():
    # Worktrees share artifacts/ and corpus/ with the main checkout; resolve through the git common dir.
    here = os.path.dirname(os.path.abspath(__file__))
    common = subprocess.run(
        ["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
        cwd=here, capture_output=True, text=True,
    )
    if common.returncode == 0:
        return os.path.dirname(common.stdout.strip())
    return os.path.dirname(here)


def build_vulkan_extras(root, mac, ext, sub):
    """sgpu's Vulkan_With_VMA module expects two prebuilt pieces that upstream ships as binaries:
    `libs/mac/libvulkan` (the loader; copied from Homebrew's vulkan-loader) and `<os>/VkMemAlloc[_DEBUG]`
    (Vulkan Memory Allocator compiled from the module's own vk_mem_alloc.cpp)."""
    mod = os.path.join(root, SGPU, "modules", "Vulkan_With_VMA")
    if mac:
        loader = os.path.realpath("/opt/homebrew/lib/libvulkan.dylib")
        out = os.path.join(mod, "libs", "mac")
        if os.path.exists(loader) and not os.path.exists(os.path.join(out, "libvulkan.dylib")):
            os.makedirs(out, exist_ok=True)
            shutil.copy2(loader, os.path.join(out, "libvulkan.dylib"))
        elif not os.path.exists(loader):
            print("warning: brew install vulkan-loader molten-vk for the Vulkan loader + MoltenVK ICD")
    cxx = os.environ.get("CXX", "c++")
    outdir = os.path.join(mod, sub)
    os.makedirs(outdir, exist_ok=True)
    for name, flags in (("VkMemAlloc", ["-O2"]), ("VkMemAlloc_DEBUG", ["-g", "-O0"])):
        lib = os.path.join(outdir, f"lib{name}.a")
        if os.path.exists(lib):
            continue
        obj = os.path.join(outdir, f"{name}.o")
        run([cxx, "-c", "-x", "c++", "-std=c++17", *flags, "-fno-exceptions", "-DVMA_IMPLEMENTATION",
             "-DNDEBUG", "-I" + os.path.join(mod, "source"),
             os.path.join(mod, "source", "vulkan_memory_allocator", "vk_mem_alloc.cpp"), "-o", obj])
        run(["ar", "rcs", lib, obj])
        os.remove(obj)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=main_checkout(), help="repo checkout holding artifacts/ and corpus/")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--force", action="store_true", help="reinstall even if outputs exist")
    args = ap.parse_args()

    mac = sys.platform == "darwin"
    ext = "dylib" if mac else "so"
    sub = "mac" if mac else "linux"
    src = os.path.join(args.root, "artifacts", "thirdparty", "slang")
    build = os.path.join(src, "build")
    dest = os.path.join(args.root, SGPU, "modules", "slang", sub)
    if not os.path.isdir(os.path.join(args.root, SGPU)):
        sys.exit(f"{SGPU} not found under {args.root}; run tools/fetch_upstreams.py first")

    if not os.path.isdir(os.path.join(src, ".git")):
        os.makedirs(os.path.dirname(src), exist_ok=True)
        run(["git", "clone", "--depth", "1", "--branch", TAG, "--recurse-submodules",
             "--shallow-submodules", "-j8", URL, src])
    else:
        have = subprocess.run(["git", "-C", src, "describe", "--tags", "--exact-match"],
                              capture_output=True, text=True).stdout.strip()
        if have != TAG:
            sys.exit(f"{src} is at {have or 'an unknown revision'}, expected {TAG}; delete it to re-clone")

    build_vulkan_extras(args.root, mac, ext, sub)

    target = os.path.join(dest, f"libslang.{ext}")
    if os.path.exists(target) and not args.force:
        print(f"{target} already installed")
        return

    gen = ["-G", "Ninja"] if shutil.which("ninja") else []
    run(["cmake", "-S", src, "-B", build, *gen,
         "-DCMAKE_BUILD_TYPE=Release",
         "-DSLANG_LIB_TYPE=SHARED",
         "-DSLANG_ENABLE_TESTS=OFF", "-DSLANG_ENABLE_EXAMPLES=OFF",
         "-DSLANG_ENABLE_GFX=OFF", "-DSLANG_ENABLE_SLANGD=OFF", "-DSLANG_ENABLE_SLANGI=OFF",
         "-DSLANG_ENABLE_SLANGRT=OFF", "-DSLANG_ENABLE_REPLAYER=OFF",
         "-DSLANG_ENABLE_DXIL=OFF", "-DSLANG_ENABLE_CUDA=OFF", "-DSLANG_ENABLE_OPTIX=OFF",
         "-DSLANG_ENABLE_NVAPI=OFF", "-DSLANG_ENABLE_AFTERMATH=OFF", "-DSLANG_ENABLE_XLIB=OFF",
         "-DSLANG_SLANG_LLVM_FLAVOR=DISABLE", "-DSLANG_ENABLE_SLANG_GLSLANG=ON",
         "-DSLANG_ENABLE_RELEASE_DEBUG_INFO=OFF"])
    run(["cmake", "--build", build, "-j", str(args.jobs)])

    libdir = os.path.join(build, "Release", "lib")
    if not os.path.isdir(libdir):
        libdir = os.path.join(build, "lib")
    os.makedirs(dest, exist_ok=True)
    wanted = glob.glob(os.path.join(libdir, f"libslang*.{ext}"))
    if not wanted:
        sys.exit(f"no libslang* outputs found in {libdir}")
    compiler = None
    for f in sorted(wanted):
        name = os.path.basename(f)
        if os.path.islink(f):
            f = os.path.realpath(f)
        shutil.copy2(f, os.path.join(dest, name))
        if name.startswith("libslang-compiler") and compiler is None:
            compiler = name
        print("installed", name)
    # sgpu links `mac/libslang`; the compiler library is installed under that name too.
    shutil.copy2(os.path.join(dest, compiler or sorted(os.listdir(dest))[0]), target)
    if mac:
        run(["install_name_tool", "-id", "@rpath/libslang.dylib", target])
        for f in glob.glob(os.path.join(dest, "*.dylib")):
            run(["codesign", "--force", "--sign", "-", f])
    print(f"Slang {TAG} installed in {dest}")


if __name__ == "__main__":
    main()
