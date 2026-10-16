#!/usr/bin/env python3
"""Focused source-built reducer/original nonheap TOML acceptance, with durable receipts."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = "sjorsdonkers--toml-jai"
FIXTURES = ROOT / "tests/developer-tools/runtime-reducers/toml-nonheap"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command_passed(
    code,
    stdout,
    stderr,
    *,
    reject=False,
    runtime=False,
    expected_stdout=None,
    application_negatives=False,
):
    if reject:
        return code != 0 and (
            "argument of type `Distinct_Number` does not match parameter type `float64`"
            in stderr
        )
    diagnostics = (stdout + stderr).lower()
    fatal = any(
        marker in diagnostics
        for marker in (
            "assertion failed",
            "runtime error",
            "null pointer read",
        )
    )
    return (
        code == 0
        and (not runtime or (not fatal and (application_negatives or not stderr)))
        and (expected_stdout is None or stdout == expected_stdout)
    )


def stage_admitted(application_root, destination):
    access = json.loads((application_root / "ACCESS.json").read_text())
    records = []
    for entry in access["files"]:
        name = entry["path"]
        if not name.startswith(PROJECT + "/"):
            continue
        relative = PurePosixPath(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("unsafe admitted application path")
        source = (application_root / relative).resolve(strict=True)
        if not source.is_relative_to(application_root.resolve(strict=True)):
            raise ValueError("application input escapes curated ACCESS root")
        data = source.read_bytes()
        if hashlib.sha256(data).hexdigest() != entry["sha256"]:
            raise ValueError(f"changed admitted input: {name}")
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        records.append(entry)
    if not records:
        raise ValueError("no admitted TOML application inputs")
    return records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, required=True)
    parser.add_argument("--jailint", type=Path, required=True)
    parser.add_argument("--jaifmt", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--application-root", type=Path, required=True)
    parser.add_argument("--source-only", action="store_true")
    args = parser.parse_args()
    target = Path(os.environ["CARGO_TARGET_DIR"]).resolve(strict=True)
    work = Path(tempfile.mkdtemp(prefix="toml-nonheap-check-", dir=target))
    env = dict(
        os.environ, TMPDIR=str(work), JAIC_STDLIB=str(args.stdlib.resolve(strict=True))
    )
    tools = {
        name: getattr(args, name).resolve(strict=True)
        for name in ("jaic", "jailint", "jaifmt")
    }
    report = {
        "phase": "source-only" if args.source_only else "interpreter/native-O0-O2",
        "runtime_executed": False,
        "tools": {
            name: {"path": str(path), "sha256": digest(path)}
            for name, path in tools.items()
        },
        "stdlib": str(args.stdlib.resolve(strict=True)),
        "commands": [],
    }
    watched = [
        args.stdlib / "internal/basic" / name
        for name in (
            "format.jai",
            "format-aggregate.jai",
            "format-float.jai",
            "format-integer.jai",
        )
    ]
    before = {str(path): digest(path) for path in watched}
    report["source_before"] = before
    inputs = stage_admitted(args.application_root, work)
    report["admitted_inputs"] = inputs
    receipt = work / "receipt.json"

    def save():
        receipt.write_text(json.dumps(report, indent=2) + "\n")

    def run(
        label,
        command,
        cwd=ROOT,
        reject=False,
        runtime=False,
        expected_stdout=None,
        application_negatives=False,
    ):
        command = list(map(str, command))
        captures = {}
        try:
            result = subprocess.run(
                command,
                cwd=cwd,
                env=env,
                capture_output=True,
                timeout=60,
            )
            decoded = {}
            for stream in ("stdout", "stderr"):
                data = getattr(result, stream)
                path = work / (str(len(report["commands"])) + "-" + stream + ".bin")
                path.write_bytes(data)
                captures[stream] = {"path": str(path), "sha256": digest(path)}
                try:
                    decoded[stream] = data.decode("utf-8", errors="strict")
                    captures[stream]["valid_utf8"] = True
                except UnicodeDecodeError:
                    captures[stream]["valid_utf8"] = False
                    if not application_negatives:
                        raise
                    # Original validation deliberately exercises arbitrary input bytes.
                    # Preserve exact bytes; escaping is only for the JSON view.
                    decoded[stream] = data.decode("utf-8", errors="backslashreplace")
            passed = command_passed(
                result.returncode,
                decoded["stdout"],
                decoded["stderr"],
                reject=reject,
                runtime=runtime,
                expected_stdout=expected_stdout,
                application_negatives=application_negatives,
            )
            row = {
                "label": label,
                "argv": command,
                "cwd": str(cwd),
                "exit": result.returncode,
                "stdout": decoded["stdout"],
                "stderr": decoded["stderr"],
                "captures": captures,
                "passed": passed,
            }
        except (OSError, UnicodeError, subprocess.TimeoutExpired) as error:
            row = {
                "label": label,
                "argv": command,
                "error": str(error),
                "captures": captures,
                "passed": False,
            }
        report["commands"].append(row)
        save()
        print(label, "PASS" if row["passed"] else "FAIL", flush=True)
        return row["passed"]

    positives = [
        FIXTURES / name
        for name in ("union-metadata.jai", "union-format.jai", "numeric-variants.jai")
    ]
    positives.append(ROOT / "tests/portable-stdlib/hash-table-contract/main.jai")
    run(
        "fixture format",
        [
            tools["jaifmt"],
            "--check",
            "--config",
            ROOT / "jaifmt.toml",
            FIXTURES / "tagged-choice-types.jai",
            *positives,
        ],
    )
    for source in positives:
        run(source.stem + " source", [tools["jaic"], "-check", "-no_dce", source])
        run(source.stem + " lint", [tools["jailint"], "-D", "warnings", source])
    run(
        "nominal type rejection",
        [
            tools["jaic"],
            "-check",
            "-no_dce",
            FIXTURES / "numeric-distinct-negative.jai",
        ],
        reject=True,
    )
    app = work / PROJECT / "examples"
    originals = [
        app / (name + ".jai")
        for name in (
            "union_and_overlay",
            "validation",
            "custom_handlers",
            "formatting_control",
        )
    ]
    for source in originals:
        run(
            source.stem + " original source", [tools["jaic"], "-check", source], cwd=app
        )
    if not args.source_only:
        for source in positives + originals:
            modes = (
                ["base", "metadata", "float", "int"]
                if source.stem == "numeric-variants"
                else [None]
            )
            cwd = app if source in originals else work
            for opt in ("-interp", "-O0", "-O2"):
                executable = work / (source.stem + opt + ".exe")
                if opt != "-interp" and not run(
                    source.stem + opt + " build",
                    [tools["jaic"], opt, source, "-exe", executable],
                    cwd=cwd,
                ):
                    continue
                for mode in modes:
                    command = (
                        [tools["jaic"], "-interp", source, "--"]
                        if opt == "-interp"
                        else [executable]
                    )
                    if mode:
                        command.append(mode)
                    report["runtime_executed"] = True
                    marker = None
                    if source.stem == "union-metadata":
                        marker = "union metadata/layout ok\n"
                    elif source.stem == "union-format":
                        marker = (
                            "count short: {COUNT, 37}\ntext short: {TEXT, chosen}\n"
                            "count long: {COUNT, 37}\ntext long: {TEXT, chosen}\n"
                        )
                    elif source.stem == "numeric-variants":
                        marker = {
                            "base": "base numeric formatting ok\n",
                            "metadata": "numeric variant metadata/storage ok\n",
                            "float": "variant float: 8.50\n",
                            "int": "variant integer: 29\n",
                        }[mode]
                    run(
                        source.stem + opt + ":" + str(mode),
                        command,
                        cwd=cwd,
                        runtime=True,
                        expected_stdout=marker,
                        application_negatives=source in originals,
                    )
    report["source_drift"] = [
        name for name, sha in before.items() if digest(Path(name)) != sha
    ]
    report["input_drift"] = [
        entry["path"]
        for entry in inputs
        if digest(work / entry["path"]) != entry["sha256"]
    ]
    report["passed"] = (
        all(row["passed"] for row in report["commands"])
        and not report["source_drift"]
        and not report["input_drift"]
    )
    save()
    print(receipt)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
