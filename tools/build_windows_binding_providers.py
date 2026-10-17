#!/usr/bin/env python3
"""Build real ABI8 fixture archives for a selected Windows SDK, never the host."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools/native_bindings"))
from build_dependencies import prepare_source  # noqa: E402
from prepare_rpmalloc import prepare  # noqa: E402
from rpmalloc_profiles import PROFILES, c_definitions  # noqa: E402
from config import RPMALLOC_SDK_BUILD_OPTIONS  # noqa: E402

TARGETS = {
    "windows-x64-mingw": ("x86_64-w64-mingw32", "IMAGE_FILE_MACHINE_AMD64"),
    "windows-arm64-mingw": ("aarch64-w64-mingw32", "IMAGE_FILE_MACHINE_ARM64"),
    "windows-x64": ("x86_64-pc-windows-msvc", "IMAGE_FILE_MACHINE_AMD64"),
    "windows-arm64": ("aarch64-pc-windows-msvc", "IMAGE_FILE_MACHINE_ARM64"),
}


def toolchain(target):
    if target not in TARGETS:
        raise ValueError("unsupported Windows provider target: " + target)
    prefix, _ = TARGETS[target]
    if target.endswith("-mingw"):
        compiler = shutil.which(prefix + "-gcc") or shutil.which(prefix + "-clang")
        archiver = shutil.which(prefix + "-ar")
        command = [compiler]
    else:
        compiler, archiver = shutil.which("clang"), shutil.which("llvm-ar")
        command = [
            compiler,
            "--target=" + prefix,
            "-fms-runtime-lib=static",
            "-fms-omit-default-lib",
        ]
    if not compiler or not archiver:
        raise ValueError("actual compiler/archiver SDK required for " + target)
    return command, archiver, target.endswith("-mingw")


def verify_machine(headers, target):
    if target not in TARGETS:
        raise ValueError("unsupported Windows provider target: " + target)
    machines = [
        line.strip().split()[1]
        for line in headers.splitlines()
        if line.strip().startswith("Machine:")
    ]
    if not machines or any(machine != TARGETS[target][1] for machine in machines):
        raise ValueError("COFF machine mismatch for " + target + ": " + repr(machines))


def provider_plan():
    manifest = json.loads(
        (ROOT / "tools/native_bindings/ci-rpmalloc-profiles.json").read_text()
    )
    profiles = {p["sdk"]: p for p in PROFILES}
    names = manifest["sdk_profiles"]
    if manifest["native_abi"] != 8 or len(names) != 23 or len(set(names)) != 23:
        raise ValueError("expected reviewed23 unique ABI8 profiles")
    api = json.loads(
        (ROOT / "stdlib/internal/bindings/rpmalloc-sdk/provenance.json").read_text()
    )
    result = []
    for name in names:
        profile = profiles[name]
        for kind, folder in [
            ("sdk", "rpmalloc-heaps"),
            ("callbacks", "rpmalloc-callbacks"),
        ]:
            result.append(
                dict(
                    name=profile[kind],
                    units=[
                        (
                            ROOT / "stdlib/internal/bindings" / folder / "bridge.c",
                            c_definitions(profile, api["functions"]),
                        )
                    ],
                )
            )
    base = profiles["binding_rpmalloc_sdk"]
    result += [
        dict(
            name="binding_callback_context",
            units=[(ROOT / "stdlib/internal/bindings/callback-context/bridge.c", [])],
        ),
        dict(
            name="rpmalloc_callback_probe",
            units=[
                (ROOT / "tests/native-bindings/rpmalloc_callback_probe.c", []),
                (
                    ROOT / "tests/native-bindings/rpmalloc_sdk_config_probe.c",
                    c_definitions(base, api["functions"]),
                ),
            ],
        ),
    ]
    probe_flags = [
        "-D" + key + "=" + str(value)
        for key, value in RPMALLOC_SDK_BUILD_OPTIONS.items()
    ]
    for name in ("rpmalloc_sdk_probe", "rpmalloc_storage_probe"):
        result.append(
            dict(
                name=name,
                units=[
                    (
                        ROOT / "tests/native-bindings/generated" / (name + ".c"),
                        probe_flags,
                    )
                ],
            )
        )
    raw = profiles["binding_rpmalloc_sdk_raw_base"]
    aliases = [
        "-Dnative_error_mode=raw_native_error_mode",
        "-Dnative_error=raw_native_error",
        "-Dnative_name=raw_native_name",
        "-Dnative_size=raw_native_size",
        "-DLIVE_NAME_COPY=" + raw["facade_prefix"] + "name_copy",
    ]
    live = ROOT / "tests/native-bindings/rpmalloc_live_config_probe.c"
    result.append(
        dict(
            name="rpmalloc_live_config_probe",
            units=[
                (live, c_definitions(base, api["functions"])),
                (live, c_definitions(raw, api["functions"]) + aliases),
            ],
        )
    )
    adaptive = profiles["binding_rpmalloc_sdk_nostats_adaptive"]
    flags = [
        "-D" + key + "=" + str(value) for key, value in adaptive["options"].items()
    ]
    flags += [
        "-D" + f["name"] + "=binding_adaptive_probe_" + f["name"]
        for f in api["functions"]
    ]
    result.append(
        dict(
            name="rpmalloc_adaptive_probe",
            units=[(ROOT / "tests/native-bindings/rpmalloc_adaptive_probe.c", flags)],
        )
    )
    return result


def build(target, output, readobj):
    compiler, archiver, mingw = toolchain(target)
    output.mkdir(parents=True, exist_ok=True)
    plan = provider_plan()
    receipt = dict(
        target=target,
        native_abi=8,
        native_execution=False,
        linkage="static target archives; no second allocator instance",
        commands=[],
        assets={},
        source_inputs={},
    )

    def run(command):
        command = list(map(str, command))
        result = subprocess.run(command, text=True, capture_output=True)
        receipt["commands"].append(
            dict(
                argv=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
            )
        )
        (output / "windows-binding-providers.json").write_text(
            json.dumps(receipt, indent=2) + "\n"
        )
        if result.returncode:
            raise RuntimeError(
                "target provider build failed: "
                + " ".join(command)
                + "\n"
                + result.stderr
            )
        return result.stdout

    pin = json.loads((ROOT / "tools/native_bindings/dependencies.json").read_text())[
        "rpmalloc"
    ]
    vendor = output / "source-cache"
    prepare_source("rpmalloc", pin, vendor)
    prepared = prepare(vendor / "rpmalloc", output / "rpmalloc-source")
    inputs = {
        Path(__file__).resolve(),
        ROOT / "tools/native_bindings/dependencies.json",
        ROOT / "tools/native_bindings/ci-rpmalloc-profiles.json",
        ROOT / "tools/native_bindings/rpmalloc_profiles.py",
        ROOT / "tools/native_bindings/prepare_rpmalloc.py",
        ROOT / "tools/native_bindings/config.py",
        ROOT / "stdlib/internal/bindings/rpmalloc-sdk/provenance.json",
    }
    inputs.update((ROOT / "stdlib/internal/bindings/rpmalloc-heaps").glob("*"))
    for item in plan:
        inputs.update(unit for unit, _ in item["units"])
    receipt["source_inputs"] = {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted(inputs)
        if p.is_file()
    }
    receipt["prepared_sdk"] = json.loads((prepared / "provenance.json").read_text())
    for item in plan:
        objects = []
        for index, (unit, definitions) in enumerate(item["units"]):
            obj = output / (item["name"] + "-" + str(index) + ".o")
            run(
                [
                    *compiler,
                    "-std=c11",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    # MinGW's NtCurrentTeb reads gs:0, which GCC sees as a read through address zero.
                    *(["-Wno-error=array-bounds"] if mingw else []),
                    "-O2",
                    "-isystem",
                    prepared,
                    "-I",
                    ROOT / "stdlib/internal/bindings/rpmalloc-heaps",
                    *definitions,
                    "-c",
                    unit,
                    "-o",
                    obj,
                ]
            )
            verify_machine(run([readobj, "--file-headers", obj]), target)
            objects.append(obj)
        archive = output / (
            ("lib" + item["name"] + ".a") if mingw else item["name"] + ".lib"
        )
        run([archiver, "rcs", archive, *objects])
        verify_machine(run([readobj, "--file-headers", archive]), target)
        receipt["assets"][archive.name] = hashlib.sha256(
            archive.read_bytes()
        ).hexdigest()
    # Compiling profiles separately cannot expose duplicate SDK inline symbols
    # or unresolved calls between archives. Pull every object into one real PE.
    control = output / "all-providers-link.c"
    control.write_text("int main(void) { return 0; }\n")
    archives = [output / name for name in receipt["assets"]]
    executable = output / "all-providers-link.exe"
    if mingw:
        arguments = ["-Wl,--whole-archive", *archives, "-Wl,--no-whole-archive"]
    else:
        arguments = ["-Wl,/WHOLEARCHIVE:" + str(path) for path in archives]
    run([*compiler, control, *arguments, "-o", executable])
    verify_machine(run([readobj, "--file-headers", executable]), target)
    receipt["whole_archive_link"] = dict(
        sha256=hashlib.sha256(executable.read_bytes()).hexdigest(),
        native_execution=False,
        qualification="link closure only; original runtime fixtures remain required",
    )
    if any(
        hashlib.sha256((ROOT / p).read_bytes()).hexdigest() != h
        for p, h in receipt["source_inputs"].items()
    ):
        raise RuntimeError("target provider source changed during build")
    receipt["completed"] = True
    (output / "windows-binding-providers.json").write_text(
        json.dumps(receipt, indent=2) + "\n"
    )
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", required=True, choices=TARGETS)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--readobj",
        default=os.environ.get("LLVM_READOBJ")
        or shutil.which("llvm-readobj")
        or shutil.which("llvm-readobj-23"),
    )
    args = parser.parse_args()
    if not args.readobj:
        parser.error("an actual LLVM readobj executable is required (--readobj)")
    build(args.platform, args.output.resolve(), args.readobj)


if __name__ == "__main__":
    main()
