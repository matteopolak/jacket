"""Source/input verification only; SDK execution requires the coordinator lease."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from shared_snapshot import capture, unchanged


def source_inventory(library):
    invalid = []
    sidecars = []
    files = sorted(library.rglob("*.jai"))
    for path in files:
        data = path.read_bytes()
        row = dict(path=str(path.relative_to(library)), size=len(data))
        if path.name.startswith("._"):
            sidecars.append(dict(row, header=data[:26].hex()))
        try:
            data.decode("utf-8")
        except UnicodeDecodeError as error:
            invalid.append(dict(row, header=data[:26].hex(), error=str(error)))
    return dict(jai_files=len(files), sidecars=sidecars, strict_utf8_failures=invalid)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--inventory-only", type=Path, help="Clean producer stdlib only"
    )
    for name in ("compiler", "jailint", "formatter", "font"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--output-root", type=Path, required=True)
    args = parser.parse_args()
    args.output_root.mkdir(parents=True, exist_ok=True)
    if args.inventory_only:
        report = source_inventory(args.inventory_only)
        path = args.output_root / "selector-source-inventory.json"
        path.write_text(json.dumps(report, indent=2) + "\n")
        print(path)
        return 1 if report["strict_utf8_failures"] else 0
    if not all((args.compiler, args.jailint, args.formatter, args.font)):
        parser.error(
            "source checks require compiler, jailint, formatter and caller-owned font"
        )
    if not args.font.is_file():
        parser.error("the caller font must exist; no default font or skip")
    extras = [
        "tests/native-bindings/freetype.jai",
        "crates/jaic-cli/tests/objc_selectors.rs",
        "tools/native_bindings",
        "jaifmt.toml",
        "jailint.toml",
    ]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix="font-selector-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        PYTHONDONTWRITEBYTECODE="1",
    )
    report = dict(
        snapshot=str(snapshot),
        identity=identity,
        checks=[],
        inventory=source_inventory(snapshot / "stdlib"),
        font=dict(
            path=str(args.font.resolve()),
            sha256=hashlib.sha256(args.font.read_bytes()).hexdigest(),
        ),
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter)
        },
        sdk_executions=0,
    )
    fixture = snapshot / extras[0]
    for target in ("macos", "linux", "windows"):
        for cpu in ("arm64", "x64"):
            commands = [
                (
                    "source",
                    [
                        args.compiler,
                        fixture,
                        "-check",
                        "-no_dce",
                        "-os",
                        target,
                        "-cpu",
                        cpu,
                    ],
                ),
                (
                    "strictlint",
                    [
                        args.jailint,
                        "-D",
                        "warnings",
                        "-os",
                        target,
                        "-cpu",
                        cpu,
                        "--context",
                        fixture,
                        "--manifest",
                        work / (target + cpu + "-lint.json"),
                        snapshot / "stdlib/freetype",
                        fixture,
                    ],
                ),
            ]
            for label, command in commands:
                result = subprocess.run(
                    list(map(str, command)),
                    env=env,
                    cwd=snapshot,
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                report["checks"].append(
                    dict(
                        label=label + " " + target + " " + cpu,
                        exit=result.returncode,
                        stdout=result.stdout,
                        stderr=result.stderr,
                        passed=result.returncode == 0,
                    )
                )
                (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    result = subprocess.run(
        [
            str(args.formatter),
            "--check",
            "--config",
            str(snapshot / "jaifmt.toml"),
            str(fixture),
        ],
        capture_output=True,
        text=True,
    )
    report["checks"].append(
        dict(
            label="fixture format",
            exit=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
            passed=result.returncode == 0,
        )
    )
    report["inputs_unchanged"] = unchanged(snapshot, identity)
    report["font_unchanged"] = (
        hashlib.sha256(args.font.read_bytes()).hexdigest() == report["font"]["sha256"]
    )
    report["qualification"] = [
        "Caller font bytes inspected, never passed to FreeType under held SDK lease.",
        "Root failed-copy AppleDouble inventory is a separate requested fact.",
        "Actual FreeType ABI/glyph/lifetime and ObjC runtime selector gates remain required.",
    ]
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(work / "report.json")
    return (
        0
        if report["inputs_unchanged"]
        and report["font_unchanged"]
        and all(row["passed"] for row in report["checks"])
        else 1
    )


if __name__ == "__main__":
    raise SystemExit(main())
