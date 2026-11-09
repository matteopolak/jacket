#!/usr/bin/env python3
"""Audit the compiler's actual project inclusion graph in reproducible file-open orders.

PYTHONPATH=tools python3 tools/lsp/project_graph.py --project ~/repos/focus --entry first.jai --out /tmp/focus-graph
PYTHONPATH=tools python3 tools/lsp/project_graph.py --upstream --out /tmp/corpus-graphs
"""

from __future__ import annotations
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

from corpus import upstream_sandbox
from toolkit.repo import ROOT, cargo_target_dir


def confinement(reads, writes, policy=upstream_sandbox.policy):
    """The sandbox the compiler runs in, or None where the policy lets it run unconfined: a CI
    runner without one. Where the policy refuses (`Unsandboxed`) so does the audit."""
    if not policy(False):
        return None
    return upstream_sandbox.make_sandbox(reads, writes)


def run_case(args, project, entry, destination):
    """Discover files by executing the entry's dependency check inside the corpus sandbox.

    The graph includes actual conditional loads, imports and metaprogram workspaces; it is not
    inferred by matching load strings. Files outside this repository are counted separately.
    """
    destination.mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix="lsp-project-graph-"))
    reads = upstream_sandbox.jaic_reads(str(args.jaic), ROOT / "stdlib", project)
    reads += [args.audit.parent.parent, project]
    sandbox = confinement(reads, [scratch])

    def command(argv, timeout):
        confined, env = upstream_sandbox.confine(
            sandbox,
            scratch,
            list(map(str, argv)),
            dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib")),
        )
        try:
            result = subprocess.run(
                confined,
                cwd=project,
                env=env,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=timeout,
                stdin=subprocess.DEVNULL,
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return None, "", "timeout"

    try:
        code, out, err = command([args.jaic, entry, "-deps"], args.check_timeout)
        (destination / "dependencies.stderr").write_text(err)
        (destination / "dependencies.stdout").write_text(out)
        files, external = set(), set()
        for line in out.splitlines():
            path = Path(line.strip())
            if not path.is_absolute():
                path = entry.parent / path
            if path.suffix != ".jai" or not path.is_file():
                continue
            path = path.resolve()
            if path.is_relative_to(project):
                files.add(path.relative_to(project).as_posix())
            else:
                external.add(str(path))
        graph = {
            "project": str(project),
            "entry": str(entry),
            "dependency_status": code,
            "files": sorted(files),
            "external_files": sorted(external),
        }
        (destination / "graph.json").write_text(json.dumps(graph, indent=2))
        if code != 0 or not files:
            return {
                "project": str(project),
                "entry": str(entry),
                "status": "graph-failed",
                "dependency_status": code,
                "files": len(files),
            }
        inputs = scratch / "inputs.json"
        inputs.write_text(json.dumps(sorted(files)))
        result = {
            "project": str(project),
            "entry": str(entry),
            "files": len(files),
            "status": "checked",
            "orders": [],
            "order_differences": [],
        }
        baseline = None
        for seed in args.seeds:
            progress = scratch / f"order-{seed}.jsonl"
            argv = [
                args.audit,
                "--json",
                "--strict",
                "--redacted",
                "--inputs",
                inputs,
                "--order-seed",
                str(seed),
                "--progress",
                progress,
                "--case-id",
                "project-graph",
                project,
            ]
            details = scratch / f"order-{seed}-details.jsonl"
            if args.details:
                argv += ["--observation-details", details]
            started = time.monotonic()
            code, out, err = command(argv, args.timeout)
            (destination / f"order-{seed}.stdout").write_text(out)
            (destination / f"order-{seed}.stderr").write_text(err)
            if details.exists():
                shutil.copyfile(details, destination / details.name)
            if progress.exists():
                shutil.copyfile(progress, destination / f"order-{seed}.jsonl")
            records = (
                [json.loads(line) for line in progress.read_text().splitlines()]
                if progress.exists()
                else []
            )
            early = {
                r["input_id"]: r
                for r in records
                if r["kind"] == "observations" and r["phase"] == "opened"
            }
            final = {
                r["input_id"]: r
                for r in records
                if r["kind"] == "observations" and r["phase"] == "final"
            }
            fields = (
                "diagnostics",
                "inlay_hints",
                "symbols",
                "references",
                "completions",
                "semantic",
            )
            if baseline is None:
                baseline = final
            for phase, observed in [("opened", early), ("final", final)]:
                for identifier, wanted in baseline.items():
                    got = observed.get(identifier)
                    changed = (
                        list(fields)
                        if got is None
                        else [field for field in fields if got[field] != wanted[field]]
                    )
                    if changed:
                        result["order_differences"].append(
                            {
                                "seed": seed,
                                "phase": phase,
                                "input_id": identifier,
                                "changed": changed,
                            }
                        )
            summary = {
                "seed": seed,
                "exit": code,
                "seconds": round(time.monotonic() - started, 2),
                "opened": len(early),
                "audited": sum(r["kind"] == "file" for r in records),
                "final": len(final),
            }
            try:
                report = json.loads(out)
                summary["coverage"] = report.get("counts")
            except (ValueError, TypeError):
                pass
            result["orders"].append(summary)
            print(json.dumps({"entry": str(entry), **summary}), flush=True)
            (destination / "summary.json").write_text(json.dumps(result, indent=2))
        if (
            any(
                order["exit"] != 0 or order["final"] != len(files)
                for order in result["orders"]
            )
            or result["order_differences"]
        ):
            result["status"] = "failed"
        return result
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument("--project", type=Path)
    selection.add_argument("--upstream", action="store_true")
    parser.add_argument("--entry", type=Path)
    parser.add_argument(
        "--details",
        action="store_true",
        help="retain full local feature responses to diagnose differences",
    )
    parser.add_argument("--check-timeout", type=int, default=60)
    parser.add_argument(
        "--jaic", type=Path, default=Path(shutil.which("jaic") or "jaic")
    )
    parser.add_argument(
        "--audit",
        type=Path,
        default=cargo_target_dir() / "release/examples/ide_coverage",
    )
    parser.add_argument(
        "--seeds",
        type=lambda s: [int(n) for n in s.split(",")],
        default=[0, 1, 8675309],
    )
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if os.name == "nt":
        parser.error("this driver requires macOS sandbox-exec or Linux bwrap")
    if args.timeout <= 0 or args.check_timeout <= 0:
        parser.error("timeouts must be positive")
    if args.upstream and args.entry:
        parser.error("--entry requires --project")
    args.jaic, args.audit = args.jaic.resolve(), args.audit.resolve()
    if not args.audit.is_file():
        parser.error(
            "build ide_coverage first: cargo build --release -p jai-language-server --example ide_coverage"
        )
    if not args.seeds or args.seeds[0] != 0 or len(set(args.seeds)) != len(args.seeds):
        parser.error("seeds must be unique and start with 0 (canonical baseline)")
    cases = []
    if args.upstream:
        spec = importlib.util.spec_from_file_location(
            "lsp_oracle", ROOT / "tools/lsp/oracle.py"
        )
        oracle = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(oracle)
        upstream = ROOT / "corpus/upstream"
        for entry in oracle.files_of("upstream", Path("/unused"), 0):
            relative = entry.relative_to(upstream)
            cases.append(((upstream / relative.parts[0]).resolve(), entry.resolve()))
    elif args.project and args.entry:
        project = args.project.expanduser().resolve()
        entry = (project / args.entry).resolve()
        cases = [(project, entry)]
    else:
        parser.error("choose --upstream or --project DIR --entry FILE")
    args.out = args.out.resolve()
    args.out.mkdir(parents=True, exist_ok=True)
    results = []
    for index, (project, entry) in enumerate(cases):
        result = run_case(args, project, entry, args.out / f"case-{index:04}")
        results.append(result)
        (args.out / "summary.json").write_text(json.dumps(results, indent=2))
        print(json.dumps(result), flush=True)
    return int(any(r["status"] != "checked" for r in results))


if __name__ == "__main__":
    raise SystemExit(main())
