#!/usr/bin/env python3
"""Build a real native ARM64 Mesa WGL/llvmpipe driver from pinned upstream sources.

Run inside a native ARM64 VS2022 developer environment. --plan works on any OS
and records configuration only; it never means the driver built or executed.
"""
import argparse
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import urllib.request

from window_render_wgl_ci import pe_machine

ROOT = Path(__file__).resolve().parents[1]
SOURCES = {
    "mesa": (
        "https://archive.mesa3d.org/mesa-25.2.5.tar.xz",
        "bb6243e7a6f525febfa1e6ab50827ca4d4bfdad73812377b0ca9b6c50998b03e",
        "mesa-25.2.5"),
    "llvm": (
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-20.1.8/llvm-project-20.1.8.src.tar.xz",
        "6898f963c8e938981e6c4a302e83ec5beb4630147c7311183cf61069af16333d",
        "llvm-project-20.1.8.src"),
    "winflexbison": (
        "https://codeload.github.com/lexxmark/winflexbison/tar.gz/1176da2adaf972b2216b1ac8dd611e128fe80bc5",
        "94f24390699474cd0448be9f144652e8127690c5eda5aeef32cd3f563a6705bc",
        "winflexbison-1176da2adaf972b2216b1ac8dd611e128fe80bc5"),
}
PACKAGES = {"cmake": "3.31.6", "ninja": "1.13.2", "meson": "1.8.3",
            "Mako": "1.3.10", "MarkupSafe": "3.0.3", "packaging": "25.0"}
MESA_OPTIONS = ["--buildtype=release", "--wrap-mode=nofallback", "-Db_vscrt=mt",
                "-Dgallium-drivers=llvmpipe", "-Dvulkan-drivers=[]", "-Dplatforms=[]",
                "-Dllvm=enabled", "-Dshared-llvm=disabled", "-Ddraw-use-llvm=true",
                "-Dglx=disabled", "-Degl=disabled", "-Dgles1=disabled", "-Dgles2=disabled",
                "-Dopengl=true", "-Dgbm=disabled", "-Dzlib=disabled", "-Dzstd=disabled",
                "-Dexpat=disabled", "-Dbuild-tests=false"]


def source_tree(name, directory):
    url, digest, top = SOURCES[name]
    archive = directory / (name + ".archive")
    if not archive.exists():
        # Interrupted downloads are never promoted into the verified source cache.
        partial = archive.with_suffix(".partial")
        with urllib.request.urlopen(url, timeout=120) as response, partial.open("wb") as output:
            shutil.copyfileobj(response, output)
        partial.replace(archive)
    with archive.open("rb") as verified:
        measured = hashlib.file_digest(verified, "sha256").hexdigest()
    if measured != digest:
        raise ValueError(f"upstream source digest mismatch: {name}")
    destination = directory / top
    if destination.exists():
        raise ValueError(f"refusing stale/modified extracted source; use a fresh --out: {destination}")
    with tarfile.open(archive) as source:
        source.extractall(directory, filter="data")
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", action="store_true")
    parser.add_argument("--out", type=Path, default=ROOT / "target/window-mesa-arm64")
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()
    output = args.out.resolve()
    if not output.is_relative_to(ROOT / "target") or args.jobs < 1:
        parser.error("use a dedicated target directory and a positive job count")
    plan = {"target": "aarch64-pc-windows-msvc", "sources": SOURCES,
            "python_build_packages": PACKAGES, "mesa_options": MESA_OPTIONS,
            "llvm_targets": ["AArch64"], "crt": "static /MT",
            "executed": False, "runtime_verified": False}
    if args.plan:
        print(json.dumps(plan, indent=2))
        return
    if platform.system() != "Windows" or platform.machine().lower() not in {"arm64", "aarch64"}:
        parser.error("native Windows ARM64 is required; no emulated/x64 driver substitute")
    if os.environ.get("VSCMD_ARG_TGT_ARCH", "").lower() != "arm64":
        parser.error("initialize the VS2022 ARM64 developer environment first")
    if sys.version_info < (3, 12):
        parser.error("Python3.12+ is required for filtered source extraction")
    sdk = os.environ.get("WindowsSDKVersion", "").strip("\\").split(".")
    toolset = os.environ.get("VCToolsVersion", "").strip("\\").split(".")
    if len(sdk) < 3 or not sdk[2].isdigit() or int(sdk[2]) < 20348:
        parser.error("Mesa requires Windows SDK20348 or newer")
    if len(toolset) < 2 or tuple(map(int, toolset[:2])) < (14, 39):
        parser.error("Mesa requires VS2022 17.9 / MSVC14.39 or newer")
    for name, version in PACKAGES.items():
        if importlib.metadata.version(name) != version:
            parser.error(f"build package must be pinned: {name}=={version}")
    plan["tool_versions"] = {}
    for tool in ["cmake", "ninja"]:
        executable = shutil.which(tool)
        if not executable or pe_machine(Path(executable)) != 0xaa64:
            parser.error(f"put the pinned native ARM64 {tool} executable on PATH")
        version = subprocess.check_output([executable, "--version"], text=True).splitlines()[0]
        if PACKAGES[tool] not in version:
            parser.error(f"PATH selects a different {tool} version: {version}")
        plan["tool_versions"][tool] = version
    for name in ["MESA_GL_VERSION_OVERRIDE", "MESA_GLSL_VERSION_OVERRIDE"]:
        if os.environ.get(name):
            parser.error(f"remove {name}; actual driver capabilities must remain observable")
    output.mkdir(parents=True, exist_ok=True)
    evidence = output / "source-build.json"
    evidence.write_text(json.dumps(plan, indent=2) + "\n")
    env = dict(os.environ)
    commands = []

    def run(command):
        command = list(map(str, command))
        commands.append(command)
        plan["commands"] = commands
        evidence.write_text(json.dumps(plan, indent=2) + "\n")
        subprocess.run(command, cwd=output, env=env, check=True)

    sources = output / "sources"
    sources.mkdir(exist_ok=True)
    trees = {name: source_tree(name, sources) for name in SOURCES}
    # The source-built parser generators run natively too. Their installed data
    # directory must accompany win_bison; copying just its EXE is insufficient.
    generators = output / "generators"
    run(["cmake", "-S", trees["winflexbison"], "-B", output / "generators-build", "-G", "Ninja",
         "-DCMAKE_BUILD_TYPE=Release", "-DUSE_STATIC_RUNTIME=ON", f"-DCMAKE_INSTALL_PREFIX={generators}"])
    run(["cmake", "--build", output / "generators-build", "--parallel", args.jobs])
    run(["cmake", "--install", output / "generators-build"])
    for name in ["win_flex.exe", "win_bison.exe"]:
        if pe_machine(generators / name) != 0xaa64:
            raise ValueError(f"source-built generator is not ARM64: {name}")
    env["PATH"] = str(generators) + os.pathsep + env["PATH"]
    llvm = output / "llvm"
    run(["cmake", "-S", trees["llvm"] / "llvm", "-B", output / "llvm-build", "-G", "Ninja",
         "-DCMAKE_BUILD_TYPE=Release", "-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded",
         "-DLLVM_USE_CRT_RELEASE=MT", "-DLLVM_TARGETS_TO_BUILD=AArch64",
         "-DLLVM_DEFAULT_TARGET_TRIPLE=aarch64-pc-windows-msvc", "-DLLVM_ENABLE_PROJECTS=",
         "-DLLVM_ENABLE_RUNTIMES=", "-DLLVM_INCLUDE_TESTS=OFF", "-DLLVM_INCLUDE_BENCHMARKS=OFF",
         "-DLLVM_ENABLE_ZLIB=OFF", "-DLLVM_ENABLE_ZSTD=OFF", "-DLLVM_ENABLE_LIBXML2=OFF",
         f"-DCMAKE_INSTALL_PREFIX={llvm}"])
    run(["cmake", "--build", output / "llvm-build", "--parallel", args.jobs])
    run(["cmake", "--install", output / "llvm-build"])
    if pe_machine(llvm / "bin/llvm-config.exe") != 0xaa64:
        raise ValueError("LLVM build is not native ARM64")
    run([llvm / "bin/llvm-config.exe", "--version", "--host-target", "--targets-built"])
    plan["llvm_identity"] = subprocess.check_output(
        [llvm / "bin/llvm-config.exe", "--version", "--host-target", "--targets-built"],
        env=env, text=True).splitlines()
    env["PATH"] = str(llvm / "bin") + os.pathsep + env["PATH"]
    env["CMAKE_PREFIX_PATH"] = str(llvm)
    install = output / "install"
    run([sys.executable, "-m", "mesonbuild.mesonmain", "setup", output / "mesa-build", trees["mesa"],
         f"--prefix={install}", *MESA_OPTIONS])
    plan["meson_build_options"] = json.loads(subprocess.check_output(
        [sys.executable, "-m", "mesonbuild.mesonmain", "introspect", output / "mesa-build", "--buildoptions"],
        env=env, text=True))
    run([sys.executable, "-m", "mesonbuild.mesonmain", "compile", "-C", output / "mesa-build", "-j", args.jobs])
    run([sys.executable, "-m", "mesonbuild.mesonmain", "install", "-C", output / "mesa-build"])
    deploy = output / "driver"
    deploy.mkdir(exist_ok=True)
    libraries = sorted(install.rglob("*.dll"))
    if not {"opengl32.dll", "libgallium_wgl.dll"}.issubset({p.name for p in libraries}):
        raise ValueError("Mesa did not install the actual WGL driver pair")
    plan["driver_files"] = []
    for library in libraries:
        if pe_machine(library) != 0xaa64:
            raise ValueError(f"non-ARM64 driver: {library}")
        shutil.copy2(library, deploy / library.name)
        plan["driver_files"].append({"name": library.name, "machine": "ARM64",
                                     "sha256": hashlib.sha256(library.read_bytes()).hexdigest()})
    plan["executed"] = True
    plan["os"] = platform.platform()
    plan["msvc_toolset"] = env.get("VCToolsVersion")
    plan["windows_sdk"] = env.get("WindowsSDKVersion")
    evidence.write_text(json.dumps(plan, indent=2) + "\n")
    print(f"Built native source driver at {deploy}; runtime remains REQUIRED via window_render_wgl_ci.py")


if __name__ == "__main__":
    main()
