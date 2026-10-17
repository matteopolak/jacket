"""Exact unnamed Texture resource edge; native execution requires a fresh lease."""

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
    parser.add_argument("--native", action="store_true", help="Fresh lease required")
    args = parser.parse_args()
    extras = [
        "stdlib/Bindings_Generator/tests/driven/sdk/metal-texture-resource.jai",
        "stdlib/Bindings_Generator/tests/driven/sdk/metal_resource_probe.m",
        "stdlib/Bindings_Generator/tests/driven/sdk/metal-resource-source-controls.jai",
        "stdlib/Bindings_Generator/tests/driven/sdk/metal.jai",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/metal.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="metal-resource-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        LIBRARY_PATH=str(work),
        DYLD_LIBRARY_PATH=str(work),
        PYTHONDONTWRITEBYTECODE="1",
        PYTHONPATH=str(snapshot / extras[4]),
    )
    report = dict(
        snapshot=str(snapshot),
        work=str(work),
        input_identity=identity,
        native_requested=args.native,
        checks=[],
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
    controls = snapshot / extras[2]
    for cpu in ("arm64", "x64"):
        for name, path in (
            ("native fixture source", fixture),
            ("ten promoted API identities", controls),
            ("retained Metal ABI", snapshot / extras[3]),
        ):
            run(
                name + " " + cpu,
                [args.compiler, path, "-check", "-no_dce", "-os", "macos", "-cpu", cpu],
            )
        run(
            "complete Metal strictlint " + cpu,
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
                snapshot / "stdlib/Metal",
                fixture,
            ],
        )
        run(
            "promoted API controls strictlint " + cpu,
            [
                args.jailint,
                "-D",
                "warnings",
                "-os",
                "macos",
                "-cpu",
                cpu,
                "--context",
                controls,
                "--manifest",
                work / (cpu + "-controls-lint.json"),
                controls,
            ],
        )
        for name, expression in (
            ("ordinary", "Ordinary.setLabel"),
            ("buffer", "M.MTLBuffer.setLabel"),
        ):
            source = work / (cpu + "-" + name + "-negative.jai")
            source.write_text(
                'M :: #import "Metal"; Ordinary :: struct { resource: M.MTLResource; } bad :: '
                + expression
                + "; main :: () {}\n"
            )
            run(
                "no invented promotion " + cpu + " " + name,
                [args.compiler, source, "-check", "-os", "macos", "-cpu", cpu],
                "has no constant member `setLabel`",
            )
        for module, member in (
            ("Metal", "binding_objc_send"),
            ("Objective_C", "NSWindow"),
        ):
            source = work / (cpu + "-" + module + "-privacy.jai")
            source.write_text(
                'M :: #import "'
                + module
                + '"; bad :: M.'
                + member
                + "; main :: () {}\n"
            )
            run(
                "private boundary " + cpu + " " + module,
                [args.compiler, source, "-check", "-os", "macos", "-cpu", cpu],
                "has no exported member `" + member + "`",
            )
    probe = snapshot / extras[1]
    for cpu in ("arm64", "x86_64"):
        run(
            "SDK compile " + cpu,
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
    linked = run(
        "SDK helper link",
        [
            "cc",
            "-dynamiclib",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "Metal",
            "-framework",
            "Foundation",
            probe,
            "-o",
            work / "libmetal_resource_probe.dylib",
        ],
    )
    if linked:
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
        "generator retention regressions",
        [sys.executable, "-m", "unittest", "test_unnamed_using", "test_spec_fragments"],
    )
    generation = (
        "from pathlib import Path; from spec import parse_spec; from generate import generate_facade,format_source; "
        "p=Path('stdlib/Metal/declarations.jai'); old=p.read_text(); "
        "body,_=generate_facade('Metal',parse_spec(Path('docs/specs/stdlib/modules/metal.md').read_text())); "
        "assert old==format_source(old.split('#scope_export',1)[0]+'#scope_export\\n\\n'+body,Path("
        + repr(str(args.formatter))
        + "))"
    )
    run("exact regeneration", [sys.executable, "-c", generation])
    run(
        "Jai formatting",
        [
            args.formatter,
            "--check",
            "--config",
            snapshot / "jaifmt.toml",
            snapshot / "stdlib/Metal/declarations.jai",
            fixture,
            controls,
        ],
    )
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["limitations"] = [
        "No runtime SDK evidence unless --native is supplied after a fresh lease.",
        "Mac x64 source/SDK object only; Linux/Windows inapplicable to reviewed Mac-only Metal.",
        "Public wrapper records are not complete native Objective-C instance layouts.",
        "Unnamed using retains canonical Resource receiver; implicit child-pointer conversion not asserted.",
    ]
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "report.json")
    raise SystemExit(
        0
        if report["inputs_unchanged"] and all(row["passed"] for row in report["checks"])
        else 1
    )


if __name__ == "__main__":
    main()
