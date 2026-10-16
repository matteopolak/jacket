"""Exact Color promotion and ordinary Depth/Stencil fields; native needs lease."""

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
        "--native", action="store_true", help="Only under fresh integration lease"
    )
    args = parser.parse_args()
    extras = [
        "tests/native-bindings/metal-render-pass-attachments.jai",
        "tests/native-bindings/metal_attachment_probe.m",
        "tests/native-bindings/metal.jai",
        "tools/native_bindings",
        "docs/specs/stdlib/modules/metal.md",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="metal-attachments-", dir=args.output_root))
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
        work=str(work),
        input_identity=identity,
        native_requested=args.native,
        checks=[],
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter)
        },
    )

    def run(label, command, rejection=None, stdin=None):
        command = [str(x) for x in command]
        try:
            result = subprocess.run(
                command,
                cwd=snapshot,
                env=env,
                input=stdin,
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
        return row

    fixture = snapshot / extras[0]
    for cpu in ("arm64", "x64"):
        run(
            "source " + cpu,
            [args.compiler, fixture, "-check", "-no_dce", "-os", "macos", "-cpu", cpu],
        )
        run(
            "retained Metal ABI source " + cpu,
            [
                args.compiler,
                snapshot / extras[2],
                "-check",
                "-no_dce",
                "-os",
                "macos",
                "-cpu",
                cpu,
            ],
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
        for wrapper in (
            "MTLRenderPassDepthAttachmentDescriptor",
            "MTLRenderPassStencilAttachmentDescriptor",
        ):
            source = work / (cpu + wrapper + "-lookup.jai")
            source.write_text(
                'M :: #import "Metal"; bad :: M.'
                + wrapper
                + ".setTexture; main :: () {}\n"
            )
            run(
                "no using lookup " + cpu + " " + wrapper,
                [args.compiler, source, "-check", "-os", "macos", "-cpu", cpu],
                "has no constant member `setTexture`",
            )
            source = work / (cpu + wrapper + "-conversion.jai")
            source.write_text(
                'M :: #import "Metal"; bad :: (p: *M.'
                + wrapper
                + ") { base: *M.MTLRenderPassAttachmentDescriptor = p; } main :: () {}\n"
            )
            run(
                "no implicit base conversion " + cpu + " " + wrapper,
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
                "expected `*MTLRenderPassAttachmentDescriptor`",
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
            work / "libmetal_attachment_probe.dylib",
        ],
    )["passed"]
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
                )["passed"]
                and args.native
            ):
                run("host execution " + mode, [exe])

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
        ],
    )
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["limitations"] = [
        "Native clear/readback omitted unless --native supplied after lease grant.",
        "Mac x64 source/SDK compilation only; no x64 or other-platform native claim.",
        "Public wrapper fields are not full native Objective-C instance layouts.",
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
