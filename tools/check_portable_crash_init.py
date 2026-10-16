#!/usr/bin/env python3
"""Run real host crash-provider initialization/lifetime controls, without faults.

Builds the production matching archive/shared plan into explicit owned storage.
Linux covers PIE, non-PIE and ELF interposition; macOS retains both existing
static/shared paths. This never substitutes for native hardware-fault tests.
"""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from build_portable_runtime import build_plan, host_platform, verify_object

ROOT = Path(__file__).resolve().parents[1]


def control_variants(target):
    if target.startswith("linux-"):
        return {
            "static-pie": ["-fPIE", "-pie"],
            "static-exec": ["-fno-pie", "-no-pie"],
            "shared": ["-DCRASH_SHARED"],
            "shared-interposed": [
                "-DCRASH_SHARED",
                "-DCRASH_INTERPOSE",
                "-Wl,--export-dynamic",
            ],
        }
    if target.startswith("macos-"):
        return {"static": [], "shared": ["-DCRASH_SHARED"]}
    raise ValueError("crash init controls require the actual desktop POSIX host")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--cc", default="clang")
    parser.add_argument("--ar", default=shutil.which("llvm-ar") or "ar")
    args = parser.parse_args()
    target = host_platform()
    output = args.out.resolve()
    output.mkdir(parents=True, exist_ok=True)
    inputs = [
        ROOT / "stdlib/internal/runtime/crash-native/runtime.c",
        ROOT / "tests/platform-io/portable-crash-init.c",
    ]

    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()

    before = {str(path): digest(path) for path in inputs}
    commands, artifacts = [], {}
    report = {
        "platform": target,
        "inputs": before,
        "commands": commands,
        "artifacts": artifacts,
        "status": "running",
        "hardware_fault_tests": False,
    }

    def run(command, label, expected=b""):
        result = subprocess.run(
            list(map(str, command)), cwd=output, capture_output=True, timeout=30
        )
        (output / (label + ".stdout")).write_bytes(result.stdout)
        (output / (label + ".stderr")).write_bytes(result.stderr)
        commands.append(
            {
                "label": label,
                "command": list(map(str, command)),
                "exit": result.returncode,
                "stdout_hex": result.stdout.hex(),
                "stderr_hex": result.stderr.hex(),
            }
        )
        if result.returncode != 0 or result.stdout != expected or result.stderr:
            raise RuntimeError(
                f"{label} failed: {result.returncode}; stdout={result.stdout!r}; stderr={result.stderr!r}"
            )

    try:
        directory = output / "provider"
        directory.mkdir(exist_ok=True)
        plan = build_plan(target, output / "stdlib", directory, args.cc, args.ar)
        plan["archive"].parent.mkdir(parents=True, exist_ok=True)
        plan["archive"].unlink(missing_ok=True)
        for index, command in enumerate(plan["commands"]):
            run(command, "provider-" + str(index))
            if index == 0:
                verify_object(plan["object"], target)
        for path in (plan["object"], plan["archive"], plan["shared"]):
            artifacts[str(path)] = digest(path)
        for optimization in ("O0", "O2"):
            for variant, flags in control_variants(target).items():
                label = variant + "-" + optimization
                executable = output / label
                libraries = (
                    [plan["archive"]]
                    if variant.startswith("static") or variant == "shared-interposed"
                    else []
                )
                system_libraries = ["-ldl"] if target.startswith("linux-") else []
                run(
                    [
                        args.cc,
                        "-std=c11",
                        "-" + optimization,
                        "-Wall",
                        "-Wextra",
                        "-Werror",
                        inputs[1],
                        *flags,
                        *libraries,
                        "-pthread",
                        *system_libraries,
                        "-o",
                        executable,
                    ],
                    "link-" + label,
                )
                artifacts[str(executable)] = digest(executable)
                run(
                    [
                        executable,
                        *([plan["shared"]] if variant.startswith("shared") else []),
                    ],
                    "run-" + label,
                    b"ok\n",
                )
        report["status"] = "passed"
    finally:
        report["input_drift"] = [
            path
            for path, value in before.items()
            if not Path(path).is_file() or digest(Path(path)) != value
        ]
        report["artifact_drift"] = [
            path
            for path, value in artifacts.items()
            if not Path(path).is_file() or digest(Path(path)) != value
        ]
        if report["input_drift"] or report["artifact_drift"]:
            report["status"] = "failed-drift"
        if report["status"] == "running":
            report["status"] = "failed"
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    if report["status"] != "passed":
        raise RuntimeError("crash initialization controls failed: " + report["status"])
    print("passed actual " + target + " crash initialization/lifetime controls")


if __name__ == "__main__":
    main()
