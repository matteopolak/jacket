"""Prepare matching source-built helpers in the third-party consumers' normal paths.

No application, fault, GUI or device harness is executed here. Native build
failures and incomplete/incorrect build receipts are fatal.
"""

import hashlib
import json
import platform
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def file_sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def host_platform():
    system = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}[
        platform.system()
    ]
    cpu = {"arm64": "arm64", "aarch64": "arm64", "amd64": "x64", "x86_64": "x64"}[
        platform.machine().lower()
    ]
    return system + "-" + cpu


def native_plan(target, native, stdlib, root=ROOT):
    system, cpu = target.split("-")
    if system not in {"windows", "macos", "linux"} or cpu not in {"x64", "arm64"}:
        raise ValueError("unsupported desktop helper target")
    runtime = stdlib / "internal/runtime/crash-native"
    audio = [
        sys.executable,
        str(root / "tools/audio_build_native.py"),
        "--platform",
        target,
        "--out",
        str(native),
    ]
    if system == "windows":
        crash = [
            sys.executable,
            str(root / "tools/build_platform_io_runtime.py"),
            "--stdlib",
            str(stdlib),
            "--cpu",
            cpu,
        ]
        triple = "aarch64" if cpu == "arm64" else "x86_64"
        audio += ["--cc", f"clang --target={triple}-pc-windows-msvc"]
        required = [
            runtime / ("platform_io_windows_crash" + suffix)
            for suffix in [".lib", ".dll"]
        ]
        required += [
            native / name
            for name in [
                "jai_audio_output.lib",
                "jai_audio_output.dll",
                "stb_vorbis.lib",
                "stb_vorbis.dll",
            ]
        ]
    else:
        crash = [
            sys.executable,
            str(root / "tools/build_portable_runtime.py"),
            "--platform",
            target,
            "--output-dir",
            str(native),
        ]
        runtime = native
        suffix = ".dylib" if system == "macos" else ".so"
        required = [runtime / ("libportable_crash" + ext) for ext in [".a", suffix]]
        required += [
            native / ("lib" + name + ext)
            for name in ["jai_audio_output", "stb_vorbis"]
            for ext in [".a", suffix]
        ]
    commands = [crash, audio]
    if system == "linux":
        commands.append(
            [
                sys.executable,
                str(root / "tools/build_portable_clipboard.py"),
                "--output",
                str(native),
            ]
        )
        required += [
            native / name
            for name in ["libportable_clipboard.a", "libportable_clipboard.so"]
        ]
    if system == "macos":
        commands.append(
            [
                sys.executable,
                str(root / "tools/build_portable_watcher.py"),
                "--output-dir",
                str(native),
            ]
        )
        required.append(native / "libportable_watch.dylib")
    return {"commands": commands, "required": required, "runtime": runtime}


def verify_clipboard_build(native, root):
    receipt = json.loads((native / "portable-clipboard-build.json").read_text())
    inputs = {
        "stdlib/internal/clipboard/native/" + name
        for name in ["data.c", "data.h", "x11.c"]
    }
    if set(receipt["inputs"]) != inputs or any(
        file_sha256(root / name) != receipt["inputs"][name] for name in inputs
    ):
        raise ValueError("clipboard build source receipt mismatch")
    for filename, key in [
        ("libportable_clipboard.a", "archive_sha256"),
        ("libportable_clipboard.so", "shared_sha256"),
    ]:
        if file_sha256(native / filename) != receipt[key]:
            raise ValueError("clipboard build artifact receipt mismatch")


def prepare_native_dependencies(target, native, stdlib, root=ROOT):
    if target != host_platform():
        raise ValueError(
            "third-party execution/preparation requires the actual host target"
        )
    plan = native_plan(target, native, stdlib, root)
    for command in plan["commands"]:
        subprocess.run(command, cwd=root, check=True)
    hashes = {}
    for path in plan["required"]:
        if not path.is_file():
            raise FileNotFoundError(f"required source-built helper missing: {path}")
        hashes[str(path)] = file_sha256(path)
    audio = json.loads((native / "audio-build.json").read_text())
    if audio["platform"] != target:
        raise ValueError("audio receipt does not match actual consumer platform")
    for name, digest in audio["artifacts"].items():
        if Path(name).name != name or hashes.get(str(native / name)) != digest:
            raise ValueError("audio helper build receipt/artifact hash mismatch")
    if len(audio["artifacts"]) != 4:
        raise ValueError("incomplete audio helper build receipt")
    if target.startswith("linux-"):
        verify_clipboard_build(native, root)
    if not target.startswith("windows-"):
        portable = json.loads(
            (plan["runtime"] / "portable-crash-build.json").read_text()
        )
        source = root / "stdlib/internal/runtime/crash-native/runtime.c"
        if (
            portable["platform"] != target
            or portable["source_sha256"]
            != hashlib.sha256(source.read_bytes()).hexdigest()
        ):
            raise ValueError(
                "portable crash receipt does not match actual target/source"
            )
        if len(portable["artifacts"]) != 2 or any(
            Path(name).name != name or hashes.get(str(plan["runtime"] / name)) != digest
            for name, digest in portable["artifacts"].items()
        ):
            raise ValueError("portable crash receipt/artifact hash mismatch")
    return {
        "platform": target,
        "artifacts": hashes,
        "runtime_dir": str(plan["runtime"]),
        "native_execution": False,
    }
