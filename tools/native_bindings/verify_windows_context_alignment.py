"""Windows x64 canonical record alignment checks; no native execution."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

from shared_snapshot import capture, unchanged


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in (
        "compiler",
        "jailint",
        "formatter",
        "cc",
        "mingw-sdk",
        "official-header",
        "output-root",
    ):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--official-header-sha256", required=True)
    args = parser.parse_args()
    header_hash = digest(args.official_header)
    if header_hash != args.official_header_sha256:
        raise SystemExit("official SDK header hash mismatch")
    header = args.official_header.read_text()
    for annotation in (
        "typedef struct DECLSPEC_ALIGN(16) _M128A {",
        "typedef struct DECLSPEC_ALIGN(16) DECLSPEC_NOINITALL _CONTEXT {",
    ):
        if annotation not in header:
            raise SystemExit("pinned SDK header lacks required explicit alignment")
    extras = [
        "tests/native-bindings/windows-context-alignment.jai",
        "tests/native-bindings/windows_context_alignment_sdk.c",
        "tools/native_bindings/verify_windows_context_alignment.py",
        "docs/specs/stdlib/modules/windows.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="windows-context-", dir=args.output_root))
    env = dict(os.environ, JAIC_STDLIB=str(snapshot / "stdlib"), TMPDIR=str(work))
    report = dict(
        snapshot=str(snapshot),
        identity=identity,
        work=str(work),
        checks=[],
        tools={
            str(p): digest(p)
            for p in (args.compiler, args.jailint, args.formatter, args.cc)
        },
        official_header=dict(path=str(args.official_header), sha256=header_hash),
        native_executions=0,
        native_windows_verified=False,
    )

    def run(label, command):
        assert unchanged(snapshot, identity)
        command = list(map(str, command))
        result = subprocess.run(
            command,
            cwd=snapshot,
            env=env,
            capture_output=True,
            text=True,
            timeout=90,
            check=False,
        )
        passed = result.returncode == 0
        report["checks"].append(
            dict(
                label=label,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        )
        (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        return passed

    fixture, probe = (snapshot / p for p in extras[:2])
    run(
        "Windows x64 canonical layouts",
        [args.compiler, fixture, "-check", "-no_dce", "-os", "windows", "-cpu", "x64"],
    )
    run(
        "Windows x64 strict lint",
        [
            args.jailint,
            "-D",
            "warnings",
            "-os",
            "windows",
            "-cpu",
            "x64",
            "--context",
            fixture,
            "--manifest",
            work / "lint.json",
            snapshot / "stdlib/Windows/declarations.jai",
            fixture,
        ],
    )
    run(
        "Jai formatting",
        [
            args.formatter,
            "--check",
            "--config",
            snapshot / "jaifmt.toml",
            snapshot / "stdlib/Windows/declarations.jai",
            fixture,
        ],
    )
    depfile = work / "sdk.d"
    obj = work / "sdk-x64.obj"
    if run(
        "MinGW x64 SDK C static assertions",
        [
            args.cc,
            "--target=x86_64-w64-windows-gnu",
            "-isystem",
            args.mingw_sdk,
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-MD",
            "-MF",
            depfile,
            "-c",
            probe,
            "-o",
            obj,
        ],
    ):
        # Record actual compiler-discovered SDK dependencies, not a hand-picked subset.
        names = shlex.split(depfile.read_text().replace("\\\n", " ").split(":", 1)[1])
        report["sdk_dependencies"] = {
            str(Path(p).resolve()): digest(Path(p)) for p in names
        }
        report["object"] = dict(path=str(obj), sha256=digest(obj))
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["official_header_unchanged"] = digest(args.official_header) == header_hash
    report["qualification"] = (
        "Windows x64 Jai source/lint and MinGW cross SDK compilation only; "
        "original MSVC940 native ABI gate rerun required. No ARM64 ABI claim."
    )
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "report.json")
    raise SystemExit(
        0
        if report["inputs_unchanged"]
        and report["official_header_unchanged"]
        and all(row["passed"] for row in report["checks"])
        else 1
    )


if __name__ == "__main__":
    main()
