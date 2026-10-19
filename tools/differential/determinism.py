#!/usr/bin/env python3
"""Check that the compiler's observable output does not depend on threads, unit counts or hash order.

Usage: tools/differential/determinism.py [--jaic PATH] [--jailsp PATH] [--repeat N] [--filter TEXT]
                                          [--jobs N] [--no-build] [--no-lsp] SET...
Sets: positive (tests/corpus/positive), negative (tests/corpus/negative), regressions
(tests/compiler/regressions), semantics (tests/compiler/semantics) or file paths.

What is compared (docs/tools/determinism.md):

- `check`: `jaic -check` diagnostics (text and order), stdout and exit code across `-parse_threads 1`,
  `-serial_parse`, `-parse_threads 8`, `-threads 1` and the default, each run `--repeat` times. Rust's
  hash maps are seeded per process, so a repeat catches output that follows a map's iteration order.
- `build`: for each `-codegen_units N`, the LLVM IR (`-emit_ir`) and the linked executable's bytes across the
  parse settings and repeats, and the executable's run output across all N (the split changes the objects, never
  what the program does). Only the sets that compile (positive, regressions, semantics).
- `lsp`: one `jailsp` session per run opening each file; the pushed diagnostics, document symbols, semantic
  tokens, inlay hints and hovers must be equal across runs.

Exit status 1 when any comparison differs.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import hashlib
import shutil
import json
import subprocess
import sys
import tempfile
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from toolkit.cli import add_jaic_argument
from toolkit.lsp import Lsp
from toolkit.repo import ROOT, stdlib_env

SETS = {
    "positive": ROOT / "tests/corpus/positive",
    "negative": ROOT / "tests/corpus/negative",
    "regressions": ROOT / "tests/compiler/regressions",
    "semantics": ROOT / "tests/compiler/semantics",
}
BUILDABLE = {"positive", "regressions", "semantics"}

# The parse settings every output must agree across; the first is the reference.
PARSE_CONFIGS = [
    ["-parse_threads", "1"],
    ["-serial_parse"],
    ["-parse_threads", "8"],
    ["-threads", "1"],
    [],
]
UNIT_COUNTS = ["1", "3"]
# The debug map of a linked macOS executable holds its objects' modification times, so the optimized
# build (which deletes its objects) is linked without debug info; the default build is compared by its kept objects.
LEVELS = [[], ["-O2", "-no_debug_info"]]
TIMEOUT = 120
# Programs whose output holds a host address by design (a pointer the compile-time interpreter returned is
# written into the program); their IR legitimately differs between runs, so only their diagnostics are compared.
HOST_ADDRESS_CASES = {"ctdata-opaque-pointer.jai", "rules-compiler-module.jai"}


def run(argv, cwd, timeout=TIMEOUT):
    """(exit code, stdout, stderr) of a command; a timeout is reported as a result, not raised."""
    try:
        p = subprocess.run(argv, cwd=cwd, capture_output=True, env=stdlib_env(), timeout=timeout)
    except subprocess.TimeoutExpired:
        return (-1, b"", b"timeout")
    return (p.returncode, p.stdout, p.stderr)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()[:16]


class Report:
    def __init__(self):
        self.lock = threading.Lock()
        self.failures: list[str] = []
        self.checked = 0

    def compare(self, label: str, results: list[tuple[str, object]]):
        """`results` are (config, value) pairs; they must all equal the first."""
        with self.lock:
            self.checked += 1
        first_name, first = results[0]
        for name, value in results[1:]:
            if value != first:
                with self.lock:
                    self.failures.append(f"{label}: `{name}` differs from `{first_name}`\n{describe(first, value)}")
                return


def describe(a, b) -> str:
    if isinstance(a, tuple) and isinstance(b, tuple):
        lines = []
        for i, (x, y) in enumerate(zip(a, b)):
            if x != y:
                sx = x.decode(errors="replace") if isinstance(x, bytes) else str(x)
                sy = y.decode(errors="replace") if isinstance(y, bytes) else str(y)
                lines.append(f"  field {i}:\n    - {sx[:600]!r}\n    + {sy[:600]!r}")
        return "\n".join(lines)
    return f"  {str(a)[:600]!r}\n  {str(b)[:600]!r}"


def check_case(jaic, path: Path, repeat: int, report: Report):
    results = []
    for i in range(repeat):
        results.append((f"{' '.join(PARSE_CONFIGS[0])} run {i + 1}", run([jaic, "-check", "-no_color", path.name, *PARSE_CONFIGS[0]], path.parent)))
    for config in PARSE_CONFIGS[1:]:
        for i in range(repeat):
            results.append((f"{' '.join(config) or 'default'} run {i + 1}", run([jaic, "-check", "-no_color", path.name, *config], path.parent)))
    report.compare(f"check {path.name}", results)


def build_case(jaic, path: Path, repeat: int, report: Report):
    outputs = []  # (build, run output) for the cross-unit comparison
    with tempfile.TemporaryDirectory(prefix="determinism-") as scratch:
        scratch = Path(scratch)
        # The same names in every build: the output path ends up inside the executable.
        exe, ll, kept = scratch / "program", scratch / "program.ll", scratch / ".build"
        for level in LEVELS:
            for units in UNIT_COUNTS:
                builds = []
                for config in PARSE_CONFIGS[:3]:
                    for i in range(repeat):
                        exe.unlink(missing_ok=True)
                        ll.unlink(missing_ok=True)
                        shutil.rmtree(kept, ignore_errors=True)
                        argv = [jaic, path.name, "-no_color", "-exe", exe, "-emit_ir", ll, "-codegen_units", units, *config, *level]
                        code, out, err = run(argv, path.parent)
                        # An unoptimized macOS executable carries its objects' modification times in the
                        # debug map and keeps the objects (`.build/`): compare those instead of the link.
                        objects = sorted(kept.glob("*.o")) if kept.is_dir() else []
                        linked = b"".join(o.read_bytes() for o in objects) if objects else (exe.read_bytes() if exe.exists() else b"")
                        value = (code, out, err, digest(ll.read_bytes()) if ll.exists() else "", digest(linked))
                        name = f"{' '.join(level)} -codegen_units {units} {' '.join(config) or 'default'} run {i + 1}"
                        builds.append((name, value))
                        if config == PARSE_CONFIGS[0] and i == 0 and exe.exists():
                            outputs.append((f"{' '.join(level)} -codegen_units {units}", run([exe], scratch, timeout=20)))
                report.compare(f"build {path.name}", builds)
        if outputs:
            report.compare(f"run {path.name}", outputs)


# The language server


def lsp_session(jailsp, path: Path) -> str:
    """Everything the checks compare for one document, as canonical JSON."""
    text = path.read_text(errors="replace")
    uri = path.resolve().as_uri()
    client = Lsp(jailsp, path.parent, stdlib_env())
    try:
        client.request("initialize", {"processId": None, "rootUri": path.parent.resolve().as_uri(), "capabilities": {}})
        client.notify("initialized", {})
        client.notify("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "jai", "version": 1, "text": text}})
        document = {"textDocument": {"uri": uri}}
        lines = text.split("\n")
        answers = {"diagnostics": client.diagnostics(uri)}
        answers["symbols"] = client.request("textDocument/documentSymbol", document)
        answers["tokens"] = client.request("textDocument/semanticTokens/full", document)
        answers["hints"] = client.request(
            "textDocument/inlayHint", {**document, "range": {"start": {"line": 0, "character": 0}, "end": {"line": len(lines), "character": 0}}}
        )
        hovers = []
        for number, line in enumerate(lines[:400]):
            column = len(line) - len(line.lstrip())
            if line.strip():
                hovers.append(client.request("textDocument/hover", {**document, "position": {"line": number, "character": column + 1}}))
        answers["hovers"] = hovers
        return json.dumps(answers, sort_keys=True)
    finally:
        client.close()


def lsp_case(jailsp, path: Path, repeat: int, report: Report):
    sessions = []
    for i in range(max(repeat, 2)):
        try:
            sessions.append((f"session {i + 1}", (lsp_session(jailsp, path),)))
        except Exception as error:  # a crash or timeout in one session is itself a difference
            sessions.append((f"session {i + 1}", (f"failed: {error!r}",)))
    report.compare(f"lsp {path.name}", sessions)


# The driver


def collect(names, name_filter):
    cases = []
    for name in names:
        if name in SETS:
            for p in sorted(SETS[name].glob("*.jai")):
                cases.append((name, p))
        else:
            cases.append(("path", Path(name).resolve()))
    return [(s, p) for s, p in cases if not name_filter or name_filter in p.name]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    add_jaic_argument(parser, profile="release")
    parser.add_argument("--jailsp", type=Path, default=None, help="the language server (default: next to --jaic)")
    parser.add_argument("--repeat", type=int, default=2, help="runs per configuration")
    parser.add_argument("--filter", default="", help="only files whose name contains this")
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--no-lsp", action="store_true")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("sets", nargs="+")
    args = parser.parse_args()
    jaic = args.jaic.resolve()
    jailsp = (args.jailsp or jaic.with_name("jailsp" + jaic.suffix)).resolve()
    report = Report()
    cases = collect(args.sets, args.filter)

    def work(item):
        which, path = item
        check_case(jaic, path, args.repeat, report)
        if not args.no_build and which in BUILDABLE | {"path"} and path.name not in HOST_ADDRESS_CASES:
            build_case(jaic, path, args.repeat, report)
        if not args.no_lsp:
            lsp_case(jailsp, path, args.repeat, report)

    with ThreadPoolExecutor(args.jobs) as pool:
        list(pool.map(work, cases))
    print(f"{len(cases)} files, {report.checked} comparisons, {len(report.failures)} differences", file=sys.stderr)
    for failure in report.failures:
        print(failure)
    return 1 if report.failures else 0


if __name__ == "__main__":
    sys.exit(main())
