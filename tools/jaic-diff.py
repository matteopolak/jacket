#!/usr/bin/env python3
"""Differential testing: run each program through every jaic backend and require the same result.

Usage: tools/jaic-diff.py [--backends interp,native,native-O2,wasm,wasm-native] [--wasm BUNDLE] SET...
Sets: corpus (tests/corpus/positive cases with a runtime expectation), stdlib (tests/stdlib),
modules (stdlib/*/tests and stdlib/tests), gen:SEED:COUNT (COUNT programs from tools/jaigen.py starting at
SEED; reproduce one with `tools/jaigen.py SEED`), or file paths.

Backends:
  interp     `jaic run`: the interpreter on the host OS
  native     `jaic build` (LLVM, -O0) and run the executable
  native-O2  the same with -O2
  wasm       the browser engine (crates/jai-wasm in node, tools/jaic_diff_wasm.mjs): the interpreter compiled
             to wasm32, target OS .WASM, sandboxed host
  wasm-native  `jaic build -os wasm` (LLVM, wasm64 with Wasi_Runtime) run by node's WASI
             (tools/wasi_run.mjs); only when asked for, it needs wasm-ld and node 24

A result is (status, stdout, stderr). Status is `exit N` or `runtime error` (an interpreter runtime error,
a native trap or signal: the backends report these differently, so only the fact is compared). stdout must
match exactly; stderr must match when every backend exited normally. A backend whose compile fails while
another's succeeds is a disagreement, except that a backend may decline a program it cannot run
(see `unsupported` below); those are reported as skips, not passes.
"""
import argparse, importlib.util, json, os, re, select, shutil, subprocess, sys, tempfile, threading, time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location("jaic_sweep", ROOT / "tools/jaic-sweep.py")
sweep = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sweep)

ALL_BACKENDS = ["interp", "native", "native-O2", "wasm", "wasm-native"]

# The wasm engine says so when a program needs a host service the sandbox lacks: these mean
# "cannot run here", never "ran differently". Source: SandboxHost in crates/jaic/src/interp.
WASM_UNSUPPORTED = re.compile(r"foreign procedure '[^']*' is not available here|unknown library|"
                              r"is not supported on wasm|no browser backend")


# Programs whose output legitimately differs between targets. Only their status (exit code or
# runtime error) is compared. Keep this list short: each entry needs a reason a reviewer can check.
OUTPUT_VARIES = {
    # Long_Double is float64 on arm64 macOS and binary128 on wasm32 (each target's C ABI); the
    # program prints which representation it tested.
    "jaic-extensions-long-double": "prints whether Long_Double is wide, which depends on the target ABI",
    # A profiler report: the numbers are measured times (a virtual clock on wasm).
    "iprof-runtime-manual": "prints measured times",
    # trace_assert prints the native call stack: return addresses and the running executable's
    # symbols (the interpreter's own frames under `jaic run`).
    "debug-assert-handlers": "prints a native stack trace",
}

# A native build of a program whose `main` calls a `#compiler` primitive stops with this message:
# those primitives (the Bindings_Generator's libclang bridge, for one) exist only inside the compiler.
COMPILE_TIME_ONLY = "is a compiler primitive; it runs only at compile time"


# A program that asks whether it runs in the browser (`OS == .WASM`) may skip work there, so its
# output on wasm can differ by design; tests/stdlib programs skip processes, native libraries and
# windows this way. For such a program only the wasm status is compared.
TARGET_AWARE = re.compile(r"\.WASM\b")


class Case:
    def __init__(self, cid, path, root=None, args=(), skip=None):
        self.id, self.path = cid, Path(path)
        # Workspace root for the wasm backend: every text file below it is visible to the program.
        self.root = Path(root) if root else self.path.parent
        self.args = list(args)
        self.skip = dict(skip or {})  # backend -> reason
        try:
            self.target_aware = bool(TARGET_AWARE.search(self.path.read_text(errors="replace")))
        except OSError:
            self.target_aware = False


def cases(name, gen_dir, gen_size=1.0):
    if name == "corpus":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if "runtime" in c:
                yield Case(c["id"], ROOT / "tests/corpus" / c["source"])
    elif name == "stdlib":
        for p in sorted((ROOT / "tests/stdlib").glob("*.jai")):
            yield Case(p.stem, p, ROOT / "tests/stdlib")
    elif name == "modules":
        found = list((ROOT / "stdlib").glob("*/tests/*.jai")) + list((ROOT / "stdlib/tests").glob("**/*.jai"))
        for p in sorted(set(found)):
            rel = p.relative_to(ROOT / "stdlib")
            if "modules" not in rel.parts:
                # The whole stdlib is the workspace: tests `#load` their module's files by relative path.
                yield Case(str(rel.with_suffix("")), p, ROOT / "stdlib")
    elif name.startswith("gen:"):
        _, seed, count = (name.split(":") + ["1"])[:3]
        sys.path.insert(0, str(ROOT / "tools"))
        import jaigen
        for s in range(int(seed), int(seed) + int(count)):
            d = gen_dir / f"gen-{s}"
            d.mkdir(parents=True, exist_ok=True)
            (d / "main.jai").write_text(jaigen.generate(s, gen_size))
            yield Case(f"gen-{s}", d / "main.jai")
    else:
        p = Path(name).resolve()
        yield Case(p.stem, p)


class Result:
    def __init__(self, status, stdout="", stderr="", note=""):
        self.status, self.stdout, self.stderr, self.note = status, stdout, stderr, note

    def key(self, with_stderr, with_output=True):
        if not with_output:
            return self.status
        return (self.status, self.stdout, self.stderr if with_stderr else None)


def runtime_error_text(err):
    return "runtime error" in err or "panic" in err.lower() or "wasm trap" in err


class WasmPool:
    """One node process per worker thread, restarted after a timeout or a crash of the driver."""

    def __init__(self, bundle, timeout):
        self.bundle, self.timeout = bundle, timeout
        self.local = threading.local()

    def proc(self):
        p = getattr(self.local, "proc", None)
        if p is None or p.poll() is not None:
            p = subprocess.Popen(["node", "--stack-size=4000", str(ROOT / "tools/jaic_diff_wasm.mjs"), str(self.bundle)],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
            self.local.proc = p
        return p

    def kill(self):
        p = getattr(self.local, "proc", None)
        if p:
            p.kill()
            p.wait()
            self.local.proc = None

    def run(self, case):
        p = self.proc()
        request = {"root": str(case.root), "main": str(case.path.relative_to(case.root))}
        p.stdin.write(json.dumps(request) + "\n")
        p.stdin.flush()
        ready, _, _ = select.select([p.stdout], [], [], self.timeout)
        if not ready:
            self.kill()
            return Result("timeout")
        line = p.stdout.readline()
        if not line:
            self.kill()
            return Result("crash", note="wasm driver exited")
        r = json.loads(line)
        if r.get("crash"):
            return Result("crash", note=r["crash"][:300])
        errors = [d for d in r["diagnostics"] if d["severity"] == "error"]
        if r["exitCode"] is None or errors:
            text = "\n".join(f"{d['file']}:{d['line']}: {d['message']}" for d in errors) or r["stderr"]
            if WASM_UNSUPPORTED.search(text):
                return Result("unsupported", note=text.splitlines()[0][:200])
            # A stdlib module that rejects the target with `#assert` (stb_image needs a native C library).
            if any(d["file"].startswith("/stdlib/") and d["message"].startswith("#assert failed") for d in errors):
                return Result("unsupported", note=f"{errors[0]['file']}: the module does not support the WASM target")
            # The sandbox file system holds the workspace and the stdlib, nothing else from the repository.
            if missing := re.search(r"Unable to open '([^']*)'|file `([^`]*)` does not exist", r["stderr"] + text):
                return Result("missing-file", r["stdout"], r["stderr"], note=f"no {missing.group(1) or missing.group(2)} in the sandbox")
            if r["exitCode"] is None:
                return Result("compile error", note=text[:300])
            return Result("runtime error", r["stdout"], r["stderr"], note=text[:300])
        return Result(f"exit {r['exitCode']}", r["stdout"], r["stderr"])


class Runner:
    """Runs a case through the chosen backends; shared by main() and tools/jaic-reduce.py."""

    def __init__(self, jaic, backends, work, wasm_bundle=None, timeout=120, memory_gib=3):
        self.jaic, self.backends, self.work = Path(jaic).resolve(), backends, Path(work)
        self.timeout, self.limit_bytes = timeout, int(memory_gib * 2**30)
        self.wasm = WasmPool(Path(wasm_bundle).resolve(), timeout) if "wasm" in backends else None

    def capped(self, cmd, cwd):
        return sweep.run_limited([str(x) for x in cmd], cwd, self.timeout, self.limit_bytes)

    @staticmethod
    def classify(out, err, code):
        if code == -1:
            return Result("timeout" if err == "timeout" else "memory", note=err)
        if code < 0 or code >= 128 and code - 128 in (4, 5, 6, 7, 8, 10, 11):
            return Result("runtime error", out, err, note=f"signal {code}")
        if code != 0 and runtime_error_text(err):
            return Result("runtime error", out, err, note=err.strip().splitlines()[-1][:200] if err.strip() else "")
        return Result(f"exit {code}", out, err)

    def run_interp(self, case):
        out, err, code = self.capped([self.jaic, "run", case.path, *(["--", *case.args] if case.args else [])],
                                     case.path.parent)
        if code == 1 and "runtime error" not in err and re.search(r"(^|\n)\S*:\d+:\d+: error:|^error:", err):
            return Result("compile error", note=err.strip()[:300])
        return self.classify(out, err, code)

    def run_native(self, case, opt, wasm=False):
        exe = self.work / "bin" / f"{re.sub(r'[^A-Za-z0-9_.-]', '_', case.id)}-{opt}{'.wasm' if wasm else ''}"
        exe.parent.mkdir(parents=True, exist_ok=True)
        target = ["-os", "wasm"] if wasm else []
        bout, berr, bcode = self.capped([self.jaic, "build", case.path, "-o", exe, f"-{opt}", *target], case.path.parent)
        if bcode != 0:
            if bcode == -1:
                return Result("timeout" if berr == "timeout" else "memory", note=berr)
            if "runtime error" in berr:
                return Result("runtime error", bout, berr, note="during compile-time execution")
            if "has no `main` procedure" in berr:
                return Result("unsupported", note="no main: the program only runs at compile time")
            return Result("compile error", note=berr.strip()[:300])
        if not exe.exists():
            # A metaprogram that takes over the build (its own workspaces, NO_OUTPUT) decides what gets
            # written; the program it compiled is not this executable.
            return Result("unsupported", note="the build wrote no executable at -o (the program's metaprogram controls output)")
        run = ["node", "--no-warnings", ROOT / "tools/wasi_run.mjs", exe] if wasm else [exe]
        out, err, code = self.capped([*run, *case.args], case.path.parent)
        exe.unlink(missing_ok=True)
        if wasm and (missing := re.search(r"wasm link error: .*", err)):
            # The program calls a C function Wasi_Runtime does not provide.
            return Result("unsupported", note=missing.group(0)[:200])
        r = self.classify(bout + out, berr + err, code)
        if r.status == "runtime error" and COMPILE_TIME_ONLY in err:
            return Result("unsupported", note=err.strip().splitlines()[-1][:200])
        return r

    @staticmethod
    def normalize(case, r):
        """The wasm engine sees the stdlib at /stdlib and the case directory at /workspace; write host
        paths the same way so paths in messages compare equal."""
        if r.status in ("unsupported", "compile error"):
            return r
        for real, virtual in ((case.root, "/workspace"), (ROOT / "stdlib", "/stdlib")):
            r.stdout = r.stdout.replace(str(real), virtual)
            r.stderr = r.stderr.replace(str(real), virtual)
        return r

    def run(self, case):
        results = {}
        for b in self.backends:
            if b in case.skip:
                results[b] = Result("unsupported", note=case.skip[b])
            elif b == "interp":
                results[b] = self.run_interp(case)
            elif b == "native":
                results[b] = self.run_native(case, "O0")
            elif b == "native-O2":
                results[b] = self.run_native(case, "O2")
            elif b == "wasm-native":
                results[b] = self.run_native(case, "O0", wasm=True)
            elif b == "wasm":
                results[b] = Result("unsupported", note="needs program arguments") if case.args else self.wasm.run(case)
            results[b] = self.normalize(case, results[b])
        # A file the sandbox lacks is only an environment difference when a host backend found it.
        if results.get("wasm") and results["wasm"].status == "missing-file":
            host = [r for b, r in results.items() if b != "wasm" and r.status != "unsupported"]
            missing = results["wasm"].note[3:-len(" in the sandbox")]
            if host and all(missing not in r.stderr for r in host):
                results["wasm"] = Result("unsupported", note=results["wasm"].note)
            else:
                results["wasm"].status = "runtime error"
        return case, results


def verdict(case, results):
    """agree, DISAGREE, invalid (every backend fails to compile) or skip (fewer than two ran)."""
    live = {b: r for b, r in results.items() if r.status != "unsupported"}
    if len(live) < 2:
        return "skip"
    if {r.status for r in live.values()} == {"compile error"}:
        return "invalid"
    with_stderr = all(r.status.startswith("exit") for r in live.values())
    with_output = case.id not in OUTPUT_VARIES
    keys = {r.key(with_stderr, with_output and not (b.startswith("wasm") and case.target_aware))
            for b, r in live.items()}
    if len({k if isinstance(k, str) else k[0] for k in keys}) > 1:
        return "DISAGREE"
    # Full results must agree among the backends compared by output; status-only ones by status.
    return "agree" if len({k for k in keys if not isinstance(k, str)}) <= 1 else "DISAGREE"


def add_backend_arguments(ap):
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument("--wasm", help="browser bundle directory (tools/build_scripting_wasm.py --output)")
    ap.add_argument("--backends", default=None,
                    help=f"comma-separated subset of {','.join(ALL_BACKENDS)} (default: all available)")
    ap.add_argument("--timeout", type=float, default=120)
    ap.add_argument("--memory-limit", type=float, default=3, help="GiB per process (default 3)")
    ap.add_argument("--allow-stale", action="store_true",
                    help="run even if the jaic binary is older than the compiler sources")


def chosen_backends(a):
    backends = a.backends.split(",") if a.backends else [b for b in ALL_BACKENDS
                                                        if (b != "wasm" or a.wasm) and b != "wasm-native"]
    for b in backends:
        if b not in ALL_BACKENDS:
            sys.exit(f"unknown backend {b}")
    if "wasm" in backends and not a.wasm:
        sys.exit("the wasm backend needs --wasm <bundle dir>")
    if not a.allow_stale and (stale := sweep.stale_sources(Path(a.jaic))):
        sys.exit(f"{a.jaic} is older than {stale}; rebuild it (or pass --allow-stale)")
    return backends


def describe(v, case, results):
    print(f"{v} {case.id}")
    for b, r in results.items():
        print(f"  {b:10} {r.status:14} {r.note[:160]!r}" if r.note else f"  {b:10} {r.status}")
    if v == "DISAGREE":
        print_diff(results)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("sets", nargs="+")
    add_backend_arguments(ap)
    ap.add_argument("--filter", default="")
    ap.add_argument("--jobs", "-j", type=int, default=3)
    ap.add_argument("--keep", help="directory for generated programs and build outputs (default: a temp dir)")
    ap.add_argument("--verbose", "-v", action="store_true")
    ap.add_argument("--json", help="write per-case results here")
    ap.add_argument("--gen-size", type=float, default=1.0, help="tools/jaigen.py --size for gen: sets")
    a = ap.parse_args()

    backends = chosen_backends(a)
    # Some stdlib tests call or link third-party C libraries (rpmalloc, stb_*); the corpus and
    # generated programs need none, so CI does not build them for those.
    if any(not (s == "corpus" or s.startswith("gen:")) for s in a.sets):
        sweep.use_native_libs()
    work = Path(a.keep) if a.keep else Path(tempfile.mkdtemp(prefix="jaic-diff-"))
    work.mkdir(parents=True, exist_ok=True)
    runner = Runner(a.jaic, backends, work, a.wasm, a.timeout, a.memory_limit)
    started = time.monotonic()
    todo = [c for s in a.sets for c in cases(s, work / "programs", a.gen_size) if a.filter in c.id]
    counts = {"agree": 0, "DISAGREE": 0, "skip": 0, "invalid": 0}
    partial = {b: 0 for b in backends}
    report = []
    with ThreadPoolExecutor(max_workers=max(1, a.jobs)) as pool:
        for case, results in pool.map(runner.run, todo):
            v = verdict(case, results)
            counts[v] += 1
            for b, r in results.items():
                if r.status == "unsupported":
                    partial[b] += 1
            report.append({"id": case.id, "path": str(case.path), "verdict": v,
                           "results": {b: vars(r) for b, r in results.items()}})
            if v in ("DISAGREE", "invalid") or a.verbose:
                describe(v, case, results)
            sys.stdout.flush()
    if a.json:
        Path(a.json).write_text(json.dumps(report, indent=1))
    if not a.keep:
        shutil.rmtree(work, ignore_errors=True)
    skipped = ", ".join(f"{b} {n}" for b, n in partial.items() if n)
    print(f"\n{counts['agree']} agree, {counts['DISAGREE']} disagree, {counts['invalid']} fail to compile everywhere, "
          f"{counts['skip']} skipped (fewer than two backends)" + (f"; unsupported per backend: {skipped}" if skipped else "")
          + f" [{', '.join(backends)}; {time.monotonic() - started:.0f} s]")
    sys.exit(1 if counts["DISAGREE"] or counts["invalid"] else 0)


def print_diff(results):
    """Show the first differing line of stdout/stderr between the first backend and each other one."""
    items = [(b, r) for b, r in results.items() if r.status != "unsupported"]
    base_b, base = items[0]
    for b, r in items[1:]:
        for stream in ("stdout", "stderr"):
            x, y = getattr(base, stream).splitlines(), getattr(r, stream).splitlines()
            if x != y:
                i = next((i for i in range(min(len(x), len(y))) if x[i] != y[i]), min(len(x), len(y)))
                print(f"    {stream} line {i + 1}: {base_b}={x[i] if i < len(x) else '<end>'!r:.120} "
                      f"{b}={y[i] if i < len(y) else '<end>'!r:.120}")


if __name__ == "__main__":
    main()
