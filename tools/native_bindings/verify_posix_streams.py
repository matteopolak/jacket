"""Verify SDK-backed mutable stdio globals; execution needs a serialized lease.

Without --native this performs source checks and native compilation/linking only.
No generated program or probe library is executed or loaded by this script.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from shared_snapshot import capture, unchanged


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "jailint", "formatter", "output_root"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    parser.add_argument(
        "--native",
        action="store_true",
        help="Execute host programs ONLY after integrator lease release",
    )
    args = parser.parse_args()
    extra = [
        "stdlib/Bindings_Generator/tests/driven/sdk/posix-stream-globals.jai",
        "stdlib/Bindings_Generator/tests/driven/sdk/posix_stream_probe.c",
        "tools/native_bindings/verify_posix_streams.py",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/posix.md",
        "jailint.toml",
        "jaifmt.toml",
    ]
    snapshot, identity = capture(args.output_root, extra)
    work = Path(tempfile.mkdtemp(prefix="posix-streams-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        LIBRARY_PATH=str(work),
        DYLD_LIBRARY_PATH=str(work),
        PYTHONDONTWRITEBYTECODE="1",
    )
    checks = []
    report = dict(
        snapshot=str(snapshot),
        input_identity=identity,
        work=str(work),
        native_requested=args.native,
        checks=checks,
        tools={
            str(p): digest(p) for p in (args.compiler, args.jailint, args.formatter)
        },
    )

    def save():
        (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    def run(label, command, contains=(), stdin=None, rejection=None):
        command = [str(x) for x in command]
        try:
            result = subprocess.run(
                command,
                cwd=snapshot,
                env=env,
                input=stdin,
                capture_output=True,
                text=True,
                timeout=180,
            )
            row = dict(
                label=label,
                command=command,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=result.returncode == 0
                and all(text in result.stdout for text in contains),
            )
            if rejection:
                row["passed"] = result.returncode != 0 and rejection in result.stderr
        except subprocess.TimeoutExpired:
            row = dict(label=label, command=command, passed=False, timeout=True)
        checks.append(row)
        save()
        return row

    fixture = snapshot / extra[0]
    probe = snapshot / extra[1]
    for os_ in ("macos", "linux"):
        for cpu in ("arm64", "x64"):
            run(
                "source " + os_ + "-" + cpu,
                [args.compiler, fixture, "-check", "-no_dce", "-os", os_, "-cpu", cpu],
            )
            run(
                "strict lint " + os_ + "-" + cpu,
                [
                    args.jailint,
                    "-D",
                    "warnings",
                    "-os",
                    os_,
                    "-cpu",
                    cpu,
                    "--context",
                    fixture,
                    "--manifest",
                    work / (os_ + cpu + "-lint.json"),
                    snapshot / "stdlib/POSIX",
                    fixture,
                ],
            )

    for cpu in ("arm64", "x86_64"):
        obj = work / (cpu + "-sdk.o")
        if run(
            "SDK compile " + cpu,
            ["cc", "-arch", cpu, "-Wall", "-Wextra", "-Werror", "-c", probe, "-o", obj],
        )["passed"]:
            run(
                "SDK relocations " + cpu,
                ["nm", "-u", obj],
                ("___stdinp", "___stdoutp", "___stderrp"),
            )

    for cpu in ("arm64", "x64"):
        for name in ("__stdinp", "__stdoutp", "__stderrp"):
            negative = work / (cpu + name + ".jai")
            negative.write_text(
                'P :: #import "POSIX"; main :: () { cell := P.' + name + "; }\n"
            )
            run(
                "physical symbol is not public " + cpu + " " + name,
                [args.compiler, negative, "-check", "-os", "macos", "-cpu", cpu],
                rejection="has no exported member `" + name + "`",
            )

    built_probe = run(
        "SDK probe library link",
        [
            "cc",
            "-dynamiclib",
            "-Wall",
            "-Wextra",
            "-Werror",
            probe,
            "-o",
            work / "libposix_stream_probe.dylib",
        ],
    )["passed"]
    if built_probe:
        for mode, flags in (
            ("O0", ["-O0"]),
            ("O2", ["-O2"]),
            ("sanitized", ["-O1", "-sanitize", "address,undefined"]),
        ):
            exe = work / mode
            if run(
                "host link " + mode,
                [
                    args.compiler,
                    fixture,
                    *flags,
                    "-no_backtrace_on_crash",
                    "-emit_ir",
                    work / (mode + ".ll"),
                    "-exe",
                    exe,
                ],
            )["passed"]:
                ir = (work / (mode + ".ll")).read_text()
                checks.append(
                    dict(
                        label="host external global identities " + mode,
                        passed=all(
                            "@" + name + " = external global" in ir
                            for name in ("__stdinp", "__stdoutp", "__stderrp")
                        ),
                    )
                )
                if args.native:
                    run("host execution " + mode, [exe])

    for relative in (extra[0], "stdlib/POSIX/declarations.jai"):
        original = (snapshot / relative).read_text()
        row = run(
            "format " + relative,
            [args.formatter, "--stdin", "--config", snapshot / "jaifmt.toml"],
            stdin=original,
        )
        row["passed"] = row["passed"] and row["stdout"] == original

    # Exercise the spec parser/generator itself, without rewriting shared sources
    # or unrelated modules' coverage ledgers.
    generation = (
        "from pathlib import Path; from generate import generate_facade,format_source; "
        "from spec import parse_spec; "
        "p=Path('stdlib/POSIX/declarations.jai'); old=p.read_text(); "
        "body,entries=generate_facade('POSIX',parse_spec("
        "Path('docs/specs/stdlib/modules/posix.md').read_text())); "
        "expected=format_source(old.split('#scope_export',1)[0]+"
        "'#scope_export\\n\\n'+body,Path(" + repr(str(args.formatter)) + ")); "
        "assert expected==old; print('Exact current POSIX regeneration PASS')"
    )
    import sys

    env["PYTHONPATH"] = str(snapshot / "tools/native_bindings")
    run("spec regeneration", [sys.executable, "-c", generation])
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["limitations"] = [
        "Execution omitted unless --native is explicitly supplied under integrator lease.",
        "Mac x64: source and SDK object checks only; no x64 Jai execution.",
        "Linux: source/lint only; no Linux native ABI/runtime claim.",
        "Android target support and native execution remain separate open gates.",
    ]
    save()
    print(work / "report.json")
    raise SystemExit(
        0 if report["inputs_unchanged"] and all(row["passed"] for row in checks) else 1
    )


if __name__ == "__main__":
    main()
