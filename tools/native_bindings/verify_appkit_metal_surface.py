"""Exact reviewed flag exports and Render-only encoder promotion, without GUI."""

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
        "--native", action="store_true", help="Bounded non-GUI SDK programs"
    )
    args = parser.parse_args()
    names = [
        "appkit-collection-behavior-names",
        "metal-command-encoder-inheritance",
        "appkit",
        "metal",
    ]
    extras = ["stdlib/Bindings_Generator/tests/driven/sdk/" + n + ".jai" for n in names]
    extras += [
        "stdlib/Bindings_Generator/tests/driven/sdk/appkit-metal-contract-probe.m",
        "stdlib/Bindings_Generator/tests/driven/sdk/apple_abi_probe.m",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/metal.md",
        "docs/specs/stdlib/modules/appkit.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="appkit-metal-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        LIBRARY_PATH=str(work),
        DYLD_LIBRARY_PATH=str(work),
        PYTHONDONTWRITEBYTECODE="1",
        PYTHONPATH=str(snapshot / "tools/native_bindings"),
    )
    checks = []
    report = dict(
        snapshot=str(snapshot),
        work=str(work),
        input_identity=identity,
        native_requested=args.native,
        checks=checks,
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter)
        },
    )

    def run(label, command, rejection=None, cwd=None):
        command = [str(x) for x in command]
        try:
            result = subprocess.run(
                command,
                cwd=cwd or snapshot,
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
        checks.append(row)
        (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        return row["passed"]

    for cpu in ("arm64", "x64"):
        for name in names:
            source = snapshot / ("stdlib/Bindings_Generator/tests/driven/sdk/" + name + ".jai")
            run(
                "source " + cpu + " " + name,
                [
                    args.compiler,
                    source,
                    "-check",
                    "-no_dce",
                    "-os",
                    "macos",
                    "-cpu",
                    cpu,
                ],
            )
        fixture = (
            snapshot / "stdlib/Bindings_Generator/tests/driven/sdk/metal-command-encoder-inheritance.jai"
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
                work / (cpu + "-metal-lint.json"),
                snapshot / "stdlib/Metal",
                fixture,
            ],
        )
        fixture = (
            snapshot / "stdlib/Bindings_Generator/tests/driven/sdk/appkit-collection-behavior-names.jai"
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
                work / (cpu + "-appkit-lint.json"),
                snapshot / "stdlib/Objective_C/AppKit",
                fixture,
            ],
        )

    short = [
        "Default",
        "CanJoinAllSpaces",
        "MoveToActiveSpace",
        "Managed",
        "Transient",
        "Stationary",
        "ParticipatesInCycle",
        "IgnoresCycle",
        "FullScreenPrimary",
        "FullScreenAuxiliary",
        "FullScreenNone",
        "FullScreenAllowsTiling",
        "FullScreenDisallowsTiling",
    ]
    for name in short:
        source = work / ("short-" + name + ".jai")
        source.write_text(
            'A :: #import "Objective_C/AppKit"; bad :: A.NSWindowCollectionBehavior.'
            + name
            + "; main :: () {}\n"
        )
        run(
            "no short alias " + name,
            [args.compiler, source, "-check"],
            "has no member `" + name + "`",
        )
    for name in ("Primary", "Auxiliary", "CanJoinAllApplications"):
        member = "NSWindowCollectionBehavior" + name
        source = work / (member + ".jai")
        source.write_text(
            'A :: #import "Objective_C/AppKit"; bad :: A.'
            + member
            + "; main :: () {}\n"
        )
        run(
            "no newer SDK export " + member,
            [args.compiler, source, "-check"],
            "has no exported member `" + member + "`",
        )
    for name in (
        "NSWindowCollectionBehavior",
        "NSWindowCollectionBehaviorFullScreenPrimary",
    ):
        source = work / ("root-" + name + ".jai")
        source.write_text(
            'O :: #import "Objective_C"; bad :: O.' + name + "; main :: () {}\n"
        )
        run(
            "root privacy " + name,
            [args.compiler, source, "-check"],
            "has no exported member `" + name + "`",
        )
    methods = (
        "endEncoding",
        "insertDebugSignpost",
        "pushDebugGroup",
        "popDebugGroup",
        "device",
        "label",
        "setLabel",
    )
    for record in (
        "MTLBlitCommandEncoder",
        "MTLComputeCommandEncoder",
        "MTLParallelRenderCommandEncoder",
    ):
        for method in methods:
            source = work / (record + method + ".jai")
            source.write_text(
                'M :: #import "Metal"; bad :: M.'
                + record
                + "."
                + method
                + "; main :: () {}\n"
            )
            run(
                "reviewed absent inheritance " + record + "." + method,
                [args.compiler, source, "-check"],
                "has no constant member `" + method + "`",
            )

    probe = snapshot / "stdlib/Bindings_Generator/tests/driven/sdk/appkit-metal-contract-probe.m"
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
    helpers = run(
        "SDK runtime helper link",
        [
            "cc",
            "-dynamiclib",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "AppKit",
            "-framework",
            "Metal",
            probe,
            "-o",
            work / "libappkit_metal_contract_probe.dylib",
        ],
    )
    helpers = (
        run(
            "retained ABI helper link",
            [
                "cc",
                "-dynamiclib",
                "-framework",
                "Foundation",
                "-framework",
                "Metal",
                "-framework",
                "OpenGL",
                snapshot / "stdlib/Bindings_Generator/tests/driven/sdk/apple_abi_probe.m",
                "-o",
                work / "libapple_abi_probe.dylib",
            ],
        )
        and helpers
    )
    if helpers:
        for name in names:
            source = snapshot / ("stdlib/Bindings_Generator/tests/driven/sdk/" + name + ".jai")
            modes = ("O0", "O2", "sanitized") if name in names[:2] else ("O0", "O2")
            for mode in modes:
                flags = (
                    ["-O1", "-sanitize", "address,undefined"]
                    if mode == "sanitized"
                    else ["-" + mode]
                )
                exe = work / (name + "-" + mode)
                if (
                    run(
                        "link " + name + " " + mode,
                        [
                            args.compiler,
                            source,
                            *flags,
                            "-no_backtrace_on_crash",
                            "-exe",
                            exe,
                        ],
                    )
                    and args.native
                ):
                    run("execute " + name + " " + mode, [exe])

    run(
        "general spec fragment tests",
        [
            sys.executable,
            "-m",
            "unittest",
            "discover",
            "-s",
            "tools/native_bindings",
            "-p",
            "test_spec_fragments.py",
        ],
    )
    generation = (
        "from pathlib import Path; from spec import parse_spec; from generate import generate_facade,format_source; "
        "p=Path('stdlib/Metal/declarations.jai'); old=p.read_text(); "
        "body,_=generate_facade('Metal',parse_spec(Path('docs/specs/stdlib/modules/metal.md').read_text())); "
        "assert old==format_source(old.split('#scope_export',1)[0]+'#scope_export\\n\\n'+body,Path("
        + repr(str(args.formatter))
        + "))"
    )
    run("exact Metal regeneration", [sys.executable, "-c", generation])
    paths = [
        snapshot / "stdlib/Objective_C/AppKit/module.jai",
        snapshot / "stdlib/Metal/declarations.jai",
    ]
    paths += [snapshot / ("stdlib/Bindings_Generator/tests/driven/sdk/" + n + ".jai") for n in names[:3]]
    run(
        "owned Jai formatting",
        [args.formatter, "--check", "--config", snapshot / "jaifmt.toml", *paths],
    )
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["limitations"] = [
        "No GUI/window created; actual Metal hardware required by native Render test.",
        "Mac x64 source/SDK only; no x64/native Linux/Windows/iOS claim.",
        "Other three encoder inheritance absent by reviewed public contract.",
    ]
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "report.json")
    raise SystemExit(
        0 if report["inputs_unchanged"] and all(row["passed"] for row in checks) else 1
    )


if __name__ == "__main__":
    main()
