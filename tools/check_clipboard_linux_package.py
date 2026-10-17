#!/usr/bin/env python3
"""Execute Linux Clipboard from a relocated package on an authorized test display."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

from clipboard_linux_diagnostics import report_failure

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_runtime_verification(proof, prior):
    if not prior["passed"] or not prior["linux_runtime_executed"]:
        failures = [row for row in prior.get("commands", []) if not row["passed"]]
        # Native Jai failures come first, so provisioning/source diagnostics do
        # not obscure the package's unmet actual-Jai-runtime requirement.
        failures.sort(key=lambda row: not row["name"].startswith("linux-jai-"))
        for row in failures[:4]:
            report_failure(row, proof.parent / "commands.json")
        raise RuntimeError("successful real Linux runtime verification is required")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verification-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Linux" or not os.environ.get("DISPLAY"):
        parser.error("requires actual Linux and an authorized ephemeral X11 display")
    proof = args.verification_receipt.resolve(strict=True)
    prior = json.loads(proof.read_text())
    require_runtime_verification(proof, prior)
    compiler = proof.parent / "tools/debug/jaic"
    if digest(compiler) != prior["compiler_sha256"]:
        raise RuntimeError("verified compiler changed")
    for name, sha in prior["source_inputs"].items():
        if digest(ROOT / name) != sha:
            raise RuntimeError(f"verified source changed: {name}")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    package = output / "before-move"
    package.mkdir()
    shutil.copy2(compiler, package / "jaic")
    shutil.copytree(ROOT / "stdlib", package / "stdlib")
    cpu = {"x86_64": "x64", "aarch64": "arm64"}[platform.machine()]
    native_relative = Path("native-libs") / ("linux-" + cpu)
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/build_portable_clipboard.py"),
            "--output",
            str(package / native_relative),
        ],
        check=True,
        timeout=120,
    )
    native_receipt = json.loads(
        (package / native_relative / "portable-clipboard-build.json").read_text()
    )
    for name, sha in native_receipt["inputs"].items():
        if sha != prior["source_inputs"][name]:
            raise RuntimeError(f"native source pairing mismatch: {name}")
    for name, key in [
        ("libportable_clipboard.a", "archive_sha256"),
        ("libportable_clipboard.so", "shared_sha256"),
    ]:
        if digest(package / native_relative / name) != native_receipt[key]:
            raise RuntimeError(f"native artifact changed: {name}")
    shutil.copy2(ROOT / "LICENSE", package / "LICENSE")
    shutil.copy2(
        ROOT / "tests/portable-stdlib/clipboard-linux-roundtrip.jai",
        output / "consumer.jai",
    )
    relocated = output / "relocated"
    package.rename(relocated)
    environment = dict(os.environ)
    # Test normal installed discovery in children, preserving the caller's env.
    environment.pop("JAIC_STDLIB", None)
    environment.pop("JAIC_NATIVE_LIBS", None)
    rows = []

    def run(argv, expected="", missing=False):
        process = subprocess.run(
            list(map(str, argv)),
            cwd=output,
            env=environment,
            capture_output=True,
            text=True,
            timeout=120,
        )
        passed = (
            process.returncode != 0 and "portable_clipboard" in process.stderr
            if missing
            else process.returncode == 0
            and process.stdout == expected
            and process.stderr == ""
        )
        rows.append(
            dict(
                argv=list(map(str, argv)),
                exit=process.returncode,
                stdout=process.stdout,
                stderr=process.stderr,
                passed=passed,
                expected_missing=missing,
            )
        )
        (output / "report.json").write_text(json.dumps(rows, indent=2) + "\n")
        if not passed:
            row = dict(rows[-1], name="relocated-package-command")
            report_failure(row, output / "report.json")
            raise RuntimeError(
                "relocated package verification failed; full output preserved in report.json"
            )

    cli = relocated / "jaic"
    source = output / "consumer.jai"
    run([cli, source, "-interp"], "ok\n")
    for optimization in ["-O0", "-O2"]:
        program = output / ("consumer" + optimization)
        run([cli, source, optimization, "-exe", program])
        run([program], "ok\n")
    native = relocated / native_relative
    held = output / "held-native"
    native.rename(held)
    try:
        run([cli, source, "-interp"], missing=True)
    finally:
        held.rename(native)
    print(f"{len(rows)} relocated Linux package checks passed")


if __name__ == "__main__":
    main()
