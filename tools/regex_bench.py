#!/usr/bin/env python3
"""Benchmark jaic's Extensions/Regex against other regex engines.

    python3 tools/regex_bench.py [--engines jai,rust,re2,pcre2,ctre,hyperscan] [--cases date,email,...]
                                 [--runs 7] [--size-mb 16] [--skip-build] [--no-write] [--list]

Generates the corpus (benchmarks/regex/corpus.py), builds every harness under benchmarks/regex/,
runs the engines one after another (never in parallel) and prints a Markdown report, which it also
writes to benchmarks/results/regex.md. Build products live in benchmarks/regex/target (gitignored).
An engine whose library or toolchain is missing is skipped with a note.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import os
import platform
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BENCH = ROOT / "benchmarks" / "regex"
TARGET = BENCH / "target"
RESULTS = ROOT / "benchmarks" / "results" / "regex.md"
CTRE_TAG = "v3.9.0"
CTRE_COMMIT = "eb9577aae3515d14e6c5564f9aeb046d2e7c1124"
CTRE_URL = "https://github.com/hanickadot/compile-time-regular-expressions"

# Columns of the report, in order: id -> (label, harness group).
ENGINES = {
    "jai-static": ("Jai `Regex.Static`", "jai"),
    "jai-runtime": ("Jai `Regex.compile`", "jai"),
    "rust-regex": ("Rust `regex`", "rust"),
    "rust-dfa": ("Rust dense DFA", "rust"),
    "re2": ("RE2", "re2"),
    "pcre2-jit": ("PCRE2 JIT", "pcre2"),
    "ctre": ("CTRE", "ctre"),
    "hyperscan": ("Hyperscan", "hyperscan"),
}
GROUPS = ["jai", "rust", "re2", "pcre2", "ctre", "hyperscan"]
REFERENCE = "rust-regex"  # the engine whose counts the others are compared with

# Known semantic differences, shown as footnotes whenever the engine appears in the report.
ENGINE_NOTES = {
    "jai-static": "Patterns compile while the program compiles (`Regex.Static`); counts use `find_all`, captures use typed `captures_all`.",
    "jai-runtime": "`Regex.compile` at run time; counts use `find_all`, captures use `captures_all` and `group`.",
    "rust-regex": "`find_iter` / `captures_iter` (the reference for correctness).",
    "rust-dfa": "regex-automata dense DFA built by build.rs and deserialized zero-copy; unanchored forward search, match ends only. No Unicode `\\b`, no capture groups.",
    "re2": "RE2's `\\w` and `\\b` are ASCII-only (no Unicode word classes), so `\\b\\w+\\b` splits words at accented letters.",
    "pcre2-jit": "`PCRE2_UTF | PCRE2_UCP`, `pcre2_jit_match`: Unicode `\\w`, `\\b` and `\\p{..}`.",
    "ctre": "Header-only, compiled by the C++ compiler; UTF-8 input with `ctre-unicode.hpp`. CTRE's `\\w` and `\\b` are ASCII-only. Compile time is the C++ build.",
    "hyperscan": "Block mode, UTF-8 + UCP, start-of-match reporting with non-overlapping filtering, since Hyperscan reports every match end. When UCP fails (a pattern too large, `\\b`), ASCII classes are used and noted.",
}


@dataclass
class Case:
    id: str
    pattern: str
    description: str


@dataclass
class Row:
    engine: str
    case: str
    status: str
    count: int
    check: int
    mbps: float
    compile_us: float
    note: str


@dataclass
class Report:
    rows: dict[tuple[str, str], Row] = field(default_factory=dict)
    skipped: dict[str, str] = field(default_factory=dict)  # group -> reason
    build_seconds: dict[str, float] = field(default_factory=dict)
    versions: dict[str, str] = field(default_factory=dict)


def load_cases() -> list[Case]:
    cases = []
    for line in (BENCH / "cases.tsv").read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        cases.append(Case(parts[0], parts[1], parts[2] if len(parts) > 2 else ""))
    return cases


def run(cmd, **kw) -> subprocess.CompletedProcess:
    return subprocess.run([str(c) for c in cmd], text=True, capture_output=True, **kw)


def must(cmd, what: str, **kw) -> str:
    r = run(cmd, **kw)
    if r.returncode != 0:
        raise RuntimeError(f"{what} failed:\n{(r.stdout + r.stderr).strip()[-2000:]}")
    return r.stdout


def prefixes() -> list[Path]:
    found = []
    brew = shutil.which("brew")
    if brew:
        r = run([brew, "--prefix"])
        if r.returncode == 0:
            found.append(Path(r.stdout.strip()))
    found += [Path("/opt/homebrew"), Path("/usr/local"), Path("/usr")]
    return [p for p in dict.fromkeys(found) if p.exists()]


def brew_version(formula: str) -> str:
    brew = shutil.which("brew")
    if brew:
        r = run([brew, "list", "--versions", formula])
        if r.returncode == 0 and r.stdout.strip():
            return r.stdout.strip().split(None, 1)[-1]
    return "installed"


def find_header(name: str) -> Path | None:
    for p in prefixes():
        if (p / "include" / name).exists():
            return p
    return None


def cxx() -> str | None:
    return os.environ.get("CXX") or shutil.which("clang++") or shutil.which("c++") or shutil.which("g++")


def find_jaic(explicit: str | None) -> Path:
    candidates = [explicit, os.environ.get("JAIC")]
    cargo_target = os.environ.get("CARGO_TARGET_DIR")
    if cargo_target:
        candidates.append(str(Path(cargo_target) / "release" / "jaic"))
    candidates += [str(ROOT / "target" / "release" / "jaic"), shutil.which("jaic")]
    for c in candidates:
        if c and Path(c).exists():
            return Path(c)
    print("jaic not found; building it with cargo (release)", file=sys.stderr)
    must(["cargo", "build", "-q", "--release", "-p", "jaic-cli"], "cargo build jaic", cwd=ROOT)
    base = Path(cargo_target) if cargo_target else ROOT / "target"
    return base / "release" / "jaic"


# ---- build and run one harness group -------------------------------------------------------


def build_and_run(group: str, ctx: dict, corpus: Path, runs: int, cases: list[str], report: Report) -> list[Row]:
    """Builds a harness group (recording build time) and returns its parsed RESULT rows."""
    t0 = time.monotonic()
    exe: Path
    extra: list[str] = []
    bin_dir = TARGET / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    if group == "jai":
        exe = bin_dir / "jai-bench"
        if not (ctx["skip_build"] and exe.exists()):
            must([ctx["jaic"], BENCH / "jai" / "bench.jai", "-O2", "-exe", exe], "jaic", cwd=ROOT)
        report.versions["jaic"] = run([ctx["jaic"], "--version"]).stdout.strip().removeprefix("jaic ") or "unknown"
    elif group == "rust":
        env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET / "rust"))
        exe = TARGET / "rust" / "release" / "regex-bench"
        if not (ctx["skip_build"] and exe.exists()):
            r = run(["cargo", "build", "-q", "--release", "--locked"], cwd=BENCH / "rust", env=env)
            if r.returncode != 0:
                raise RuntimeError(f"cargo build failed:\n{r.stderr[-2000:]}")
        lock = (BENCH / "rust" / "Cargo.lock").read_text()
        for crate in ("regex", "regex-automata"):
            m = re.search(rf'name = "{crate}"\nversion = "([^"]+)"', lock)
            if m:
                report.versions[crate] = m.group(1)
    else:
        compiler = ctx["cxx"]
        flags = ["-O2", "-std=c++20", "-w"]
        src = BENCH / "cpp" / {"re2": "re2.cpp", "pcre2": "pcre2.cpp", "ctre": "ctre.cpp", "hyperscan": "hyperscan.cpp"}[group]
        exe = bin_dir / group
        link: list[str] = []
        if group == "re2":
            p = ctx["re2"]
            flags += [f"-I{p}/include"]
            link = [f"-L{p}/lib", "-lre2", f"-Wl,-rpath,{p}/lib"]
            report.versions["RE2"] = brew_version("re2")
        elif group == "pcre2":
            p = ctx["pcre2"]
            flags += [f"-I{p}/include"]
            link = [f"-L{p}/lib", "-lpcre2-8", f"-Wl,-rpath,{p}/lib"]
            r = run([p / "bin" / "pcre2-config", "--version"]) if (p / "bin" / "pcre2-config").exists() else None
            report.versions["PCRE2"] = r.stdout.strip() if r and r.returncode == 0 else "installed"
        elif group == "hyperscan":
            p = ctx["hyperscan"]
            flags += [f"-I{p}/include"]
            link = [f"-L{p}/lib", "-lhs", f"-Wl,-rpath,{p}/lib"]
            report.versions["Hyperscan"] = brew_version("vectorscan") or brew_version("hyperscan")
        elif group == "ctre":
            flags += [f"-I{ctx['ctre']}/include", "-Wno-missing-template-arg-list-after-template-kw"]
            report.versions["CTRE"] = CTRE_TAG
        if not (ctx["skip_build"] and exe.exists()):
            must([compiler, *flags, src, "-o", exe, *link], f"{compiler} {src.name}")
        extra = [str(BENCH / "cases.tsv")]
    report.build_seconds[group] = time.monotonic() - t0

    cmd = [exe, corpus, str(runs), *extra, *cases]
    r = run(cmd, timeout=ctx["timeout"])
    if r.returncode != 0:
        raise RuntimeError(f"{exe.name} exited with {r.returncode}:\n{(r.stdout + r.stderr)[-1500:]}")
    rows = []
    for line in r.stdout.splitlines():
        f = line.split("\t")
        if len(f) >= 8 and f[0] == "RESULT":
            rows.append(Row(f[1], f[2], f[3], int(f[4]), int(f[5]), float(f[6]), float(f[7]), f[8] if len(f) > 8 else ""))
    return rows


def prepare_ctre() -> Path | None:
    """Fetches the pinned CTRE release into benchmarks/regex/target/ctre (not vendored)."""
    dest = TARGET / "ctre"
    if not (dest / "include" / "ctre-unicode.hpp").exists():
        if not shutil.which("git"):
            return None
        if dest.exists():
            shutil.rmtree(dest)
        r = run(["git", "clone", "-q", "--depth", "1", "--branch", CTRE_TAG, CTRE_URL, dest])
        if r.returncode != 0:
            print(f"ctre: fetch failed: {r.stderr.strip()[-300:]}", file=sys.stderr)
            return None
    head = run(["git", "-C", dest, "rev-parse", "HEAD"]).stdout.strip()
    if head != CTRE_COMMIT:
        print(f"ctre: checkout is {head}, expected {CTRE_COMMIT}; skipping", file=sys.stderr)
        return None
    return dest


# ---- report ---------------------------------------------------------------------------------


def machine() -> str:
    cpu = ""
    if sys.platform == "darwin":
        cpu = run(["sysctl", "-n", "machdep.cpu.brand_string"]).stdout.strip()
        mem = run(["sysctl", "-n", "hw.memsize"]).stdout.strip()
        mem_gib = f", {int(mem) // 2**30} GiB" if mem.isdigit() else ""
        cores = run(["sysctl", "-n", "hw.ncpu"]).stdout.strip()
        return f"{cpu} ({cores} cores{mem_gib}), macOS {platform.mac_ver()[0]}, {platform.machine()}"
    if Path("/proc/cpuinfo").exists():
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                cpu = line.split(":", 1)[1].strip()
                break
    return f"{cpu or platform.processor() or 'unknown CPU'}, {platform.system()} {platform.release()}, {platform.machine()}"


def fmt_speed(mbps: float) -> str:
    if mbps >= 10000:
        return f"{mbps:,.0f}"
    if mbps >= 100:
        return f"{mbps:,.0f}"
    return f"{mbps:.1f}"


def fmt_time(us: float) -> str:
    if us <= 0:
        return "-"
    if us >= 1000:
        return f"{us / 1000:.2f} ms"
    return f"{us:.1f} µs"


def build_report(report: Report, cases: list[Case], engines: list[str], args, corpus: Path) -> tuple[str, list[str]]:
    footnotes: list[str] = []  # dynamic footnotes, in order of first use
    marks: dict[str, int] = {}

    def mark(text: str) -> str:
        if text not in marks:
            footnotes.append(text)
            marks[text] = len(footnotes)
        return f"[^{marks[text]}]"

    problems: list[str] = []

    def cell(case: Case, eng: str, kind: str) -> str:
        row = report.rows.get((eng, case.id))
        group = ENGINES[eng][1]
        if row is None:
            if group in report.skipped:
                return "skipped"
            return "-"
        if row.status != "ok":
            return "n/a" + mark(f"{ENGINES[eng][0]} on `{case.id}`: {row.note}")
        text = fmt_speed(row.mbps)
        ref = report.rows.get((REFERENCE, case.id))
        if ref and ref.status == "ok" and eng != REFERENCE and (row.count != ref.count or row.check != ref.check):
            msg = (
                f"MISMATCH {ENGINES[eng][0]} on `{case.id}`: {row.count} matches"
                + (f", checksum {row.check}" if row.check or ref.check else "")
                + f" against {ref.count}"
                + (f", checksum {ref.check}" if row.check or ref.check else "")
                + f" from {ENGINES[REFERENCE][0]} (different `\\w` or `\\b` definition; see Notes)"
            )
            problems.append(msg)
            text = f"**{text} ⚠**" + mark(msg)
        if row.note and row.status == "ok":
            text += mark(f"{ENGINES[eng][0]} on `{case.id}`: {row.note}")
        return text

    out: list[str] = []
    labels = [ENGINES[e][0] for e in engines]
    aligns = "| --- | --- | " + " | ".join(["---:"] * len(engines)) + " |"

    def table(title: str, subset: list[Case], kind: str) -> None:
        out.append(f"### {title}\n")
        out.append("| Case | Pattern | " + " | ".join(labels) + " |")
        out.append(aligns)
        for c in subset:
            pat = c.pattern.replace("|", "\\|")
            cells = [cell(c, e, kind) for e in engines]
            out.append(f"| `{c.id}` | `{pat}` | " + " | ".join(cells) + " |")
        out.append("")

    plain = [c for c in cases if c.id != "caps"]
    caps = [c for c in cases if c.id == "caps"]
    out.append("## Results\n")
    out.append(
        f"Throughput in MB/s (1 MB = 10^6 bytes) over the {args.size_mb} MB corpus, best of {args.runs} runs; "
        "higher is better. **Bold with ⚠** marks a match count that differs from the Rust `regex` crate.\n"
    )
    if plain:
        table("Count all matches (MB/s)", plain, "count")
    if caps:
        table("Find every match and extract its three groups (MB/s)", caps, "caps")

    # match counts
    out.append("### Match counts\n")
    out.append("Matches found by the reference engine (Rust `regex`); every other engine's count is compared with it.\n")
    out.append("| Case | Matches |")
    out.append("| --- | ---: |")
    for c in cases:
        ref = report.rows.get((REFERENCE, c.id))
        out.append(f"| `{c.id}` | {ref.count:,} |" if ref else f"| `{c.id}` | - |")
    out.append("")

    # compile time
    runtime_engines = [e for e in engines if e in ("jai-runtime", "rust-regex", "re2", "pcre2-jit", "hyperscan")]
    if runtime_engines:
        out.append("### Pattern compile time (run-time engines, best of 20)\n")
        out.append("| Case | " + " | ".join(ENGINES[e][0] for e in runtime_engines) + " |")
        out.append("| --- | " + " | ".join(["---:"] * len(runtime_engines)) + " |")
        for c in cases:
            vals = []
            for e in runtime_engines:
                row = report.rows.get((e, c.id))
                vals.append(fmt_time(row.compile_us) if row and row.status == "ok" else "n/a" if row else "-")
            out.append(f"| `{c.id}` | " + " | ".join(vals) + " |")
        out.append("")
        out.append(
            "Precompiled variants (`Regex.Static`, the dense DFA, CTRE) pay this at build time instead: "
            "the Jai compiler, build.rs or the C++ compiler.\n"
        )

    # build time
    out.append("### Harness build time\n")
    out.append("| Harness | Seconds |")
    out.append("| --- | ---: |")
    for g in GROUPS:
        if g in report.build_seconds:
            out.append(f"| {g} | {report.build_seconds[g]:.1f} |")
    out.append("")
    out.append(
        "Includes compiling every pattern for the precompiled engines (Jai `Regex.Static`, CTRE, the Rust dense DFAs). "
        "Rust is a cold build when `benchmarks/regex/target` is empty.\n"
    )

    out.append("## Notes\n")
    for e in engines:
        out.append(f"- **{ENGINES[e][0]}**: {ENGINE_NOTES[e]}")
    for g, why in report.skipped.items():
        out.append(f"- **Skipped `{g}`**: {why}")
    out.append(
        "- **Not included**: Zig has no regular-expression engine in its standard library and is not installed here, "
        "so it is left out. `std::regex` is left out as well: libc++'s ECMAScript backtracking matcher recurses per "
        "character (stack overflow on a 16 MB subject), has no `\\p{Greek}` and no Unicode `\\w` on `char`, and is "
        "orders of magnitude slower than every engine above."
    )
    out.append("")
    if footnotes:
        out.append("## Footnotes\n")
        for i, f in enumerate(footnotes, 1):
            out.append(f"[^{i}]: {f}")
        out.append("")
    return "\n".join(out), problems


def header(report: Report, args, corpus: Path) -> str:
    data = corpus.read_bytes()
    sha = hashlib.sha256(data).hexdigest()[:16]
    commit = run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT).stdout.strip()
    ver = ", ".join(f"{k} {v}" for k, v in report.versions.items())
    lines = [
        "# Regex engine benchmark",
        "",
        f"- Machine: {machine()}",
        f"- Date: {datetime.date.today().isoformat()}, repository commit {commit}",
        f"- Corpus: {len(data):,} bytes, seed {args.seed}, sha256 `{sha}` (generated by `benchmarks/regex/corpus.py`; "
        "7/8 English-like ASCII with dates, e-mails and names, 1/8 Greek, Cyrillic, CJK and accented Latin)",
        f"- Versions: {ver}",
        "- Engines run one after another, one process each; best of "
        f"{args.runs} timed runs after the file is in memory. Regenerate with `python3 tools/regex_bench.py`.",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--engines", default=",".join(GROUPS), help=f"comma list of {GROUPS} or engine ids {list(ENGINES)}")
    ap.add_argument("--cases", default="", help="comma list of case ids (default: all in cases.tsv)")
    ap.add_argument("--runs", type=int, default=7, help="timed runs per case; the best is reported")
    ap.add_argument("--size-mb", type=float, default=16)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--jaic", help="path to jaic (default: $JAIC, $CARGO_TARGET_DIR/release/jaic, target/release/jaic)")
    ap.add_argument("--skip-build", action="store_true", help="reuse harness binaries already built")
    ap.add_argument("--timeout", type=int, default=900, help="seconds allowed per harness run")
    ap.add_argument("--out", default=str(RESULTS))
    ap.add_argument("--no-write", action="store_true", help="print the report but do not write the results file")
    ap.add_argument("--list", action="store_true", help="list engines and cases, then exit")
    args = ap.parse_args()

    all_cases = load_cases()
    if args.list:
        print("engines:", ", ".join(ENGINES))
        for c in all_cases:
            print(f"  {c.id:8} {c.pattern}   {c.description}")
        return 0

    wanted_cases = [c for c in args.cases.split(",") if c]
    unknown = [c for c in wanted_cases if c not in {x.id for x in all_cases}]
    if unknown:
        print(f"unknown cases: {unknown}", file=sys.stderr)
        return 2
    cases = [c for c in all_cases if not wanted_cases or c.id in wanted_cases]

    groups: list[str] = []
    shown: list[str] = []
    for name in [e for e in args.engines.split(",") if e]:
        if name in GROUPS:
            groups.append(name)
            shown += [e for e, (_, g) in ENGINES.items() if g == name]
        elif name in ENGINES:
            groups.append(ENGINES[name][1])
            shown.append(name)
        else:
            print(f"unknown engine: {name}", file=sys.stderr)
            return 2
    groups = [g for g in GROUPS if g in groups]
    engines = [e for e in ENGINES if e in shown]

    TARGET.mkdir(parents=True, exist_ok=True)
    corpus = TARGET / f"corpus-{args.size_mb:g}mb-seed{args.seed}.txt"
    if not corpus.exists():
        must(["python3", "-I", BENCH / "corpus.py", corpus, "--size-mb", args.size_mb, "--seed", args.seed], "corpus")
    print(f"corpus: {corpus} ({corpus.stat().st_size:,} bytes)", file=sys.stderr)

    report = Report()
    ctx: dict = {"skip_build": args.skip_build, "timeout": args.timeout, "cxx": cxx()}
    for g in groups:
        # Preconditions: skip an engine whose toolchain or library is missing.
        if g == "jai":
            ctx["jaic"] = find_jaic(args.jaic)
        elif g == "rust" and not shutil.which("cargo"):
            report.skipped[g] = "cargo not found."
        elif g in ("re2", "pcre2", "ctre", "hyperscan") and not ctx["cxx"]:
            report.skipped[g] = "no C++ compiler found (set CXX)."
        elif g == "re2":
            p = find_header("re2/re2.h")
            if p is None:
                report.skipped[g] = "RE2 headers not found (`brew install re2`)."
            ctx["re2"] = p
        elif g == "pcre2":
            p = find_header("pcre2.h")
            if p is None:
                report.skipped[g] = "PCRE2 headers not found (`brew install pcre2`)."
            ctx["pcre2"] = p
        elif g == "hyperscan":
            p = find_header("hs/hs.h")
            if p is None:
                report.skipped[g] = "Hyperscan/vectorscan headers not found (`brew install vectorscan`, or hyperscan on x86)."
            ctx["hyperscan"] = p
        elif g == "ctre":
            ctre = prepare_ctre()
            if ctre is None:
                report.skipped[g] = f"could not fetch CTRE {CTRE_TAG} ({CTRE_URL}); needs git and network access."
            ctx["ctre"] = ctre
        if g in report.skipped:
            print(f"skipping {g}: {report.skipped[g]}", file=sys.stderr)
            continue
        print(f"== {g}", file=sys.stderr)
        try:
            rows = build_and_run(g, ctx, corpus, args.runs, [c.id for c in cases], report)
        except Exception as e:  # keep going: report the failure and continue with the other engines
            report.skipped[g] = f"build or run failed: {str(e).splitlines()[0] if str(e) else e}"
            print(f"{g}: {e}", file=sys.stderr)
            continue
        for row in rows:
            report.rows[(row.engine, row.case)] = row

    if REFERENCE not in {e for e, _ in report.rows}:
        print("warning: the reference engine (rust-regex) did not run, so counts are not cross-checked", file=sys.stderr)

    body, problems = build_report(report, cases, engines, args, corpus)
    text = header(report, args, corpus) + "\n" + body
    print(text)
    if problems:
        print("\n!!! CORRECTNESS MISMATCHES !!!", file=sys.stderr)
        for p in problems:
            print("  " + p, file=sys.stderr)
    if not args.no_write:
        out = Path(args.out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text if text.endswith("\n") else text + "\n")
        print(f"wrote {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
