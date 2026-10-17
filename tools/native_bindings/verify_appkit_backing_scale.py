"""Reviewed public procedure alias and real CGFloat ABI without window creation."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from shared_snapshot import capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "jailint", "formatter", "output_root"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    parser.add_argument(
        "--native",
        action="store_true",
        help="Three sequential programs under coordinated non-GUI grant",
    )
    args = parser.parse_args()
    extras = [
        "stdlib/Bindings_Generator/tests/driven/sdk/appkit-window-backing-scale.jai",
        "stdlib/Bindings_Generator/tests/driven/sdk/appkit_backing_scale_probe.m",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/appkit.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="backing-scale-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        LIBRARY_PATH=str(work),
        DYLD_LIBRARY_PATH=str(work),
        PYTHONDONTWRITEBYTECODE="1",
        PYTHONPATH=str(snapshot / "tools/native_bindings"),
    )
    report = dict(
        snapshot=str(snapshot),
        input_identity=identity,
        work=str(work),
        checks=[],
        native_requested=args.native,
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter)
        },
    )

    def run(label, command, rejection=None):
        command = [str(x) for x in command]
        try:
            result = subprocess.run(
                command,
                cwd=snapshot,
                env=env,
                capture_output=True,
                text=True,
                timeout=90,
            )
            passed = result.returncode == 0
            if rejection:
                passed = result.returncode != 0 and rejection in result.stderr
            row = dict(
                label=label,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        except subprocess.TimeoutExpired:
            row = dict(label=label, command=command, timeout=True, passed=False)
        report["checks"].append(row)
        (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        return row["passed"]

    fixture = snapshot / extras[0]
    for cpu in ("arm64", "x64"):
        run(
            "public alias source " + cpu,
            [args.compiler, fixture, "-check", "-no_dce", "-os", "macos", "-cpu", cpu],
        )
        run(
            "complete AppKit strictlint " + cpu,
            [
                args.jailint,
                "-D",
                "warnings",
                "-os",
                "macos",
                "-cpu",
                cpu,
                "--context",
                fixture,
                "--manifest",
                work / (cpu + "-lint.json"),
                snapshot / "stdlib/Objective_C/AppKit",
                fixture,
            ],
        )
        negative = work / (cpu + "-root-privacy.jai")
        negative.write_text(
            'O :: #import "Objective_C"; bad :: O.NSWindow; main :: () {}\n'
        )
        run(
            "root Objective_C privacy " + cpu,
            [args.compiler, negative, "-check", "-os", "macos", "-cpu", cpu],
            "has no exported member `NSWindow`",
        )

    probe = snapshot / extras[1]
    for cpu in ("arm64", "x86_64"):
        run(
            "SDK source " + cpu,
            [
                "cc",
                "-arch",
                cpu,
                "-Wall",
                "-Wextra",
                "-Werror",
                "-c",
                probe,
                "-o",
                work / (cpu + "-sdk.o"),
            ],
        )
    if run(
        "SDK helper link",
        [
            "cc",
            "-dynamiclib",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "AppKit",
            probe,
            "-o",
            work / "libappkit_backing_scale_probe.dylib",
        ],
    ):
        for mode, flags in (
            ("O0", ["-O0"]),
            ("O2", ["-O2"]),
            ("sanitized", ["-O1", "-sanitize", "address,undefined"]),
        ):
            exe = work / mode
            if (
                run(
                    "host link " + mode,
                    [
                        args.compiler,
                        fixture,
                        *flags,
                        "-no_backtrace_on_crash",
                        "-exe",
                        exe,
                    ],
                )
                and args.native
            ):
                run("host execution " + mode, [exe])

    run(
        "general public alias unit tests",
        [
            sys.executable,
            "-m",
            "unittest",
            "discover",
            "-s",
            "tools/native_bindings",
            "-p",
            "test_appkit_aliases.py",
        ],
    )
    generated = work / "generated.jai"
    manual = work / "module.jai"
    run(
        "spec alias/facade regeneration",
        [
            sys.executable,
            snapshot / "tools/native_bindings/generate_appkit.py",
            "--output",
            generated,
            "--module-output",
            manual,
        ],
    )
    run(
        "format regeneration",
        [args.formatter, "--config", snapshot / "jaifmt.toml", generated, manual],
    )
    same = all(
        (work / name).read_bytes()
        == (snapshot / "stdlib/Objective_C/AppKit" / name).read_bytes()
        for name in ("generated.jai", "module.jai")
    )
    report["checks"].append(
        dict(label="exact deterministic facade and alias regeneration", passed=same)
    )
    run(
        "Jai formatting",
        [
            args.formatter,
            "--check",
            "--config",
            snapshot / "jaifmt.toml",
            snapshot / "stdlib/Objective_C/AppKit/module.jai",
            fixture,
        ],
    )
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["limitations"] = [
        "Actual SDK receiver is read-only NSScreen; no NSWindow/NSView/NSApplication creation.",
        "NSWindow instance selector/CGFloat metadata checked, no allocated-window behavioral claim.",
        "Mac x64 source/SDK compile only; no x64 or other-platform native claim.",
        "Four legacy receiver mismatches and expected-miss audit remain unchanged.",
    ]
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "report.json")
    raise SystemExit(
        0
        if report["inputs_unchanged"] and all(c["passed"] for c in report["checks"])
        else 1
    )


if __name__ == "__main__":
    main()
