#!/usr/bin/env python3
"""Real pinned C provider closure for CI; no Jai implementation or staging overlay."""

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools/native_bindings"))
from build_dependencies import INPUTS  # noqa: E402
from rpmalloc_profiles import PROFILES, c_definitions  # noqa: E402


def selected_libraries():
    manifest = json.loads(
        (ROOT / "tools/native_bindings/ci-rpmalloc-profiles.json").read_text()
    )
    profiles = {p["sdk"]: p for p in PROFILES}
    names = manifest["sdk_profiles"]
    if not names or len(names) != len(set(names)) or set(names) - profiles.keys():
        raise ValueError(
            "CI profile inventory is duplicated or absent from production profiles"
        )
    selected = {name for name in INPUTS if not name.startswith("binding_rpmalloc_")}
    if platform.system() != "Darwin":
        selected.difference_update({"fsevents_context_probe", "objc_runtime_probe"})
    for name in names:
        selected.update([name, profiles[name]["callbacks"]])
    return sorted(selected), [profiles[name] for name in names], manifest["native_abi"]


def build_counter_probes(output, profiles):
    """The same real SDK/C counter fixtures as the accepted native baseline."""
    api = json.loads(
        (ROOT / "stdlib/internal/bindings/rpmalloc-sdk/provenance.json").read_text()
    )
    prepared = output / "rpmalloc-source"
    cc = os.environ.get("CC", "cc")
    common = [
        "-std=c11",
        "-fPIC",
        "-pthread",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-isystem",
        str(prepared),
    ]
    live = []
    for raw in [False, True]:
        profile = next(
            p
            for p in profiles
            if p["statistics"]
            and p["global_cache"]
            and p["thread_cache"]
            and p["assertions"]
            and p["validation"]
            and not p["adaptive"]
            and p["raw"] == raw
            and (not raw or not p["public_heaps"])
        )
        aliases = (
            [
                "-Dnative_error_mode=raw_native_error_mode",
                "-Dnative_error=raw_native_error",
                "-Dnative_name=raw_native_name",
                "-Dnative_size=raw_native_size",
                "-DLIVE_NAME_COPY=" + profile["facade_prefix"] + "name_copy",
            ]
            if raw
            else []
        )
        obj = output / ("ci-live-raw.o" if raw else "ci-live.o")
        subprocess.run(
            [
                cc,
                *common,
                *c_definitions(profile, api["functions"]),
                *aliases,
                "-c",
                str(ROOT / "tests/native-bindings/rpmalloc_live_config_probe.c"),
                "-o",
                str(obj),
            ],
            check=True,
        )
        live.append(obj)
    adaptive = next(
        p
        for p in profiles
        if p["adaptive"]
        and not p["raw"]
        and p["global_cache"]
        and p["thread_cache"]
        and p["assertions"]
        and p["validation"]
    )
    flags = [
        "-D" + name + "=" + str(value) for name, value in adaptive["options"].items()
    ]
    flags += [
        "-D" + f["name"] + "=binding_adaptive_probe_" + f["name"]
        for f in api["functions"]
    ]
    obj = output / "ci-adaptive.o"
    subprocess.run(
        [
            cc,
            *common,
            "-O2",
            *flags,
            "-c",
            str(ROOT / "tests/native-bindings/rpmalloc_adaptive_probe.c"),
            "-o",
            str(obj),
        ],
        check=True,
    )
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    shared = (
        ["-dynamiclib", "-Wl,-rpath,@loader_path"]
        if suffix == "dylib"
        else ["-shared", "-Wl,-rpath,$ORIGIN"]
    )
    for name, objects, dependencies in [
        (
            "rpmalloc_live_config_probe",
            live,
            [
                "-L",
                str(output),
                "-lbinding_rpmalloc_sdk",
                "-lbinding_rpmalloc_sdk_raw_base",
            ],
        ),
        ("rpmalloc_adaptive_probe", [obj], []),
    ]:
        subprocess.run(
            ["ar", "rcs", str(output / ("lib" + name + ".a")), *map(str, objects)],
            check=True,
        )
        subprocess.run(
            [
                cc,
                *shared,
                "-pthread",
                *map(str, objects),
                *dependencies,
                "-lm",
                "-o",
                str(output / ("lib" + name + "." + suffix)),
            ],
            check=True,
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if platform.system() not in {"Darwin", "Linux"}:
        parser.error("this real SDK provider recipe requires a macOS/Linux build host")
    output = args.output.resolve()
    if output.exists():
        parser.error(
            "native provider output must be fresh; preserve prior receipts and choose a new path"
        )
    if output.is_relative_to((ROOT / "stdlib").resolve()):
        parser.error(
            "test provider output must remain outside production library sources"
        )
    libraries, profiles, abi = selected_libraries()
    input_files = [
        Path(__file__).resolve(),
        ROOT / "tools/native_bindings/ci-rpmalloc-profiles.json",
        ROOT / "tools/native_bindings/build_dependencies.py",
        ROOT / "tools/native_bindings/build_rpmalloc.py",
        ROOT / "tools/native_bindings/prepare_rpmalloc.py",
        ROOT / "tools/native_bindings/rpmalloc_profiles.py",
        ROOT / "tools/native_bindings/config.py",
        ROOT / "tests/native-bindings/abi_probe.c",
        ROOT / "tests/native-bindings/rpmalloc_live_config_probe.c",
        ROOT / "tests/native-bindings/rpmalloc_adaptive_probe.c",
    ]
    source_hashes = {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in input_files
    }
    command = [
        sys.executable,
        str(ROOT / "tools/native_bindings/build_dependencies.py"),
        "--output",
        str(output),
    ]
    for name in libraries:
        command += ["--library", name]
    subprocess.run(command, check=True)
    build_counter_probes(output, profiles)
    vendor = ROOT / ".work/vendor"
    pins = json.loads((ROOT / "tools/native_bindings/dependencies.json").read_text())
    includes = [
        vendor / "lz4/lib",
        vendor / "xxhash",
        vendor / "meshoptimizer/src",
        vendor / pins["mojoshader"]["archive"]["root"],
    ]
    font_flags = shlex.split(
        subprocess.check_output(["pkg-config", "--cflags", "freetype2"], text=True)
    )
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    probe = output / ("libabi_probe." + suffix)
    compile_command = ["cc", "-Wall", "-Wextra", "-Werror", *flags, *font_flags]
    for include in includes:
        compile_command += ["-isystem", str(include)]
    compile_command += [
        str(ROOT / "tests/native-bindings/abi_probe.c"),
        "-o",
        str(probe),
    ]
    subprocess.run(compile_command, check=True)
    queries = []
    keys = [
        "statistics",
        "raw",
        "public_heaps",
        "global_cache",
        "thread_cache",
        "assertions",
        "validation",
        "adaptive",
    ]
    for profile in profiles:
        library = output / ("lib" + profile["sdk"] + "." + suffix)
        loaded = ctypes.CDLL(str(library))
        query = getattr(loaded, profile["facade_prefix"] + "build_flags")
        query.restype = ctypes.c_uint32
        query.argtypes = []
        value = query()
        expected = (abi << 16) | sum(
            int(bool(profile[key])) << i for i, key in enumerate(keys)
        )
        if value != expected:
            raise RuntimeError(
                f"Native profile ABI/flags mismatch: {library}: {value} != {expected}"
            )
        queries.append({"profile": profile["sdk"], "actual_flags": value})
    hashes = {
        p.name: hashlib.sha256(p.read_bytes()).hexdigest()
        for p in output.iterdir()
        if p.is_file()
        and not p.name.startswith("._")
        and p.suffix in {".a", ".so", ".dylib"}
    }
    if source_hashes != {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in input_files
    }:
        raise RuntimeError("native provider source inputs changed during construction")
    (output / "ci-provider-receipt.json").write_text(
        json.dumps(
            {
                "libraries_requested": libraries,
                "profiles": queries,
                "native_abi": abi,
                "source_inputs": source_hashes,
                "assets": hashes,
                "abi_probe_command": compile_command,
                "qualification": "Actual native ABI flag queries only; not allocator lifecycle, full SDK, sanitizer or platform runtime proof",
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    main()
