"""Guarded four-receiver correction: source/privacy and real non-GUI SDK calls."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from shared_snapshot import ROOT, capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "jailint", "formatter", "cc", "output-root"):
        parser.add_argument("--" + name, type=Path, required=True)
    native = parser.add_mutually_exclusive_group()
    native.add_argument(
        "--native",
        action="store_true",
        help="Requires a coordinated bounded four-program SDK slot",
    )
    native.add_argument(
        "--native-sanitizer-only",
        action="store_true",
        help="One SDK program under a separately coordinated retry slot",
    )
    args = parser.parse_args()
    executing = args.native or args.native_sanitizer_only
    runtime = Path(
        subprocess.check_output(
            [str(args.cc), "-print-file-name=libclang_rt.asan_osx_dynamic.dylib"],
            text=True,
        ).strip()
    ).resolve(strict=True)
    extras = [
        "stdlib/Bindings_Generator/tests/driven/sdk/appkit-receiver-forwarding.jai",
        "stdlib/Bindings_Generator/tests/driven/sdk/appkit_receiver_probe.m",
        "crates/jaic-cli/tests/objc_selectors.rs",
        "tests/compiler/objc-selectors.txt",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/appkit.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="receiver-forwarding-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        PYTHONDONTWRITEBYTECODE="1",
        ASAN_OPTIONS="abort_on_error=1",
        UBSAN_OPTIONS="halt_on_error=1",
    )
    report = dict(
        snapshot=str(snapshot),
        identity=identity,
        work=str(work),
        checks=[],
        native_requested=executing,
        sanitizer_only=args.native_sanitizer_only,
        asan_runtime=dict(
            path=str(runtime), sha256=hashlib.sha256(runtime.read_bytes()).hexdigest()
        ),
        sdk_programs=0,
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter, args.cc)
        },
    )
    output = work / "report.json"

    def run(label, command, rejection=None, sdk=False, harness=False):
        assert unchanged(snapshot, identity)
        command = list(map(str, command))
        if sdk:
            report["sdk_programs"] += 1
        try:
            result = subprocess.run(
                command,
                cwd=snapshot,
                env=env,
                capture_output=True,
                text=True,
                timeout=40 if sdk else 90,
            )
            passed = result.returncode == 0
            if rejection:
                passed = result.returncode != 0 and rejection in result.stderr
            elif sdk:
                passed = (
                    passed
                    and not result.stderr
                    and (
                        "0 failed; 0 ignored" in result.stdout
                        if harness
                        else not result.stdout
                    )
                )
            row = dict(
                label=label,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        except subprocess.TimeoutExpired as error:
            row = dict(label=label, command=command, passed=False, timeout=str(error))
        report["checks"].append(row)
        output.write_text(json.dumps(report, indent=2) + "\n")
        print(label, row["passed"], row.get("stderr", "")[-1200:], flush=True)
        return row["passed"]

    try:
        fixture, probe = (snapshot / p for p in extras[:2])
        for cpu in ("arm64", "x64"):
            run(
                "public source " + cpu,
                [
                    args.compiler,
                    fixture,
                    "-check",
                    "-no_dce",
                    "-os",
                    "macos",
                    "-cpu",
                    cpu,
                ],
            )
            run(
                "strict lint " + cpu,
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
                    work / (cpu + ".json"),
                    snapshot / "stdlib/Objective_C/AppKit",
                    fixture,
                ],
            )
            private = work / (cpu + "-privacy.jai")
            private.write_text(
                'O :: #import "Objective_C"; bad :: O.NSView; main :: () {}\n'
            )
            run(
                "root privacy " + cpu,
                [args.compiler, private, "-check", "-os", "macos", "-cpu", cpu],
                rejection="has no exported member `NSView`",
            )
        run(
            "Jai format",
            [
                args.formatter,
                "--check",
                "--config",
                snapshot / "jaifmt.toml",
                fixture,
                snapshot / "stdlib/Objective_C/AppKit/generated.jai",
            ],
        )
        run(
            "generator regressions",
            [
                sys.executable,
                "-m",
                "unittest",
                "discover",
                "-s",
                "tools/native_bindings",
                "-p",
                "test_appkit_receivers.py",
            ],
        )
        for cpu in ("arm64", "x86_64"):
            run(
                "SDK compile " + cpu,
                [
                    args.cc,
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
        generated = work / "generated.jai"
        run(
            "regenerate",
            [
                sys.executable,
                snapshot / "tools/native_bindings/generate_appkit.py",
                "--output",
                generated,
            ],
        )
        run(
            "format regenerated",
            [args.formatter, "--config", snapshot / "jaifmt.toml", generated],
        )
        report["checks"].append(
            dict(
                label="exact regeneration",
                passed=(
                    generated.read_bytes()
                    == (
                        snapshot / "stdlib/Objective_C/AppKit/generated.jai"
                    ).read_bytes()
                ),
            )
        )
        for mode, flags, cflags in (
            ("O0", ["-O0"], ["-O0"]),
            ("O2", ["-O2"], ["-O2"]),
            (
                "sanitized",
                ["-O1", "-sanitize", "address,undefined"],
                ["-O1", "-fsanitize=address,undefined"],
            ),
        ):
            directory = work / mode
            directory.mkdir()
            env.update(
                LIBRARY_PATH=str(directory),
                DYLD_LIBRARY_PATH=str(directory),
                JAIC_NATIVE_LIBS=str(directory),
            )
            helper = directory / "libappkit_receiver_probe.dylib"
            built = run(
                "SDK helper " + mode,
                [
                    args.cc,
                    "-dynamiclib",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    *cflags,
                    "-framework",
                    "AppKit",
                    probe,
                    "-o",
                    helper,
                ],
            )
            exe = directory / "receiver-test"
            if (
                built
                and run(
                    "native build " + mode,
                    [
                        args.compiler,
                        fixture,
                        *flags,
                        "-no_backtrace_on_crash",
                        "-exe",
                        exe,
                    ],
                )
                and (
                    args.native or (args.native_sanitizer_only and mode == "sanitized")
                )
            ):
                # The SDK helper is independently instrumented. Preload the
                # matching compiler runtime before any dependent dylib loads.
                if mode == "sanitized":
                    env["DYLD_INSERT_LIBRARIES"] = str(runtime)
                try:
                    run("native SDK execution " + mode, [exe], sdk=True)
                finally:
                    env.pop("DYLD_INSERT_LIBRARIES", None)
        harness_source = work / "original-selector-audit.rs"
        harness_source.write_text(
            (snapshot / extras[2]).read_text()
            + "\n#[test]\nfn preserves_audited_receiver_count() { assert_eq!(scan().receivers.len(), 874); }\n"
        )
        env.update(
            CARGO_MANIFEST_DIR=str(snapshot / "crates/jaic-cli"),
            CARGO_TARGET_TMPDIR=str(work),
            CARGO_BIN_EXE_jaic=str(args.compiler),
        )
        harness = work / "original-selector-harness"
        if (
            run(
                "original audit build plus count guard",
                ["rustc", "--test", "--edition=2024", harness_source, "-o", harness],
            )
            and args.native
        ):
            run(
                "original SDK audit and 874 count guard",
                [harness, "--test-threads=1"],
                sdk=True,
                harness=True,
            )
    finally:
        report["inputs_unchanged"] = unchanged(snapshot, identity)
        report["lease_released"] = executing
        report["limitations"] = [
            "Mac ARM64 native only when --native; x64 source/SDK compile only.",
            "No application/window/GUI/clipboard/fault program.",
        ]
        output.write_text(json.dumps(report, indent=2) + "\n")
    print(output)
    raise SystemExit(
        0
        if report["inputs_unchanged"] and all(row["passed"] for row in report["checks"])
        else 1
    )


if __name__ == "__main__":
    main()
