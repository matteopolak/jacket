#!/usr/bin/env python3
"""Which public procedures of each stdlib module the runtime tests run, per module.

    stdlib_coverage.py --record cov.txt                   # report
    stdlib_coverage.py --record cov.txt --check           # and fail below the baseline
    stdlib_coverage.py --record cov.txt --update          # rewrite this platform's baseline
    stdlib_coverage.py --record cov.txt --uncovered File  # list what no test runs

The record is what `tools/stdlib_runtime.py --coverage cov.txt` writes: `jaic` (with
`JAIC_COVERAGE`) appends `path:line name` for every procedure the interpreter ran, compile-time
code included. A public procedure is one a module exports: declared at the top level of the
module's files (`stdlib/X.jai` or `stdlib/X/module.jai` and what they `#load`, every `#if` branch)
outside `#scope_file` and `#scope_module`, with a body (`#foreign` and other bodiless declarations
are not counted). It is covered when the record has its file and line, or, for procedures the
interpreter records without a line (`inline` and `#c_call` ones), its file and name. Macros
(`#expand`) are inlined and never run as procedures: one counts as covered when a runtime test's
source calls it by name. Files named for another OS (`windows.jai`, `native_posix.jai`,
`generated_macos.jai`...) are left out of a platform's count; `--all-platforms` keeps them.

`tests/stdlib-coverage.txt` holds the baseline: per platform, each module's covered count.
`--check` fails when a module covers fewer procedures than recorded there, so coverage can only
go up; `--update` writes the new counts after tests are added. See
docs/tools/stdlib-runtime-tests.md.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STDLIB = ROOT / "stdlib"
BASELINE = ROOT / "tests/stdlib-coverage.txt"
sys.path.insert(0, str(ROOT / "tools"))
import stdlib_runtime  # noqa: E402

DECL = re.compile(r"^\s*(operator\s*\S+?|[A-Za-z_][A-Za-z0-9_]*)\s*::\s*(.*)$")
PROC_START = re.compile(r"^(inline\s+|no_inline\s+)?\(")
BODILESS = ("#foreign", "#compiler", "#intrinsic", "#runtime_support", "#elsewhere")
LOAD = re.compile(r'#load\s+"([^"]+)"')
SCOPE = re.compile(r"^\s*#scope_(export|file|module)\b")


def modules():
    """{import name: entry file} for `stdlib/X.jai` and `stdlib/X/module.jai`."""
    found = {}
    for path in sorted(STDLIB.iterdir()):
        if path.is_dir() and (path / "module.jai").is_file():
            found[path.name] = path / "module.jai"
        elif path.suffix == ".jai" and path.is_file():
            found[path.stem] = path
    return found


def code_lines(text):
    """Each line with comments, string contents and here-strings blanked out (so braces and
    `::` in them do not count), keeping line numbers."""
    out, depth, here = [], 0, None
    for line in text.split("\n"):
        if here is not None:
            if line.strip() == here:
                here = None
            out.append("")
            continue
        result, i, n = [], 0, len(line)
        while i < n:
            c = line[i]
            if depth:
                if line.startswith("*/", i):
                    depth -= 1
                    i += 2
                elif line.startswith("/*", i):
                    depth += 1
                    i += 2
                else:
                    i += 1
                continue
            if line.startswith("//", i):
                break
            if line.startswith("/*", i):
                depth += 1
                i += 2
                continue
            if c == '"':
                j = i + 1
                while j < n and line[j] != '"':
                    j += 2 if line[j] == "\\" else 1
                result.append('""')
                i = j + 1
                continue
            m = re.match(r"#string\s+(\S+)", line[i:])
            if m:
                here = m.group(1).rstrip(";")
                result.append('""')
                break
            result.append(c)
            i += 1
        out.append("".join(result))
    return out


def declarations(path, seen):
    """Public procedures of `path` and the files it loads, as dicts (file, line, name, macro)."""
    if path in seen or not path.is_file():
        return []
    seen.add(path)
    raw = path.read_text(encoding="utf-8", errors="replace")
    lines = code_lines(raw)
    loads = [path.parent / m.group(1) for m in LOAD.finditer(raw)]
    procs, stack, exported = [], [], True
    for number, line in enumerate(lines, 1):
        top = all(stack)
        if top:
            scope = SCOPE.match(line)
            if scope:
                exported = scope.group(1) == "export"
            decl = DECL.match(line)
            if decl and exported and PROC_START.match(decl.group(2)):
                header = header_text(lines, number - 1)
                if header is not None and not any(d in header for d in BODILESS):
                    procs.append({"file": path, "line": number, "name": decl.group(1).replace(" ", ""),
                                  "macro": "#expand" in header})
        # Braces: an `#if`/`else` block keeps its contents at the top level; any other is a body.
        stripped = line.strip()
        transparent = stripped.startswith(("#if", "} else", "else", "#else")) or stripped == "{"
        for c in line:
            if c == "{":
                stack.append(transparent and all(stack))
                transparent = False
            elif c == "}" and stack:
                stack.pop()
                # `} else {` or `} else #if ... {` reopen a transparent block.
                transparent = re.match(r"^\s*}\s*else\b", line) is not None
    for load in loads:
        procs += declarations(load.resolve(), seen)
    return procs


def header_text(lines, start):
    """The text of the declaration starting at line index `start` up to its body's `{`, or None
    when it ends with `;` first (a procedure type or a parenthesized constant, not a procedure)."""
    text, parens = [], 0
    for line in lines[start:start + 40]:
        for i, c in enumerate(line):
            if c == "(":
                parens += 1
            elif c == ")":
                parens -= 1
            elif parens == 0 and c == "{":
                return "".join(text) + line[:i]
            elif parens == 0 and c == ";":
                header = "".join(text) + line[:i]
                return header if any(d in header for d in BODILESS) or "=>" in header else None
        text.append(line + "\n")
    return None


# Words in a file name that tie it to OSes, and the OSes they name.
PLATFORM_WORDS = {
    "windows": {"windows"}, "win32": {"windows"}, "linux": {"linux"}, "android": {"linux"},
    "macos": {"macos"}, "osx": {"macos"}, "unix": {"linux", "macos"}, "posix": {"linux", "macos"},
}


def for_platform(path, os_name):
    """Whether a stdlib file belongs to `os_name`, judging by OS words in its name."""
    words = set(re.split(r"[^a-z0-9]+", path.stem.lower()))
    oses = set().union(*(PLATFORM_WORDS[w] for w in words if w in PLATFORM_WORDS))
    return not oses or os_name in oses


def normalize(path_text):
    """A recorded path relative to the repository's stdlib, or None for files outside it."""
    p = path_text.replace("\\", "/")
    root = STDLIB.as_posix()
    if p.lower().startswith(root.lower() + "/"):
        return p[len(root) + 1:]
    # A record made in another checkout: the part after its last `/stdlib/` that is not tests'.
    idx = p.rfind("/stdlib/")
    if idx >= 0 and not p[:idx].endswith("/tests"):
        return p[idx + len("/stdlib/"):]
    return None


def read_record(path):
    by_line, by_name = set(), set()
    for line in Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        m = re.match(r"^(.*):(\d+) (.*)$", line)
        if not m:
            continue
        rel = normalize(m.group(1))
        if rel is None:
            continue
        number = int(m.group(2))
        if number:
            by_line.add((rel, number))
        by_name.add((rel, m.group(3)))
    return by_line, by_name


def test_sources():
    return "\n".join(p.read_text(encoding="utf-8", errors="replace") for _, p in stdlib_runtime.cases())


def measure(record, os_name=None):
    """{module: [(procedure, covered)]}, leaving out files of OSes other than `os_name`."""
    by_line, by_name = read_record(record)
    sources = test_sources()
    result = {}
    for name, entry in modules().items():
        procs = []
        for proc in declarations(entry.resolve(), set()):
            if os_name and not for_platform(proc["file"], os_name):
                continue
            rel = proc["file"].relative_to(STDLIB.resolve()).as_posix()
            if proc["macro"]:
                covered = re.search(rf"\b{re.escape(proc['name'])}\s*\(", sources) is not None
            else:
                covered = (rel, proc["line"]) in by_line or (rel, proc["name"]) in by_name
            procs.append((proc, covered))
        result[name] = procs
    return result


def read_baseline(platform):
    counts, section = {}, None
    if not BASELINE.exists():
        return counts
    for line in BASELINE.read_text().splitlines():
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
        elif section == platform:
            module, count = line.split()
            counts[module] = int(count)
    return counts


def write_baseline(platform, counts):
    sections, order, current = {}, [], None
    header = []
    if BASELINE.exists():
        for line in BASELINE.read_text().splitlines():
            if line.startswith("[") and line.strip().endswith("]"):
                current = line.strip()[1:-1]
                order.append(current)
                sections[current] = []
            elif current is None:
                header.append(line)
            elif line.strip():
                sections[current].append(line)
    if not header:
        header = [
            "# Covered public procedures per stdlib module and platform: tools/stdlib_coverage.py --check",
            "# fails when a module covers fewer than listed here. Update a platform's section with",
            "# --update after adding tests (docs/tools/stdlib-runtime-tests.md).",
            "",
        ]
    if platform not in sections:
        order.append(platform)
    sections[platform] = [f"{m} {c}" for m, c in sorted(counts.items()) if c]
    out = list(header)
    for name in sorted(order):
        out += [f"[{name}]", *sections[name], ""]
    BASELINE.write_text("\n".join(out).rstrip("\n") + "\n")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--record", required=True, help="the file tools/stdlib_runtime.py --coverage wrote")
    ap.add_argument("--platform", default=stdlib_runtime.host_platform())
    ap.add_argument("--check", action="store_true", help="fail when a module covers fewer procedures than the baseline")
    ap.add_argument("--update", action="store_true", help="write this platform's counts to the baseline")
    ap.add_argument("--uncovered", metavar="MODULE", help="list the public procedures of MODULE no test runs")
    ap.add_argument("--all", action="store_true", help="list modules with nothing to cover too")
    ap.add_argument("--print-baseline", action="store_true", help="print this platform's baseline section")
    ap.add_argument("--all-platforms", action="store_true", help="count files named for other OSes too")
    args = ap.parse_args()
    result = measure(args.record, None if args.all_platforms else args.platform.split("-")[0])
    if args.uncovered:
        procs = result.get(args.uncovered)
        if procs is None:
            sys.exit(f"no stdlib module {args.uncovered!r}")
        for proc, covered in procs:
            if not covered:
                rel = proc["file"].relative_to(STDLIB.resolve()).as_posix()
                print(f"stdlib/{rel}:{proc['line']} {proc['name']}{' (macro)' if proc['macro'] else ''}")
        return 0
    counts = {m: sum(c for _, c in procs) for m, procs in result.items()}
    total = sum(len(p) for p in result.values())
    covered = sum(counts.values())
    print(f"{'module':<34} {'covered':>8} {'public':>7} {'%':>6}")
    for module, procs in sorted(result.items()):
        if not procs and not args.all:
            continue
        pct = 100 * counts[module] / len(procs) if procs else 100
        print(f"{module:<34} {counts[module]:>8} {len(procs):>7} {pct:>5.1f}%")
    print(f"{'all modules':<34} {covered:>8} {total:>7} {100 * covered / max(total, 1):>5.1f}%")
    if args.print_baseline:
        print(f"\nbaseline section for {BASELINE.relative_to(ROOT)}:\n[{args.platform}]")
        for module, count in sorted(counts.items()):
            if count:
                print(f"{module} {count}")
    if args.update:
        write_baseline(args.platform, counts)
        print(f"wrote the {args.platform} baseline to {BASELINE.relative_to(ROOT)}")
    if args.check:
        baseline = read_baseline(args.platform)
        if not baseline:
            print(f"no {args.platform} baseline in {BASELINE.relative_to(ROOT)}; nothing to check")
            return 0
        drops = [f"{m}: {counts.get(m, 0)} covered, baseline {n}" for m, n in sorted(baseline.items())
                 if counts.get(m, 0) < n]
        if drops:
            print(f"\ncoverage dropped below the {args.platform} baseline:")
            for line in drops:
                print(f"  {line}")
            return 1
        rises = [m for m, c in counts.items() if c > baseline.get(m, 0)]
        if rises:
            print(f"\n{len(rises)} modules cover more than the baseline; run with --update to raise it")
    return 0


if __name__ == "__main__":
    sys.exit(main())
