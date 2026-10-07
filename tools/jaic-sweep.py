#!/usr/bin/env python3
"""Run jaic over test programs and summarize results.

Usage: tools/jaic-sweep.py [--jaic PATH] [--filter TEXT] [--verbose] SET...
Sets: corpus (tests/corpus/positive with expected runtime), negative (tests/corpus/negative:
`jaic check` must fail and report the recorded text), stdlib (tests/stdlib,
run must succeed), modules (the stdlib's own tests: stdlib/tests and stdlib/<Module>/tests,
run must succeed; a test directory's `modules/` folder holds its mock modules), examples (tests/examples.json:
example programs such as examples/tour, whose stdout must contain the listed lines), upstream
(tools/upstream-cases.json: upstream project entry points that must pass, with the output the
project documents where a case has an `expect` record), or file paths.

--native builds each `run` case with `jaic build` and runs the executable instead, with the
same expectations; --sanitize address,undefined (implies --native) instruments those builds and
fails a case on any sanitizer report (docs/native/sanitizers.md).
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
    # Files of this repository placed into the copy (a driver for a project whose own build
    # script cannot run here); destination relative to the scratch root -> repository path.
    for dest, source in c.get("files", {}).items():
        shutil.copyfile(ROOT / source, root / dest)
    return root

def upstream_expect(c):
    """An upstream case's `expect` record (output the project itself documents), in the shape
    run() compares: exit code, exact stdout, or lines that must appear (in order or anywhere)."""
    e = c.get("expect")
    if not e and not c.get("run_after"):
        return None
    e = e or {}
    return {"upstream": True, "exit_code": e.get("exit_code", 0), "stdout": e.get("stdout"),
            "ordered": e.get("stdout_ordered", []), "contains": e.get("stdout_contains", []),
            "excludes": [], "run_after": c.get("run_after")}

def missing_in_order(lines, out):
    """The first of `lines` not found after the previous one's match in `out`, or None."""
    at = 0
    for line in lines:
        found = out.find(line, at)
        if found < 0:
            return line
        at = found + len(line)
    return None

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
            yield c["id"], upstream_root(c) / c["path"], c["mode"], upstream_expect(c), c.get("args", [])
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
# `jaic build`'s exit status for a program without `main` (NO_MAIN_STATUS in jaic-cli).
NO_MAIN_EXIT = 3

def run_limited(command, cwd, timeout, limit_bytes, jaic=True):
    """Runs `command` with jaic's exact allocation limit armed, killing it on timeout. Returns
    (stdout, stderr, code), with code -1 and a one-line verdict as stderr when a limit stopped it.
    `jaic` says whether the command is jaic, whose exit status MEMORY_LIMIT_EXIT means the limit
    (another program may exit with any status)."""
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
    if jaic and code == MEMORY_LIMIT_EXIT:
        return "", next((l for l in err.splitlines() if l.strip()), "memory limit exceeded"), -1
    return out, err, code

# Runtime options for sanitized executables. Leaks are not reported: programs routinely leave
# their memory to the OS at exit, and a leak is not the codegen or memory-safety bug the
# sanitizer run looks for. Every other report stops the program with a failing status.
SANITIZER_ENV = {
    "ASAN_OPTIONS": "detect_leaks=0:halt_on_error=1:abort_on_error=0:detect_stack_use_after_return=1:strict_string_checks=1:check_initialization_order=1",
    "UBSAN_OPTIONS": "print_stacktrace=1:halt_on_error=1",
}

def sanitizer_report(err):
    """The first line of a sanitizer report in `err`, or None."""
    for line in err.splitlines():
        if ("Sanitizer:" in line and "ERROR" in line) or ": runtime error:" in line:
            return line.strip()
    return None

def symbolizer():
    """llvm-symbolizer from the LLVM jaic links against, so reports show source lines."""
    prefix = os.environ.get("LLVM_SYS_231_PREFIX")
    candidates = [Path(prefix) / "bin/llvm-symbolizer"] if prefix else []
    candidates += [Path(p) for p in (shutil.which("llvm-symbolizer-23"), shutil.which("llvm-symbolizer")) if p]
    return next((str(p) for p in candidates if p.is_file()), None)

# `run_native`'s status for a program whose build writes no executable.
NO_EXECUTABLE = "no executable"

def opens_windows(path):
    """Whether a case opens windows: it imports Window_Creation. Machines without a display or
    GPU (CI runners) can build such programs but not run them."""
    try:
        return '#import "Window_Creation"' in path.read_text(errors="replace")
    except OSError:
        return False

def run_native(jaic, path, extra, build_flags, scratch, timeout, limit_bytes, run=True):
    """Build `path` natively with `build_flags` and run the executable in the source's directory.
    Returns (stdout, stderr, code) like `run_limited`; a failed build or a sanitizer report puts
    an `error:` line first in stderr."""
    # A case's extra arguments are `jaic run` options; the program's own come after `--`.
    split = extra.index("--") if "--" in extra else len(extra)
    jaic_args, program_args = extra[:split], extra[split + 1:]
    if any(a in jaic_args for a in ("-os", "-cpu", "-target", "--target")):
        return "", "", NO_EXECUTABLE  # built for another platform
    exe = Path(tempfile.mkdtemp(dir=scratch)) / path.stem
    try:
        _, err, code = run_limited([jaic, "build", str(path), "-o", str(exe), *build_flags, *jaic_args],
                                   path.parent, timeout, limit_bytes)
        # A program without `main` (its checks are `#run` directives), or whose metaprogram asks
        # for no output, did all its work at compile time.
        if code == NO_MAIN_EXIT:
            return "", "", NO_EXECUTABLE
        if code != 0:
            cause = next((l.strip() for l in err.splitlines() if "error" in l.lower()), "")
            return "", f"error: native build failed (exit {code}): {cause}\n{err}", code
        if not exe.exists():
            return "", "", NO_EXECUTABLE
        if not run:
            return "", "", 0
        out, err, code = run_limited([str(exe), *program_args], path.parent, timeout, limit_bytes, jaic=False)
        report = sanitizer_report(err)
        if report:
            return out, f"error: sanitizer: {report}\n{err}", code if code != 0 else 1
        return out, err, code
    finally:
        shutil.rmtree(exe.parent, ignore_errors=True)

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
    ap.add_argument("--headless", action="store_true",
                    help="build (or check) programs that open windows instead of running them")
    ap.add_argument("--native", action="store_true",
                    help="build `run` cases with `jaic build` and run the executables")
    ap.add_argument("--sanitize", default="",
                    help="sanitizers for native builds (address, undefined or both, comma-separated); implies --native")
    ap.add_argument("--opt", default="", choices=["", "O0", "O1", "O2", "O3"],
                    help="optimization level of native builds (default: what the program asks for)")
    a = ap.parse_args()
    # Cases run in their own directory, so a relative path would not resolve there.
    a.jaic = str(Path(a.jaic).resolve())
    if a.sanitize or a.opt:
        a.native = True
    build_flags = (["-sanitize", a.sanitize] if a.sanitize else []) + ([f"-{a.opt}"] if a.opt else [])
    if a.sanitize:
        os.environ.update({k: v for k, v in SANITIZER_ENV.items() if k not in os.environ})
        if "ASAN_SYMBOLIZER_PATH" not in os.environ and symbolizer():
            os.environ["ASAN_SYMBOLIZER_PATH"] = symbolizer()
    scratch = tempfile.mkdtemp(prefix="jaic-sweep-native-") if a.native else None
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
        windowed = a.headless and mode == "run" and opens_windows(path)
        if windowed and expect is not None:
            expect = None  # Only whether it builds is checked.
        if a.native and mode == "run":
            out, err, code = run_native(a.jaic, path, extra, build_flags, scratch, a.timeout, limit_bytes,
                                        run=not windowed)
            if code == NO_EXECUTABLE:
                return cid, None, out, err, code
        else:
            # A sweep only checks what metaprograms' workspaces produce; it writes nothing.
            quiet = ["-no_workspace_output"] if mode == "run" and not windowed else []
            out, err, code = run_limited([a.jaic, "check" if windowed else mode, str(path), *quiet, *extra],
                                         path.parent, a.timeout, limit_bytes)
        # `run_after`: the program the build produced is run, and the expectations apply to it.
        if code == 0 and expect and expect.get("run_after"):
            program = expect["run_after"]
            out, err, code = run_limited([str(path.parent / program[0]), *program[1:]], path.parent,
                                         a.timeout, limit_bytes, jaic=False)
        if expect is None:
            ok = code == 0
        elif "upstream" in expect:
            problems = [f"exit code {code}, expected {expect['exit_code']}"] if code != expect["exit_code"] else []
            if expect["stdout"] is not None and out != expect["stdout"]:
                problems.append("stdout differs from the recorded output")
            problems += [f"stdout lacks {line!r}" for line in expect["contains"] if line not in out]
            gap = missing_in_order(expect["ordered"], out)
            if gap is not None:
                problems.append(f"stdout lacks {gap!r} (in order)")
            ok = not problems
            err = "".join(f"error: {p}\n" for p in problems) + err
        elif "negative" in expect:
            ok = code != 0 and code != -1 and expect["negative"] in err
        elif "contains" in expect:
            missing = [line for line in expect["contains"] if line not in out]
            present = [text for text in expect["excludes"] if text in out]
            ok = code == 0 and not missing and not present
            if code == 0 and not ok:
                err += "".join(f"\nerror: stdout lacks {line!r}" for line in missing)
                err += "".join(f"\nerror: stdout has {text!r}" for text in present)
        elif "error" in expect:
            # A runtime error: any failing status (a native build traps), with the message.
            ok = code not in (0, -1) and expect["error"] in err and out == expect.get("stdout", "")
        else:
            ok = code == expect.get("exit_code", 0) and out == expect.get("stdout", "")
        return cid, ok, out, err, code

    passed, failed, no_exe = 0, [], []
    with ThreadPoolExecutor(max_workers=max(1, a.jobs)) as pool:
        for cid, ok, out, err, code in pool.map(run, todo):
            if ok is None:
                no_exe.append(cid)
                continue
            if ok:
                passed += 1
                continue
            first = next((l for l in err.splitlines() if "error" in l), err.strip().splitlines()[0] if err.strip() else f"exit {code}, stdout {out[:80]!r}")
            failed.append((cid, first))
            if a.verbose:
                print(f"--- {cid}\n{err[:1500]}{out[:500]}")
    for root in WORKDIRS.values():
        shutil.rmtree(root, ignore_errors=True)
    if scratch:
        shutil.rmtree(scratch, ignore_errors=True)
    for cid, msg in failed:
        print(f"FAIL {cid}: {msg[:220]}")
    if no_exe:
        print(f"\nnot run natively (compile-time only, interpreter-only or another platform): {', '.join(no_exe)}")
    print(f"\n{passed} passed, {len(failed)} failed" + (f", {len(no_exe)} not run natively" if no_exe else ""))
    # A failing status lets CI use the sweep directly.
    sys.exit(1 if failed else 0)

if __name__ == "__main__":
    main()
