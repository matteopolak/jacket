#!/usr/bin/env python3
"""Build the native C++ libraries of the Vk-Engine corpus project (macOS / Linux).

    python3 tools/build_vk_engine_libs.py [--jobs N] [--force] [--release] [imgui] [vma] [joltc]

Vk-Engine's `Build.jai` expects, per module, a `Libs/<OS>/` directory next to the module that its
`generate.jai` normally fills by running a C++ toolchain at metaprogram time. Upstream only implements that
for Windows and Linux, so this tool does the same for the host:

  * ImGui    -> Modules/ImGui/Libs/<MacOS|Linux>/libImGui.{dylib,so}   (imgui*.cpp, as `BuildImGui`)
  * VMA      -> Modules/Vulkan/Libs/<...>/libVkMemAlloc.a              (vk_mem_alloc.cpp, as `BuildVulkanMemoryAllocator`)
  * JoltC    -> Jolt-Jai/Libs/<...>/libJoltC.{dylib,so}                (CMake, combined Jolt + JoltC shared library,
                                                                        the options `Build.jai` passes to Jolt-Jai's generate.jai)

Sources come from `tools/fetch_upstreams.py` (Vk-Engine, Jolt-Jai and its JoltC submodule); Jolt Physics itself is
cloned at the tag JoltC's CMakeLists names (v5.6.0) into artifacts/thirdparty/JoltPhysics. Idempotent: existing
outputs are kept unless --force. Paths resolve through the git common dir, so it works from a worktree.
Needs a C++17/20 compiler, CMake and git (`brew install cmake`).
"""
import argparse
import os
import shutil
import subprocess
import sys

JOLT_TAG = "v5.6.0"
JOLT_URL = "https://github.com/jrouwe/JoltPhysics"
VK = os.path.join("corpus", "upstream", "ostef--Vk-Engine")
JOLT_JAI = os.path.join("corpus", "upstream", "ostef--Jolt-Jai")
JOLTC = os.path.join("corpus", "upstream", "ostef--JoltC")


def run(cmd, **kw):
    print("+", " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, **kw)


def main_checkout():
    here = os.path.dirname(os.path.abspath(__file__))
    common = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
                            cwd=here, capture_output=True, text=True)
    return os.path.dirname(common.stdout.strip()) if common.returncode == 0 else os.path.dirname(here)


def build_imgui(root, libdir, ext, cxx, release):
    mod = os.path.join(root, VK, "Modules", "ImGui")
    out = os.path.join(libdir(mod), f"libImGui.{ext}")
    src = os.path.join(mod, "Source")
    # generate.jai copies the engine's imconfig.h over the vendored one before building.
    shutil.copy2(os.path.join(mod, "imconfig.h"), os.path.join(src, "imconfig.h"))
    files = [os.path.join(src, f"{n}.cpp") for n in ("imgui", "imgui_widgets", "imgui_draw", "imgui_tables", "imgui_demo")]
    flags = ["-O2"] if release else ["-g", "-O0"]
    cmd = [cxx, "-w", "-std=c++17", *flags, "-fPIC", "-I" + src, "-shared", *files, "-o", out]
    if ext == "dylib":
        cmd += ["-Wl,-install_name,@rpath/libImGui.dylib"]
    os.makedirs(os.path.dirname(out), exist_ok=True)
    run(cmd)


def build_vma(root, libdir, cxx, release):
    mod = os.path.join(root, VK, "Modules", "Vulkan")
    out = os.path.join(libdir(mod), "libVkMemAlloc.a")
    src = os.path.join(mod, "Source")
    obj = out[:-2] + ".o"
    flags = ["-O2", "-DNDEBUG"] if release else ["-g", "-O0"]
    os.makedirs(os.path.dirname(out), exist_ok=True)
    run([cxx, "-c", "-w", "-std=c++17", "-fPIC", *flags, "-I" + src,
         os.path.join(src, "VulkanMemoryAllocator", "vk_mem_alloc.cpp"), "-o", obj])
    run(["ar", "rcs", out, obj])
    os.remove(obj)


def build_joltc(root, libdir, ext, jobs, release):
    jolt = os.path.join(root, "artifacts", "thirdparty", "JoltPhysics")
    if not os.path.isdir(os.path.join(jolt, ".git")):
        os.makedirs(os.path.dirname(jolt), exist_ok=True)
        run(["git", "clone", "--depth", "1", "--branch", JOLT_TAG, JOLT_URL, jolt])
    build = os.path.join(root, "artifacts", "thirdparty", "joltc-build-" + ("release" if release else "debug"))
    on = lambda b: "ON" if b else "OFF"
    # Mirrors JoltCompileOptions.Default | CombinedSharedLibs (| UseAsserts in Debug) from Jolt-Jai/generate.jai.
    debug = not release
    gen = ["-G", "Ninja"] if shutil.which("ninja") else []
    run(["cmake", "-S", os.path.join(root, JOLTC), "-B", build, *gen,
         "-DFETCHCONTENT_SOURCE_DIR_JOLTPHYSICS=" + jolt,
         "-DFETCHCONTENT_FULLY_DISCONNECTED=ON", "-DJPH_USE_VK=OFF", "-DENABLE_ALL_WARNINGS=OFF",
         "-DCMAKE_BUILD_TYPE=" + ("Debug" if debug else "Release"),
         "-DBUILD_SHARED_LIBS=OFF", "-DJOLTC_BUILD_COMBINED_SHARED_LIBS=ON",
         "-DDOUBLE_PRECISION=OFF", "-DCROSS_PLATFORM_DETERMINISTIC=OFF", "-DFLOATING_POINT_EXCEPTIONS_ENABLED=OFF",
         "-DPROFILER_IN_DEBUG_AND_RELEASE=ON", "-DPROFILER_IN_DISTRIBUTION=OFF", "-DJPH_USE_EXTERNAL_PROFILE=OFF",
         "-DDEBUG_RENDERER_IN_DEBUG_AND_RELEASE=ON", "-DDEBUG_RENDERER_IN_DISTRIBUTION=OFF",
         "-DDISABLE_CUSTOM_ALLOCATOR=OFF", "-DOBJECT_LAYER_BITS=OFF", "-DUSE_ASSERTS=" + on(debug),
         "-DENABLE_OBJECT_STREAM=ON", "-DINTERPROCEDURAL_OPTIMIZATION=OFF",
         "-DCMAKE_POLICY_VERSION_MINIMUM=3.5"])
    run(["cmake", "--build", build, "--target", "JoltC", "-j", str(jobs)])
    built = os.path.join(build, f"libJoltC.{ext}")
    if not os.path.exists(built):
        sys.exit(f"{built} was not produced")
    out = os.path.join(libdir(os.path.join(root, JOLT_JAI)), f"libJoltC.{ext}")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    shutil.copy2(built, out)
    if ext == "dylib":
        run(["install_name_tool", "-id", "@rpath/libJoltC.dylib", out])
        run(["codesign", "--force", "--sign", "-", out])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("libs", nargs="*", help="imgui, vma, joltc (default: all)")
    ap.add_argument("--root", default=main_checkout(), help="repo checkout holding artifacts/ and corpus/")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--force", action="store_true", help="rebuild even if outputs exist")
    ap.add_argument("--release", action="store_true", help="optimized build (upstream default is Debug)")
    args = ap.parse_args()

    mac = sys.platform == "darwin"
    ext = "dylib" if mac else "so"
    sub = "MacOS" if mac else "Linux"
    cxx = os.environ.get("CXX", "c++")
    libdir = lambda mod: os.path.join(mod, "Libs", sub)
    for needed in (VK, JOLT_JAI, JOLTC):
        if not os.path.isdir(os.path.join(args.root, needed)):
            sys.exit(f"{needed} not found under {args.root}; run tools/fetch_upstreams.py first")

    todo = args.libs or ["imgui", "vma", "joltc"]
    paths = {
        "imgui": os.path.join(args.root, VK, "Modules", "ImGui", "Libs", sub, f"libImGui.{ext}"),
        "vma": os.path.join(args.root, VK, "Modules", "Vulkan", "Libs", sub, "libVkMemAlloc.a"),
        "joltc": os.path.join(args.root, JOLT_JAI, "Libs", sub, f"libJoltC.{ext}"),
    }
    for name in todo:
        if os.path.exists(paths[name]) and not args.force:
            print(f"{paths[name]} already built")
            continue
        if name == "imgui":
            build_imgui(args.root, libdir, ext, cxx, args.release)
        elif name == "vma":
            build_vma(args.root, libdir, cxx, args.release)
        else:
            build_joltc(args.root, libdir, ext, args.jobs, args.release)
        print("built", paths[name])


if __name__ == "__main__":
    main()
