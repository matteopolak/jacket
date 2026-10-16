#!/usr/bin/env python3
"""Build matching static/shared private POSIX crash libraries from our C source.

No executable or fault harness runs. The source-relative archive and shared
library are installed together; intermediate objects honor CARGO_TARGET_DIR.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def host_platform():
    system = {"Darwin": "macos", "Linux": "linux"}.get(platform.system())
    cpu = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x64", "amd64": "x64"}.get(
        platform.machine().lower()
    )
    if not system or not cpu:
        raise ValueError(
            "portable native crash backend requires a supported POSIX host"
        )
    return system + "-" + cpu


def build_plan(target, stdlib, temporary, cc, ar):
    if target not in {"macos-x64", "macos-arm64", "linux-x64", "linux-arm64"}:
        raise ValueError("portable crash builder supports desktop POSIX targets only")
    system, cpu = target.split("-")
    directory = stdlib / "internal/runtime/crash-native"
    obj = temporary / "portable-crash.o"
    archive = directory / "libportable_crash.a"
    shared = directory / (
        "libportable_crash.dylib" if system == "macos" else "libportable_crash.so"
    )
    target_flags = (
        ["-arch", "arm64" if cpu == "arm64" else "x86_64"]
        if system == "macos"
        else [
            "--target="
            + ("aarch64" if cpu == "arm64" else "x86_64")
            + "-unknown-linux-gnu"
        ]
    )
    compile_ = [
        cc,
        *target_flags,
        "-std=c11",
        "-O2",
        "-fno-omit-frame-pointer",
        "-fvisibility=hidden",
        "-fPIC",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-c",
        str(ROOT / "stdlib/internal/runtime/crash-native/runtime.c"),
        "-o",
        str(obj),
    ]
    link = [
        cc,
        *target_flags,
        "-dynamiclib" if system == "macos" else "-shared",
        str(obj),
        "-o",
        str(shared),
        "-pthread",
    ]
    if system == "linux":
        link.extend(["-ldl", "-Wl,-soname,libportable_crash.so"])
    else:
        link.append("-Wl,-install_name,@rpath/libportable_crash.dylib")
    return {
        "commands": [compile_, [ar, "rcs", str(archive), str(obj)], link],
        "object": obj,
        "archive": archive,
        "shared": shared,
    }


def verify_object(path, target):
    with path.open("rb") as source:
        header = source.read(64)
    actual = None
    if len(header) >= 20 and header[:6] == b"\x7fELF\x02\x01":
        cpu = {62: "x64", 183: "arm64"}.get(int.from_bytes(header[18:20], "little"))
        actual = "linux-" + str(cpu)
    elif len(header) >= 8 and header[:4] == b"\xcf\xfa\xed\xfe":
        cpu = {0x1000007: "x64", 0x100000C: "arm64"}.get(
            int.from_bytes(header[4:8], "little")
        )
        actual = "macos-" + str(cpu)
    if actual != target:
        raise ValueError(
            f"compiled crash object target mismatch: requested {target}, found {actual}"
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--platform", choices=["macos-x64", "macos-arm64", "linux-x64", "linux-arm64"]
    )
    parser.add_argument("--stdlib", type=Path, default=ROOT / "stdlib")
    args = parser.parse_args()
    target = args.platform or host_platform()
    if target != host_platform():
        parser.error(
            "native preparation requires the actual host; cross SDK builds are separate"
        )
    stdlib = args.stdlib.resolve(strict=True)
    if not stdlib.is_relative_to(ROOT):
        parser.error("helper install must remain in this checkout")
    temporary = (
        Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
        / "native-helpers"
        / target
    )
    temporary.mkdir(parents=True, exist_ok=True)
    plan = build_plan(
        target, stdlib, temporary, "clang", shutil.which("llvm-ar") or "ar"
    )
    plan["archive"].parent.mkdir(parents=True, exist_ok=True)
    plan["archive"].unlink(missing_ok=True)
    for index, command in enumerate(plan["commands"]):
        subprocess.run(command, cwd=ROOT, check=True)
        if index == 0:
            verify_object(plan["object"], target)
    artifacts = {}
    for path in [plan["archive"], plan["shared"]]:
        artifacts[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    source = ROOT / "stdlib/internal/runtime/crash-native/runtime.c"
    receipt = {
        "platform": target,
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "artifacts": artifacts,
        "native_execution": False,
    }
    (plan["archive"].parent / "portable-crash-build.json").write_text(
        json.dumps(receipt, indent=2) + "\n"
    )
    print("Built matching private crash static/shared libraries from source:", target)


if __name__ == "__main__":
    main()
