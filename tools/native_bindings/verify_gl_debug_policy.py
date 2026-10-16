"""Review-derived GL policy controls; procedure-table tests are not GPU evidence."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from shared_snapshot import capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "jailint", "formatter", "output_root"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    parser.add_argument("--native", action="store_true", help="Bounded lease required")
    args = parser.parse_args()
    extras = [
        "tests/native-bindings/gl-debug-policy.jai",
        "tests/native-bindings/gl-context.jai",
        "tests/native-bindings/gl-debug-output.jai",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/gl.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="gl-debug-policy-", dir=args.output_root))
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
        work=str(work),
        input_identity=identity,
        native_requested=args.native,
        native_executions=0,
        checks=[],
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter)
        },
    )

    def save():
        (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    def run(label, command, rejection=None, execution=False):
        assert unchanged(snapshot, identity)
        command = [str(x) for x in command]
        try:
            result = subprocess.run(
                command,
                cwd=snapshot,
                env=env,
                capture_output=True,
                text=True,
                timeout=60,
            )
            passed = result.returncode == 0
            if rejection:
                passed = result.returncode != 0 and rejection in result.stderr
            if execution:
                report["native_executions"] += 1
                passed = passed and result.stdout == "" and result.stderr == ""
            row = dict(
                label=label,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
            )
        except subprocess.TimeoutExpired:
            if execution:
                report["native_executions"] += 1
            row = dict(label=label, command=command, timeout=True, passed=False)
        row["inputs_after"] = unchanged(snapshot, identity)
        row["passed"] = row["passed"] and row["inputs_after"]
        report["checks"].append(row)
        save()
        print(label, row["passed"], flush=True)
        return row["passed"]

    fixture = snapshot / extras[0]
    expanded = work / "all-extensions.jai"
    expanded.write_text(
        fixture.read_text().replace(
            '#import "GL";', '#import "GL"(ENABLE_ALL_EXTENSIONS = true);'
        )
    )
    expanded.chmod(0o444)
    report["derived_source"] = dict(
        path=str(expanded), sha256=hashlib.sha256(expanded.read_bytes()).hexdigest()
    )
    report["inputs_before"] = unchanged(snapshot, identity)
    try:
        for name, source in (("default", fixture), ("all", expanded)):
            for target in ("macos", "windows", "linux"):
                for cpu in ("arm64", "x64"):
                    run(
                        "source " + name + " " + target + " " + cpu,
                        [
                            args.compiler,
                            source,
                            "-check",
                            "-no_dce",
                            "-os",
                            target,
                            "-cpu",
                            cpu,
                        ],
                    )
            run("interpreter policy " + name, [args.compiler, source, "-interp"])
            run(
                "complete strictlint " + name,
                [
                    args.jailint,
                    "-D",
                    "warnings",
                    "--context",
                    source,
                    "--manifest",
                    work / (name + "-lint.json"),
                    snapshot / "stdlib/GL",
                    source,
                ],
            )
        for cpu in ("arm64", "x64"):
            run(
                "retained SDK fixture source " + cpu,
                [args.compiler, snapshot / extras[1], "-check", "-no_dce", "-cpu", cpu],
            )
        for member in (
            "binding_debug_state",
            "Binding_Debug_State",
            "binding_debug_message",
        ):
            negative = work / (member + "-private.jai")
            negative.write_text(
                'G :: #import "GL"; bad :: G.' + member + "; main :: () {}\n"
            )
            run(
                "private " + member,
                [args.compiler, negative, "-check"],
                "has no exported member `" + member + "`",
            )
        run(
            "production and test format",
            [
                args.formatter,
                "--check",
                "--config",
                snapshot / "jaifmt.toml",
                snapshot / "stdlib/GL/wrappers.jai",
                fixture,
                snapshot / extras[1],
            ],
        )
        # Exactly one default-profile policy program in three modes. No GPU or trap.
        for mode, flags in (
            ("O0", ["-O0"]),
            ("O2", ["-O2"]),
            ("sanitized", ["-O1", "-sanitize", "address,undefined"]),
        ):
            executable = work / mode
            if (
                run(
                    "native build " + mode,
                    [
                        args.compiler,
                        fixture,
                        *flags,
                        "-no_backtrace_on_crash",
                        "-exe",
                        executable,
                    ],
                )
                and args.native
            ):
                run("native policy " + mode, [executable], execution=True)
    finally:
        report["inputs_after"] = unchanged(snapshot, identity)
        report["derived_input_unchanged"] = (
            hashlib.sha256(expanded.read_bytes()).hexdigest()
            == report["derived_source"]["sha256"]
        )
        report["lease_released"] = args.native
        report["qualification"] = [
            "Procedure-table policy doubles, not native GPU success or real core/ARB support.",
            "glEnable is an SDK constant procedure: synchronous true path source-only here.",
            "Error debug_break(true) not executed; no trap/fault lease requested.",
            "Real Mac4.1 unsupported path and original Focus startup replay coordinator-owned.",
            "Linux/Windows native and supported-driver callback/sync/break tests remain separate gates.",
        ]
        save()
        if args.native:
            root = Path(__file__).resolve().parents[2]
            with (root / "docs/worker-inbox.md").open("a") as inbox:
                inbox.write(
                    "\n\n## Bindings RELEASE GL debug policy bounded native lease\n\n"
                    + str(report["native_executions"])
                    + " native programs completed/reaped; lease RELEASED in finally. "
                    "No GUI/GPUcontext/trap/fault program. Exact policy-only proof: "
                    + str(work / "report.json")
                    + "\n"
                )
    print(work / "report.json")
    raise SystemExit(
        0
        if report["inputs_after"]
        and report["derived_input_unchanged"]
        and all(c["passed"] for c in report["checks"])
        else 1
    )


if __name__ == "__main__":
    main()
