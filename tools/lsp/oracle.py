#!/usr/bin/env python3
"""Language-server oracle checks over every file `jaic -check` accepts (docs/tools/lsp-oracle.md).

Picks the files (stdlib, tests/compiler, examples, the reviewed upstream corpus, jaigen programs),
keeps those `jaic -check` accepts, runs the `lsp_oracle` example over them in chunks, then runs each
program whose unreachable regions can hold a statement with a marker there: if the marker prints,
the region was reachable. Findings are compared with `tools/lsp/oracle-known.json`; a new one fails.

    python3 tools/lsp/oracle.py --sets examples,compiler --jobs 2
    python3 tools/lsp/oracle.py --sets upstream            # inside tools/bench/sandbox.py
    python3 tools/lsp/oracle.py --sets stdlib --shard 1/4 --out target/lsp-oracle
"""


import argparse
import sys
import concurrent.futures
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from corpus import upstream_sandbox
from toolkit.repo import ROOT, cargo_target_dir, jaifmt_path

SETS = ("stdlib", "compiler", "examples", "upstream", "jaigen")
KNOWN = ROOT / "tools/lsp/oracle-known.json"
MARKER = "@@LSP_ORACLE_REACHED@@"
UPSTREAM = ROOT / "corpus/upstream"
# Case keys that change how a case compiles: such a case cannot be opened as a plain file.
UPSTREAM_SPECIAL = ("args", "setup", "copy", "link", "files", "only", "run_after", "disabled")


def rel(path: Path) -> str:
    if path.parent.name == "lsp-oracle-jaigen":
        return f"jaigen/{path.name}"  # wherever the target directory is
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def files_of(name: str, generated: Path, jaigen_count: int) -> list[Path]:
    if name == "stdlib":
        return sorted(p for p in (ROOT / "stdlib").rglob("*.jai") if "/tests/" in p.as_posix())
    if name == "compiler":
        skip = ("rejections",)
        return sorted(p for p in (ROOT / "tests/compiler").rglob("*.jai")
                      if not any(part in skip for part in p.relative_to(ROOT / "tests/compiler").parts))
    if name == "examples":
        return sorted((ROOT / "examples").rglob("*.jai"))
    if name == "upstream":
        cases = json.loads((ROOT / "tools/upstream-cases.json").read_text())
        out = []
        for case in cases:
            if case.get("review") == "pending" or case.get("mode") == "build":
                continue
            if any(key in case for key in UPSTREAM_SPECIAL):
                continue
            path = UPSTREAM / case["path"]
            if path.is_file() and path not in out:
                out.append(path)
        return out
    if name == "jaigen":
        generated.mkdir(parents=True, exist_ok=True)
        out = []
        for seed in range(jaigen_count):
            path = generated / f"jaigen_{seed}.jai"
            if not path.exists():
                subprocess.run([sys.executable, str(ROOT / "tools/differential/jaigen.py"), str(seed),
                                "--out", str(path)], check=True)
            out.append(path)
        return out
    raise SystemExit(f"unknown set {name}")


class Runner:
    """Runs jaic and the oracle, inside the sandbox for upstream files."""

    def __init__(self, args):
        self.args = args
        self.scratch = Path(tempfile.mkdtemp(prefix="lsp-oracle-"))
        self.sandbox = None
        self.sandboxed = False

    def use_sandbox(self):
        if upstream_sandbox.policy(self.args.no_sandbox):
            reads = upstream_sandbox.jaic_reads(str(self.args.jaic), ROOT / "stdlib", UPSTREAM)
            reads += [self.args.oracle.resolve().parent.parent, self.args.jaifmt.resolve().parent]
            self.sandbox = upstream_sandbox.make_sandbox(reads, [self.scratch])
        self.sandboxed = True

    def run(self, command, timeout, cwd=None):
        command, env = upstream_sandbox.confine(self.sandbox, self.scratch, [str(c) for c in command],
                                                dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib")))
        try:
            p = subprocess.run(command, capture_output=True, text=True, timeout=timeout, cwd=cwd, env=env,
                               stdin=subprocess.DEVNULL)
            return p.returncode, p.stdout, p.stderr
        except subprocess.TimeoutExpired as e:
            out = e.stdout.decode(errors="replace") if isinstance(e.stdout, bytes) else (e.stdout or "")
            return None, out, "timeout"


def accepted(runner: Runner, path: Path) -> bool:
    code, _, _ = runner.run([runner.args.jaic, path, "-check"], timeout=runner.args.check_timeout, cwd=path.parent)
    return code == 0


def oracle_chunk(runner: Runner, paths: list[Path]) -> list[dict]:
    """The oracle's reports for `paths`; a chunk that crashes or hangs is retried file by file."""
    a = runner.args
    command = [a.oracle, "--jaifmt", a.jaifmt, "--probes", str(a.probes), "--checks", a.checks]
    code, out, err = runner.run(command + [str(p) for p in paths], timeout=a.timeout * len(paths))
    reports = [json.loads(line) for line in out.splitlines() if line.startswith("{")]
    if code in (0, 1) and len(reports) == len(paths):
        return reports
    if len(paths) > 1:
        return [r for p in paths for r in oracle_chunk(runner, [p])]
    what = "hang" if code is None else f"exit {code}"
    tail = err.strip().splitlines()[-3:]
    return [{"path": str(paths[0]), "status": "crash", "detail": what,
             "findings": [{"check": "a-crash", "line": 0, "character": 0, "name": what,
                           "detail": " | ".join(tail)}], "dead": [], "counts": {}}]


def reached(runner: Runner, report: dict) -> list[dict]:
    """Runs the program with a marker at the start of each unreachable region that can hold one;
    a marker that prints is a finding. The original must run cleanly first."""
    path = Path(report["path"])
    probes = [d for d in report["dead"] if d.get("probe") is not None]
    if not probes or runner.sandboxed:
        return []
    code, _, _ = runner.run([runner.args.jaic, path, "-interp"], timeout=runner.args.check_timeout, cwd=path.parent)
    if code != 0:
        return []
    text = path.read_text()
    data = text.encode()
    findings = []
    for d in probes:
        at = len(data[: d["probe"]].decode(errors="replace"))
        marked = text[:at] + f' write_string("{MARKER}\\n"); ' + text[at:]
        probe = path.with_name(f".{path.stem}.lsp-oracle-probe.jai")
        try:
            probe.write_text(marked)
            code, out, _ = runner.run([runner.args.jaic, probe, "-interp"], timeout=runner.args.check_timeout,
                                      cwd=path.parent)
        finally:
            probe.unlink(missing_ok=True)
        if MARKER in out:
            findings.append({"check": "a-dead-ran", "line": d["line"], "character": 0, "name": "",
                             "detail": d["text"][:80]})
    return findings


def key(path: str, finding: dict) -> str:
    return f"{rel(Path(path))}|{finding['check']}|{finding['name']}"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--sets", default="stdlib,compiler,examples,jaigen", help=f"comma list of {', '.join(SETS)}")
    ap.add_argument("--jaic", type=Path, default=cargo_target_dir() / "release/jaic")
    ap.add_argument("--oracle", type=Path, default=cargo_target_dir() / "release/examples/lsp_oracle")
    ap.add_argument("--jaifmt", type=Path, default=jaifmt_path())
    ap.add_argument("--jaigen", type=int, default=20, help="jaigen programs (seeds 0..N)")
    ap.add_argument("--shard", default="1/1", help="I/N: every Nth file from the I-th")
    ap.add_argument("--jobs", type=int, default=2)
    ap.add_argument("--chunk", type=int, default=8)
    ap.add_argument("--probes", type=int, default=300)
    ap.add_argument("--checks", default="abcd")
    ap.add_argument("--timeout", type=int, default=120, help="seconds per file")
    ap.add_argument("--check-timeout", type=int, default=60)
    ap.add_argument("--out", type=Path, help="write reports.jsonl and summary.json here")
    ap.add_argument("--update-known", action="store_true", help="rewrite oracle-known.json with what was found")
    ap.add_argument("--no-sandbox", action="store_true", help="run upstream files unconfined (CI only)")
    ap.add_argument("files", nargs="*", type=Path, help="check these files instead of the sets")
    a = ap.parse_args()
    a.jaic, a.oracle, a.jaifmt = a.jaic.resolve(), a.oracle.resolve(), a.jaifmt.resolve()
    for tool in (a.jaic, a.oracle, a.jaifmt):
        if not tool.exists():
            raise SystemExit(f"missing {tool}")
    index, count = (int(x) for x in a.shard.split("/"))
    sets = [s for s in a.sets.split(",") if s]
    generated = cargo_target_dir() / "lsp-oracle-jaigen"
    groups = {"files": [p.resolve() for p in a.files]} if a.files else {s: files_of(s, generated, a.jaigen) for s in sets}

    reports = []
    for name, paths in groups.items():
        paths = paths[index - 1::count]
        runner = Runner(a)
        if name == "upstream":
            runner.use_sandbox()
        with concurrent.futures.ThreadPoolExecutor(a.jobs) as pool:
            ok = [p for p, good in zip(paths, pool.map(lambda p: accepted(runner, p), paths)) if good]
            print(f"{name}: {len(ok)} of {len(paths)} files pass jaic -check", flush=True)
            chunks = [ok[i:i + a.chunk] for i in range(0, len(ok), a.chunk)]
            for chunk_reports in pool.map(lambda c: oracle_chunk(runner, c), chunks):
                for r in chunk_reports:
                    r["findings"] += reached(runner, r)
                    r["set"] = name
                    reports.append(r)
        shutil.rmtree(runner.scratch, ignore_errors=True)

    known = set(json.loads(KNOWN.read_text())) if KNOWN.exists() else set()
    found = {key(r["path"], f): (r, f) for r in reports for f in r["findings"]}
    new = sorted(k for k in found if k not in known)
    checked = [rel(Path(r["path"])) for r in reports]
    stale = sorted(k for k in known if k.split("|")[0] in checked and k not in found)
    for k in new:
        r, f = found[k]
        print(f"NEW {rel(Path(r['path']))}:{f['line']}:{f['character']} {f['check']} {f['name']}: {f['detail'][:300]}")
    for k in stale:
        print(f"no longer found (remove from oracle-known.json): {k}")
    statuses = {}
    for r in reports:
        statuses[r["status"]] = statuses.get(r["status"], 0) + 1
    by_check = {}
    for k in found:
        c = k.split("|")[1]
        by_check[c] = by_check.get(c, 0) + 1
    summary = {"files": len(reports), "statuses": statuses, "findings": by_check, "new": len(new), "stale": len(stale)}
    print(json.dumps(summary))
    if a.out:
        a.out.mkdir(parents=True, exist_ok=True)
        with open(a.out / "reports.jsonl", "w") as f:
            for r in reports:
                f.write(json.dumps(r) + "\n")
        (a.out / "summary.json").write_text(json.dumps(summary, indent=1))
    if a.update_known:
        keep = {k for k in known if k.split("|")[0] not in checked}
        KNOWN.write_text(json.dumps(sorted(keep | set(found)), indent=1) + "\n")
    return 1 if new else 0


if __name__ == "__main__":
    sys.exit(main())
