#!/usr/bin/env python3
"""Coordinator-only opaque scanner orchestration. Never reads or stages Jai source."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


def read_progress(path):
    header = None
    files = {}
    cursor = None
    plan = None
    try:
        lines = path.read_text().splitlines()
    except OSError:
        return None
    for index, line in enumerate(lines):
        try:
            record = json.loads(line)
        except ValueError:
            # A killed append may leave only the final record incomplete.
            if index == len(lines) - 1:
                break
            raise ValueError("invalid checkpoint before final record")
        if record.get("redacted") is not True:
            raise ValueError("unredacted checkpoint")
        if record["kind"] == "header":
            header = record
        elif record["kind"] == "file":
            files[record["file"]["input_id"]] = record["file"]
        elif record["kind"] in ("cursor", "start"):
            cursor = record
        elif record["kind"] == "plan":
            plan = record["fingerprint"]
    if header is None:
        return None
    return dict(partial=True, redacted=True, accepted=False, build_id=header["build_id"],
                stdlib_id=header["stdlib_id"], expected_inputs=header["expected_inputs"],
                selection=header["selection"], plan=plan, completed_files=len(files),
                remaining_inputs=header["expected_inputs"] - sum(file["status"] == "audited" for file in files.values()),
                unexecuted_records=sum(file["status"] == "unexecuted" for file in files.values()),
                files=list(files.values()), cursor=cursor)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scanner", required=True, type=Path)
    parser.add_argument("--stdlib", required=True, type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--timeout", type=float, default=300)
    parser.add_argument("--cases", help="exact case IDs to retry; not whole-manifest acceptance")
    parser.add_argument("--checkpoint-dir", type=Path)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    assert manifest["source_snapshot_id"] and manifest["stdlib_id"]
    cases = manifest["cases"]
    assert cases and len({case["id"] for case in cases}) == len(cases)
    assert all(re.fullmatch(r"[A-Za-z0-9_-]+", case["id"]) for case in cases)
    all_cases = len(cases)
    if args.cases:
        selected = set(args.cases.split(","))
        assert selected <= {case["id"] for case in cases}
        cases = [case for case in cases if case["id"] in selected]
    checkpoints = args.checkpoint_dir or args.output.with_suffix(".checkpoints")
    if any((checkpoints / (case["id"] + ".jsonl")).exists() for case in cases):
        parser.error("checkpoint paths must be new; prior evidence is not a retry result")
    report = dict(schema=1, scope=manifest["scope"], source_snapshot_id=manifest["source_snapshot_id"],
                  stdlib_id=manifest["stdlib_id"], scanner_sha256=hashlib.sha256(args.scanner.read_bytes()).hexdigest(),
                  manifest_sha256=hashlib.sha256(args.manifest.read_bytes()).hexdigest(), accepted=False, cases=[])
    report["whole_manifest_cases"] = all_cases
    report["selected_cases"] = len(cases)
    for case in cases:
        command = [str(args.scanner.resolve()), "--json", "--redacted", "--strict", "--case-id", case["id"],
                   "--stdlib", str(args.stdlib.resolve()), "--stdlib-id", manifest["stdlib_id"]]
        progress = checkpoints / (case["id"] + ".jsonl")
        command += ["--progress", str(progress)]
        for option in ("os", "cpu", "review", "inputs"):
            if option in case:
                command += ["--" + option, case[option]]
        if "only_inputs" in case:
            command += ["--only-inputs", ",".join(case["only_inputs"])]
        if case.get("no_runtime", False):
            command.append("--no-runtime")
        command.append(case["root"])
        result = dict(id=case["id"], status="unexecuted", accepted=False, expected_files=case["expected_files"])
        try:
            process = subprocess.run(command, capture_output=True, text=True, timeout=args.timeout)
            result["exit_code"] = process.returncode
            # Raw stderr can contain source snippets; it is never copied into this handoff.
            result["stderr_present"] = bool(process.stderr)
            scan = json.loads(process.stdout)
            if not scan.get("redacted"):
                raise ValueError("scanner did not redact")
            result.update(status="executed", scan=scan)
            result["input_count_matches"] = scan["counts"]["files"] == case["expected_files"]
            result["accepted"] = process.returncode == 0 and scan["accepted"] and result["input_count_matches"]
        except subprocess.TimeoutExpired:
            result["failure_kind"] = "timeout"
        except (OSError, ValueError, KeyError):
            result["failure_kind"] = "launch_or_report_failure"
        if not result["accepted"]:
            try:
                partial = read_progress(progress)
                if partial:
                    result["progress"] = partial
            except (ValueError, KeyError, TypeError):
                result["progress_failure"] = "invalid_checkpoint"
        report["cases"].append(result)
        report["remaining_cases"] = len(cases) - len(report["cases"])
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    report["accepted_selected_cases"] = all(case["accepted"] for case in report["cases"])
    report["accepted"] = len(cases) == all_cases and report["accepted_selected_cases"]
    report["unexecuted_cases"] = sum(case["status"] == "unexecuted" for case in report["cases"])
    report["failed_cases"] = sum(not case["accepted"] for case in report["cases"])
    report["expected_input_files"] = sum(case["expected_files"] for case in report["cases"])
    report["unexecuted_input_files"] = sum(
        max(0, case["expected_files"] - sum(file["status"] == "audited" for file in case.get("progress", {}).get("files", [])))
        if case["status"] == "unexecuted" else case["scan"]["counts"]["unexecuted_files"]
        for case in report["cases"]
    )
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
