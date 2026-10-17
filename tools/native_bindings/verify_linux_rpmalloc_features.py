#!/usr/bin/env python3
"""Compile all accepted ABI8 domains against actual Linux headers, never run target code."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

from build_rpmalloc import platform_c_flags
from prepare_rpmalloc import ROOT, prepare
from rpmalloc_profiles import PROFILES, c_definitions


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--sdk-source",
        type=Path,
        required=True,
        help="original hash-pinned official rpmalloc input directory",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cc", default=os.environ.get("CC", "cc"))
    parser.add_argument("--target", choices=["aarch64-linux-gnu", "x86_64-linux-gnu"])
    parser.add_argument("--sysroot", type=Path)
    args = parser.parse_args()
    if platform.system() != "Linux" and not (args.target and args.sysroot):
        parser.error(
            "cross-host verification requires an explicit Linux target and real sysroot"
        )
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    scratch = output / "tmp"
    scratch.mkdir(exist_ok=True)
    env = dict(os.environ, TMPDIR=str(scratch))
    report = output / "report.json"
    inventory = ROOT / "tools/native_bindings/ci-rpmalloc-profiles.json"
    manifest = json.loads(inventory.read_text())
    all_profiles = {p["sdk"]: p for p in PROFILES}
    names = manifest["sdk_profiles"]
    if manifest["native_abi"] != 8 or len(names) != 23 or len(set(names)) != 23:
        raise ValueError("Expected the unchanged accepted23 ABI8 inventory")
    selected = [all_profiles[name] for name in names]
    watched = sorted((ROOT / "stdlib/internal/bindings/rpmalloc-heaps").glob("*"))
    watched = [p for p in watched if p.is_file()]
    watched += [
        ROOT / "stdlib/internal/bindings/rpmalloc-callbacks/bridge.c",
        ROOT / "tools/native_bindings/build_rpmalloc.py",
        ROOT / "tools/native_bindings/config.py",
        ROOT / "tools/native_bindings/rpmalloc_profiles.py",
        inventory,
        ROOT / "tools/native_bindings/prepare_rpmalloc.py",
        ROOT / "tools/native_bindings/dependencies.json",
        ROOT / "stdlib/internal/bindings/rpmalloc-sdk/provenance.json",
        ROOT / "stdlib/Bindings_Generator/tests/driven/sdk/linux-feature-headers.c",
        Path(__file__).resolve(),
    ]

    def source_hashes():
        return {str(p.relative_to(ROOT)): sha(p) for p in watched}

    evidence = {
        "complete": False,
        "native_host": platform.system(),
        "target": args.target or platform.machine(),
        "Linux_runtime_verified": False,
        "ABI8_runtime_queries_executed": 0,
        "source_before": source_hashes(),
        "commands": [],
        "objects": [],
        "profile_inventory_sha256": sha(inventory),
    }
    source = prepare(args.sdk_source.resolve(), output / "prepared-rpmalloc")
    api = json.loads(
        (ROOT / "stdlib/internal/bindings/rpmalloc-sdk/provenance.json").read_text()
    )
    common = [args.cc]
    if args.target:
        common += ["--target=" + args.target]
    if args.sysroot:
        common += ["--sysroot=" + str(args.sysroot.resolve())]
    common += [
        *platform_c_flags("Linux"),
        "-std=c11",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-O2",
        "-fPIC",
        "-isystem",
        str(source),
        "-I",
        str(ROOT / "stdlib/internal/bindings/rpmalloc-heaps"),
    ]

    def checked(command, object_path):
        p = subprocess.run(
            [str(x) for x in command],
            cwd=output,
            env=env,
            capture_output=True,
            text=True,
            timeout=120,
        )
        evidence["commands"].append(
            {
                "command": [str(x) for x in command],
                "exit_code": p.returncode,
                "stdout": p.stdout,
                "stderr": p.stderr,
            }
        )
        report.write_text(json.dumps(evidence, indent=2) + "\n")
        if p.returncode or p.stderr:
            raise RuntimeError(
                "Linux C compilation failed; exact diagnostic retained in "
                + str(report)
            )
        data = object_path.read_bytes()
        machine = int.from_bytes(data[18:20], "little")
        expected = (
            183
            if (args.target or platform.machine()).startswith(("aarch64", "arm64"))
            else 62
        )
        if data[:6] != b"\x7fELF\x02\x01" or machine != expected:
            raise RuntimeError(
                "Compiler did not produce the requested real Linux ELF64 object"
            )
        evidence["objects"].append(
            {
                "path": str(object_path),
                "sha256": sha(object_path),
                "ELF_machine": machine,
            }
        )

    probe = output / "linux-feature-headers.o"
    checked(
        [
            *common,
            "-c",
            ROOT / "stdlib/Bindings_Generator/tests/driven/sdk/linux-feature-headers.c",
            "-o",
            probe,
        ],
        probe,
    )
    for profile in selected:
        for kind, folder in [
            ("sdk", "rpmalloc-heaps"),
            ("callbacks", "rpmalloc-callbacks"),
        ]:
            obj = output / (profile[kind] + ".o")
            checked(
                [
                    *common,
                    *c_definitions(profile, api["functions"]),
                    "-c",
                    ROOT / "stdlib/internal/bindings" / folder / "bridge.c",
                    "-o",
                    obj,
                ],
                obj,
            )
    evidence["source_after"] = source_hashes()
    if evidence["source_before"] != evidence["source_after"]:
        raise RuntimeError("Native source changed during verification")
    evidence["sdk_profiles_compiled"] = 23
    evidence["callback_profiles_compiled"] = 23
    evidence["thread_cache_disabled_profiles_compiled"] = sum(
        not p["thread_cache"] for p in selected
    )
    evidence["complete"] = True
    report.write_text(json.dumps(evidence, indent=2) + "\n")
    print(report)


if __name__ == "__main__":
    main()
