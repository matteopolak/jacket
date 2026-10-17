#!/usr/bin/env python3
"""Verify production helper/foreign links and discovery through a configured SDK."""
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import build_developer_bindings
import developer_native_runtime

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, output=None):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV, capture_output=True,
                            text=True, timeout=60)
    if result.returncode or result.stderr or (output is not None and result.stdout != output):
        raise RuntimeError(f"Configured SDK check failed: {command}: {result.returncode}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def link(path, target):
    if path.is_symlink():
        path.unlink()
    elif path.exists():
        raise RuntimeError(f"Refusing to replace non-symlink SDK fixture path: {path}")
    path.symlink_to(target, target_is_directory=target.is_dir())


def dependencies(binary, sdk_library):
    if platform.system() == "Darwin":
        output = run(["/usr/bin/otool", "-L", binary])
        paths = [line.strip().split(" (compatibility", 1)[0] for line in output.splitlines()[1:]]
    else:
        output = run(["ldd", binary])
        paths = re.findall(r"=>\s+(\S+)", output)
    selected = [path for path in paths if "libclang" in Path(path).name]
    if platform.system() == "Darwin":
        valid = len(selected) == 1 and selected[0] in {"@rpath/libclang.dylib", "@loader_path/libclang.dylib"}
    else:
        valid = selected and all(Path(path).resolve() == sdk_library.resolve() for path in selected)
    if not valid:
        raise RuntimeError("Production executable does not select the configured/bundled libclang runtime")
    return selected


def main():
    host = platform.system()
    if host not in {"Darwin", "Linux"}:
        raise RuntimeError("Configured SDK execution check currently requires macOS/Linux")
    primary = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm")).resolve()
    sdk = ROOT / "target/developer-tools-configured-sdk"
    if sdk.is_symlink():
        raise RuntimeError("Configured SDK fixture root must be an owned directory")
    # The builder resolves its prefix. Use that same directory identity when a
    # parent build directory is symlinked (for example to an external volume).
    sdk = sdk.resolve()
    (sdk / "bin").mkdir(parents=True, exist_ok=True)
    link(sdk / "include", primary / "include")
    link(sdk / "lib", primary / "lib")
    wrapper = sdk / "bin/clang"
    if wrapper.is_symlink():
        raise RuntimeError("Configured compiler wrapper must be an owned file")
    shutil.copyfile(ROOT / "tests/developer-tools/configured-clang.py", wrapper)
    wrapper.chmod(0o755)
    marker = sdk / "include-probe.txt"
    (sdk / "compiler.json").write_text(json.dumps({"compiler": str(primary / "bin/clang"), "include_probe_marker": str(marker)}) + "\n")
    library_name = "libclang.dylib" if host == "Darwin" else "libclang.so"
    sdk_library = sdk / "lib" / library_name
    evidence = {"scope": "production source-built helper, include discovery and public native consumers through a nondefault owned SDK layout",
                "configured_prefix": str(sdk), "underlying_official_sdk": str(primary), "consumers": []}
    try:
        helper = build_developer_bindings.build(sdk)
        native_directory = ROOT / "stdlib/internal/developer-tools/clang-bindings"
        library = native_directory / library_name
        if host == "Darwin":
            manifest = json.loads((native_directory / "runtime-sdk.json").read_text())
            if library.is_symlink() or manifest[library_name]["sdk_sha256"] != developer_native_runtime.sha256(sdk_library):
                raise RuntimeError("Production bundled libclang did not originate from the configured SDK")
            evidence["bundled_runtime"] = manifest
        elif library.readlink() != sdk_library:
            raise RuntimeError("Production native facade link did not retain configured SDK path")
        evidence["helper_libclang"] = dependencies(helper, sdk_library)
        if host == "Darwin":
            load_commands = run(["/usr/bin/otool", "-l", helper])
            rpaths = re.findall(r"cmd LC_RPATH\n\s+cmdsize \d+\n\s+path (.*?) \(offset", load_commands)
        else:
            dynamic = run(["readelf", "-d", helper])
            rpaths = [part for value in re.findall(r"\((?:RPATH|RUNPATH)\).*?\[(.*?)\]", dynamic) for part in value.split(":")]
        if host != "Darwin" and str(sdk / "lib") not in rpaths:
            raise RuntimeError("Production helper lacks the configured runtime library search path")
        if host == "Darwin" and rpaths:
            raise RuntimeError("Mac helper must load its bundled runtime directly, without SDK rpaths")
        evidence["helper_runtime_search_paths"] = rpaths
        extension = "dylib" if host == "Darwin" else "so"
        flags = ["-dynamiclib"] if host == "Darwin" else ["-shared", "-fPIC"]
        run([sdk / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *flags,
             ROOT / "tests/developer-tools/bindings-configured-sdk.c", "-o", ROOT / f"target/libdeveloper_tools_configured.{extension}"])
        jaic = ROOT / "target/debug/jaic"
        for mode in ["interp", "o0", "o2"]:
            if marker.exists():
                marker.unlink()
            emitter = ROOT / f"target/developer-tools-configured-emitter-{mode}"
            fixture = ROOT / "tests/developer-tools/bindings-configured-sdk.jai"
            if mode == "interp":
                run([jaic, fixture, "-interp"], output="ok\n")
            else:
                run([jaic, fixture, f"-O{mode[1:]}", "-exe", emitter])
                run([emitter], output="ok\n")
                evidence["consumers"].append({"mode": mode, "libclang": dependencies(emitter, sdk_library)})
            if not marker.is_file():
                raise RuntimeError("Production include discovery did not invoke the configured SDK compiler")
            roundtrip = ROOT / "tests/developer-tools/bindings-configured-sdk-roundtrip.jai"
            consumer = ROOT / f"target/developer-tools-configured-roundtrip-{mode}"
            if mode == "interp":
                run([jaic, roundtrip, "-interp", "-I", ROOT / "target"], output="ok\n")
            else:
                run([jaic, roundtrip, f"-O{mode[1:]}", "-I", ROOT / "target", "-exe", consumer])
                run([consumer], output="ok\n")
        generated = ROOT / "target/developer-tools-configured-generated.jai"
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", generated])
        run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", generated])
        (ROOT / "target/developer-tools-configured-sdk-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    finally:
        build_developer_bindings.build(primary)
    print("Configured SDK production helper/links/discovery and two public interpreter/native O0/O2 fixtures passed")
    return 2


if __name__ == "__main__":
    main()
