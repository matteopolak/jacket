#!/usr/bin/env python3
"""Run jaic over test programs and summarize results.

Usage: tools/jaic-sweep.py [--jaic PATH] [--filter TEXT] [--verbose] SET...
Sets: corpus (tests/corpus/positive with expected runtime), negative (tests/corpus/negative:
`jaic check` must fail and report the recorded text), stdlib (tests/stdlib,
run must succeed), modules (the stdlib's own tests: stdlib/tests and stdlib/<Module>/tests,
run must succeed; a test directory's `modules/` folder holds its mock modules), examples (tests/examples.json:
example programs such as examples/tour, whose stdout must contain the listed lines), upstream
(tools/upstream-cases.json: upstream project entry points that must pass), or file paths.
"""
import argparse, json, os, shutil, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UPSTREAM = ROOT / "corpus/upstream"
# Upstream cases with `copy` run in a scratch copy of those project directories (`link` names
# read-only siblings to symlink next to them), so a build that writes generated files never
# changes the pinned corpus. Case id -> scratch root, filled before the cases start.
WORKDIRS = {}

def upstream_root(c):
    return WORKDIRS.get(c["id"], UPSTREAM)

def make_workdir(c):
    root = Path(tempfile.mkdtemp(prefix=f"jaic-sweep-{c['id']}-"))
    for name in c.get("copy", []):
        shutil.copytree(UPSTREAM / name, root / name, symlinks=True)
    for name in c.get("link", []):
        (root / name).symlink_to(UPSTREAM / name, target_is_directory=True)
    return root

def cases(name):
    if name == "corpus":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if "runtime" in c:
                yield c["id"], ROOT / "tests/corpus" / c["source"], "run", c["runtime"], []
    elif name == "negative":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if c.get("kind") == "negative":
                yield c["id"], ROOT / "tests/corpus" / c["source"], "check", {"negative": next(iter(c["negative"].values()))}, []
    elif name == "stdlib":
        for p in sorted((ROOT / "tests/stdlib").glob("*.jai")):
            yield p.stem, p, "run", None, []
    elif name == "modules":
        found = list((ROOT / "stdlib").glob("*/tests/*.jai")) + list((ROOT / "stdlib/tests").glob("**/*.jai"))
        for p in sorted(set(found)):
            if "modules" not in p.relative_to(ROOT / "stdlib").parts:
                yield str(p.relative_to(ROOT / "stdlib").with_suffix("")), p, "run", None, []
    elif name == "examples":
        for c in json.loads((ROOT / "tests/examples.json").read_text())["cases"]:
            expect = {"contains": c["stdout_contains"], "excludes": c.get("stdout_excludes", [])}
            yield c["id"], ROOT / c["directory"] / c["main"], "run", expect, c.get("args", [])
    elif name == "upstream":
        for c in json.loads((ROOT / "tools/upstream-cases.json").read_text()):
            yield c["id"], upstream_root(c) / c["path"], c["mode"], None, c.get("args", [])
    else:
        p = Path(name)
        yield p.stem, p, "run", None, []

def use_native_libs():
    """Point jaic at the third-party C libraries (rpmalloc, stb_*) that some cases call or link,
    building any that are missing. Worktrees share the main checkout's artifacts/."""
    if os.environ.get("JAIC_NATIVE_LIBS") or sys.platform not in ("darwin", "linux"):
        return
    sys.path.insert(0, str(ROOT / "tools"))
    import build_native_libs
    if build_native_libs.missing():
        subprocess.run([sys.executable, str(ROOT / "tools/build_native_libs.py")], check=True)
    os.environ["JAIC_NATIVE_LIBS"] = str(build_native_libs.output_dir())

def physical_memory_gib():
    try:
        return os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES") / 2**30
    except (ValueError, OSError, AttributeError):
        return 16

def stale_sources(jaic):
    """A compiler source newer than the jaic binary: a stale build can lack limits that keep
    negative cases (unbounded recursion and the like) from exhausting memory."""
    try:
        built = jaic.stat().st_mtime
    except OSError:
        return None
    for crate in ("jaic", "jaic-cli", "jaic-llvm"):
        for f in (ROOT / "crates" / crate / "src").rglob("*.rs"):
            if f.stat().st_mtime > built:
                return f.relative_to(ROOT)
    return None

# jaic's exit status when JAIC_MEMORY_LIMIT stops it (`jaic::memory_limit::EXIT_CODE`).
MEMORY_LIMIT_EXIT = 120

def run_limited(command, cwd, timeout, limit_bytes):
    """Runs `command` with jaic's exact allocation limit armed, killing it on timeout. Returns
    (stdout, stderr, code), with code -1 and a one-line verdict as stderr when a limit stopped it."""
    env = dict(os.environ, JAIC_MEMORY_LIMIT=str(limit_bytes))
    # Output goes to files rather than pipes: a program the case launched may outlive a killed
    # jaic and would keep a pipe open.
    with tempfile.TemporaryFile() as out_file, tempfile.TemporaryFile() as err_file:
        proc = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                stdout=out_file, stderr=err_file)
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            return "", "timeout", -1
        out_file.seek(0)
        err_file.seek(0)
        out = out_file.read().decode(errors="replace")
        err = err_file.read().decode(errors="replace")
    if code == MEMORY_LIMIT_EXIT and "error: memory limit of" in err:
        return "", next(l for l in err.splitlines() if "error: memory limit of" in l), -1
    return out, err, code

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sets", nargs="+")
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument("--filter", default="")
    ap.add_argument("--verbose", "-v", action="store_true")
    ap.add_argument("--timeout", type=float, default=60)
    ap.add_argument("--memory-limit", type=float, default=3,
                    help="GiB a case may allocate before jaic stops it (JAIC_MEMORY_LIMIT; default: 3)")
    ap.add_argument("--jobs", "-j", type=int, default=0,
                    help="cases run at once (default: CPU count, capped so jobs x memory limit fits in RAM)")
    ap.add_argument("--allow-stale", action="store_true",
                    help="run even if the jaic binary is older than the compiler sources")
    a = ap.parse_args()
    limit_bytes = int(a.memory_limit * 2**30)
    if a.jobs <= 0:
        a.jobs = max(1, min(os.cpu_count() or 1, int(physical_memory_gib() // a.memory_limit)))
    if not a.allow_stale:
        stale = stale_sources(Path(a.jaic))
        if stale:
            sys.exit(f"jaic-sweep: {a.jaic} is older than {stale}; rebuild it (or pass --allow-stale)")
    use_native_libs()
    # Upstream cases may name setup commands (building a C library the program loads); they run
    # once, serially, in the case's directory before any case starts.
    if "upstream" in a.sets:
        for c in json.loads((ROOT / "tools/upstream-cases.json").read_text()):
            if a.filter in c["id"]:
                if "copy" in c:
                    WORKDIRS[c["id"]] = make_workdir(c)
                for command in c.get("setup", []):
                    subprocess.run(command, cwd=(upstream_root(c) / c["path"]).parent,
                                   capture_output=True, stdin=subprocess.DEVNULL)
    todo = [c for s in a.sets for c in cases(s) if a.filter in c[0]]

    def run(case):
        cid, path, mode, expect, extra = case
        out, err, code = run_limited([a.jaic, mode, str(path), *extra], path.parent, a.timeout, limit_bytes)
        if expect is None:
            ok = code == 0
        elif "negative" in expect:
            ok = code != 0 and code != -1 and expect["negative"] in err
        elif "contains" in expect:
            missing = [line for line in expect["contains"] if line not in out]
            present = [text for text in expect["excludes"] if text in out]
            ok = code == 0 and not missing and not present
            if code == 0 and not ok:
                err += "".join(f"\nerror: stdout lacks {line!r}" for line in missing)
                err += "".join(f"\nerror: stdout has {text!r}" for text in present)
        else:
            ok = code == expect.get("exit_code", 0) and out == expect.get("stdout", "")
        return cid, ok, out, err, code

    passed, failed = 0, []
    with ThreadPoolExecutor(max_workers=max(1, a.jobs)) as pool:
        for cid, ok, out, err, code in pool.map(run, todo):
            if ok:
                passed += 1
                continue
            first = next((l for l in err.splitlines() if "error" in l), err.strip().splitlines()[0] if err.strip() else f"exit {code}, stdout {out[:80]!r}")
            failed.append((cid, first))
            if a.verbose:
                print(f"--- {cid}\n{err[:1500]}{out[:500]}")
    for root in WORKDIRS.values():
        shutil.rmtree(root, ignore_errors=True)
    for cid, msg in failed:
        print(f"FAIL {cid}: {msg[:220]}")
    print(f"\n{passed} passed, {len(failed)} failed")

if __name__ == "__main__":
    main()
