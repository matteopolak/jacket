#!/usr/bin/env python3
"""Build jaic, jailsp and jailint with profile-guided optimisation, and optionally BOLT.

Steps (docs/tools/pgo-and-bolt.md):
  1. build instrumented binaries (`-Cprofile-generate`) into <target-dir>/pgo/instrumented;
  2. run the training workload in this repository with them (sweep sets, native builds,
     benchmarks, jailint over the stdlib, a scripted jailsp session);
  3. merge the raw profiles with the toolchain's own llvm-profdata (rustup's `llvm-tools`);
  4. rebuild with `-Cprofile-use` into <target-dir>/pgo/optimized;
  5. with --bolt (Linux ELF only): instrument those binaries with llvm-bolt, run the training
     workload again, and rewrite them with the collected profile.
The results are copied to --out (default <target-dir>/pgo/dist).

Usage: tools/build_pgo.py [--llvm dynamic|static|none] [--bolt] [--jobs N] [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TOOLS = ("jaic", "jailsp", "jailint")
EXE = ".exe" if os.name == "nt" else ""
# `jaic-cli` features per --llvm choice; the other two packages always use their defaults.
LLVM_FEATURES = {
    "dynamic": [],
    "static": ["--no-default-features", "--features", "static-llvm"],
    "none": ["--no-default-features"],
}
# Files the scripted jailsp session opens: a program with imports, a generic module and tests.
LSP_FILES = [
    "examples/tour/main.jai",
    "examples/tour/generics/polymorphism.jai",
    "examples/compile-time-record.jai",
    "tests/stdlib/hash-table-collisions.jai",
    "stdlib/String/module.jai",
]


def log(message):
    print(f"build_pgo: {message}", flush=True)


def host_triple():
    out = subprocess.check_output(["rustc", "-vV"], text=True, cwd=ROOT)
    return next(l.split(":", 1)[1].strip() for l in out.splitlines() if l.startswith("host:"))


def rustflags_var(triple, environ):
    """The variable to extend with our flags. RUSTFLAGS, when set, overrides every
    target-specific setting, so extend it then; otherwise extend the target's own variable
    (CI puts `+crt-static` and the macOS linker flags there)."""
    if environ.get("RUSTFLAGS"):
        return "RUSTFLAGS"
    return "CARGO_TARGET_" + triple.upper().replace("-", "_").replace(".", "_") + "_RUSTFLAGS"


def profdata_tool():
    """llvm-profdata from the pinned toolchain: raw profiles must be read by the LLVM that
    rustc itself was built with."""
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True, cwd=ROOT).strip())
    tool = sysroot / "lib" / "rustlib" / host_triple() / "bin" / f"llvm-profdata{EXE}"
    if not tool.exists():
        sys.exit(f"build_pgo: {tool} is missing; run `rustup component add llvm-tools` "
                 "for the toolchain in rust-toolchain.toml")
    return tool


def bolt_tool(name, llvm_bin):
    """llvm-bolt / merge-fdata: --llvm-bin, then an LLVM_SYS_*_PREFIX install (the official
    Linux release tarball ships BOLT), then PATH (including apt's versioned names)."""
    dirs = [Path(llvm_bin)] if llvm_bin else []
    dirs += [Path(v) / "bin" for k, v in sorted(os.environ.items())
             if k.startswith("LLVM_SYS_") and k.endswith("_PREFIX")]
    for d in dirs:
        if (d / name).exists():
            return str(d / name)
    found = shutil.which(name)
    if found:
        return found
    for major in range(40, 17, -1):
        found = shutil.which(f"{name}-{major}")
        if found:
            return found
    sys.exit(f"build_pgo: {name} not found; pass --llvm-bin (the LLVM release's bin directory)")


def run(cmd, env=None, cwd=ROOT, check=True, **kw):
    log("$ " + " ".join(str(c) for c in cmd))
    result = subprocess.run([str(c) for c in cmd], env=env, cwd=cwd, **kw)
    if check and result.returncode != 0:
        sys.exit(f"build_pgo: command failed with exit code {result.returncode}")
    return result


def build_env(target, environ, target_dir, flags):
    """The environment for a cargo build of `target` with extra rustc `flags`."""
    env = dict(environ, CARGO_TARGET_DIR=str(target_dir))
    var = rustflags_var(target, env)
    env[var] = " ".join(f for f in [env.get(var, "")] + flags if f)
    if "msvc" not in target:
        # The cc crate copies -Cprofile-generate/-use into the flags of clang-compiled C code
        # (llvm-sys's target wrappers). That clang is not rustc's LLVM, so its profile records
        # have another layout and the instrumented jaic crashes writing them at exit; and it
        # cannot read our .profdata. Environment CFLAGS come last, so these switch it off again.
        cvar = "CFLAGS_" + target.replace("-", "_")
        base = env.get(cvar, env.get("CFLAGS", ""))
        env[cvar] = " ".join(f for f in [base, "-fno-profile-generate", "-fno-profile-use"] if f)
    return env


def cargo_build(args, target_dir, flags):
    env = build_env(args.target, os.environ, target_dir, flags)
    common = ["cargo", "build", "--release", "--locked", "--target", args.target, "-j", str(args.jobs)]
    run(common + ["-p", "jaic-cli"] + LLVM_FEATURES[args.llvm], env=env)
    run(common + ["-p", "jai-language-server", "-p", "jailint"], env=env)
    out = Path(target_dir) / args.target / "release"
    return {tool: out / f"{tool}{EXE}" for tool in TOOLS}


# --- training workload -------------------------------------------------------------------------

def lsp_session(jailsp, env):
    """Drive jailsp the way an editor does: open files, ask for symbols, tokens, hovers and
    completions across each file, type into it, then shut down."""
    messages, next_id = [], [0]

    def request(method, params):
        next_id[0] += 1
        messages.append({"jsonrpc": "2.0", "id": next_id[0], "method": method, "params": params})

    def notify(method, params):
        messages.append({"jsonrpc": "2.0", "method": method, "params": params})

    request("initialize", {"capabilities": {}})
    notify("initialized", {})
    for rel in LSP_FILES:
        path = ROOT / rel
        if not path.exists():
            continue
        uri, text = path.resolve().as_uri(), path.read_text(errors="replace")
        lines = text.split("\n")
        doc = {"uri": uri}
        notify("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "jai", "version": 1, "text": text}})
        for method in ("documentSymbol", "semanticTokens/full", "foldingRange", "documentLink", "codeLens"):
            request(f"textDocument/{method}", {"textDocument": doc})
        request("textDocument/inlayHint", {"textDocument": doc, "range": {
            "start": {"line": 0, "character": 0}, "end": {"line": len(lines) - 1, "character": 0}}})
        # Positions on identifiers spread over the file.
        spots = []
        for n, line in enumerate(lines):
            col = next((i for i, ch in enumerate(line) if ch.isalpha() or ch == "_"), None)
            if col is not None and not line.lstrip().startswith("//"):
                spots.append((n, col + 1))
        for line, col in spots[:: max(1, len(spots) // 40)]:
            at = {"textDocument": doc, "position": {"line": line, "character": col}}
            for method in ("hover", "completion", "definition", "documentHighlight", "signatureHelp"):
                request(f"textDocument/{method}", at)
        for line, col in spots[:: max(1, len(spots) // 4)]:
            at = {"textDocument": doc, "position": {"line": line, "character": col}}
            request("textDocument/references", dict(at, context={"includeDeclaration": True}))
            request("textDocument/typeDefinition", at)
            request("textDocument/prepareRename", at)
        # Type a new procedure at the end, character by character, completing as it goes.
        typed, version = "\nprobe_for_training :: () {\n    total := 0;\n    for 1..10 total += it;\n    pri", 1
        end = len(lines) - 1
        cursor = [end, len(lines[end])]
        for ch in typed:
            version += 1
            notify("textDocument/didChange", {"textDocument": {"uri": uri, "version": version}, "contentChanges": [{
                "range": {"start": {"line": cursor[0], "character": cursor[1]},
                          "end": {"line": cursor[0], "character": cursor[1]}}, "text": ch}]})
            cursor = [cursor[0] + 1, 0] if ch == "\n" else [cursor[0], cursor[1] + 1]
            if ch in " .(r":
                request("textDocument/completion", {"textDocument": doc, "position": {"line": cursor[0], "character": cursor[1]}})
                request("textDocument/semanticTokens/full", {"textDocument": doc})
        version += 1
        notify("textDocument/didChange", {"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]})
        request("textDocument/documentSymbol", {"textDocument": doc})
        request("workspace/symbol", {"query": "print"})
        notify("textDocument/didClose", {"textDocument": doc})
    request("shutdown", None)
    notify("exit", None)
    data = b"".join(b"Content-Length: %d\r\n\r\n%s" % (len(body), body)
                    for body in (json.dumps(m).encode() for m in messages))
    try:
        result = subprocess.run([str(jailsp)], input=data, capture_output=True, env=env, cwd=ROOT, timeout=900)
    except subprocess.TimeoutExpired:
        return "timeout", f"{len(messages)} messages"
    return result.returncode, f"{len(messages)} messages, {len(result.stdout)} bytes of responses"


def train(bins, args, env, native):
    """Run the training workload with `bins`. Failures are reported but do not stop the build:
    the profile only needs representative work, and the sweep is the correctness gate."""
    py = sys.executable
    sweep = [py, ROOT / "tools/jaic-sweep.py", "--jaic", bins["jaic"], "--allow-stale", "--headless",
             "--jobs", str(args.jobs), "--timeout", "300"]
    steps = [("sweep (check and run)", sweep + ["corpus", "negative", "stdlib", "modules", "examples"])]
    if native:
        steps += [("sweep (native -O0)", sweep + ["--opt", "O0", "corpus", "stdlib", "examples"]),
                  ("sweep (native -O2)", sweep + ["--opt", "O2", "corpus", "examples"])]
    for bench in sorted((ROOT / "benchmarks").glob("*.jai")):
        steps.append((f"run {bench.name}", [bins["jaic"], "run", bench]))
    scratch = Path(tempfile.mkdtemp(prefix="jaic-pgo-"))
    jaifmt = ROOT / "jaifmt/main.jai"
    if native:
        steps += [(f"build jaifmt -{o}", [bins["jaic"], "build", jaifmt, f"-{o}", "-o", scratch / f"jaifmt-{o}{EXE}"])
                  for o in ("O0", "O2")]
    else:
        steps.append(("check jaifmt", [bins["jaic"], "check", jaifmt]))
    steps += [("check tour", [bins["jaic"], "check", ROOT / "examples/tour/main.jai"]),
              ("jailint stdlib", [bins["jailint"], "--color", "never", "-j", str(args.jobs), "stdlib"]),
              ("jailint examples and tests", [bins["jailint"], "--color", "never", "examples", "tests/stdlib"])]
    problems = []
    for name, cmd in steps:
        start = time.monotonic()
        quiet = None if args.verbose else subprocess.DEVNULL
        try:
            # A stuck step must not eat the release job's time limit.
            code = run(cmd, env=env, check=False, stdout=quiet, stderr=quiet, timeout=args.step_timeout).returncode
        except subprocess.TimeoutExpired:
            code = "timeout"
        log(f"{name}: exit {code} in {time.monotonic() - start:.0f}s")
        # jailint exits 1 when it finds something, which is fine here.
        if code not in (0, 1) if name.startswith("jailint") else code != 0:
            problems.append(f"{name} (exit {code})")
    start = time.monotonic()
    code, summary = lsp_session(bins["jailsp"], env)
    log(f"jailsp session: exit {code}, {summary} in {time.monotonic() - start:.0f}s")
    if code != 0:
        problems.append(f"jailsp session (exit {code})")
    shutil.rmtree(scratch, ignore_errors=True)
    for p in problems:
        log(f"warning: training step failed: {p}")
    return problems


# --- driver ------------------------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--target-dir", default=os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target"))
    ap.add_argument("--target", default="", help="Rust target triple (default: CARGO_BUILD_TARGET, else the host)")
    ap.add_argument("--llvm", choices=sorted(LLVM_FEATURES), default="dynamic",
                    help="how jaic links LLVM: dynamic (development default), static (release archives), none")
    ap.add_argument("--jobs", "-j", type=int, default=4, help="cargo jobs and parallel training cases")
    ap.add_argument("--out", default="", help="where the optimized binaries go (default <target-dir>/pgo/dist)")
    ap.add_argument("--profile", default="", help="use this merged .profdata instead of training")
    ap.add_argument("--bolt", action="store_true", help="also optimize with llvm-bolt (Linux only)")
    ap.add_argument("--llvm-bin", default="", help="directory with llvm-bolt and merge-fdata")
    ap.add_argument("--step-timeout", type=float, default=1800, help="seconds one training step may take")
    ap.add_argument("--strict", action="store_true", help="fail when a training step fails")
    ap.add_argument("--verbose", "-v", action="store_true", help="show the training steps' output")
    args = ap.parse_args()
    args.target = args.target or os.environ.get("CARGO_BUILD_TARGET") or host_triple()
    if args.bolt and not sys.platform.startswith("linux"):
        sys.exit("build_pgo: BOLT only rewrites ELF binaries; use it on Linux (macOS and Windows get PGO only)")
    work = Path(args.target_dir).resolve() / "pgo"
    if " " in str(work):
        sys.exit(f"build_pgo: {work} contains a space, which RUSTFLAGS cannot carry; pass --target-dir")
    out = Path(args.out).resolve() if args.out else work / "dist"
    native = args.llvm != "none"
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    problems = []

    if args.profile:
        profile = Path(args.profile).resolve()
    else:
        profdata = profdata_tool()
        raw = work / "profraw"
        shutil.rmtree(raw, ignore_errors=True)
        raw.mkdir(parents=True)
        log("building instrumented binaries")
        bins = cargo_build(args, work / "instrumented", [f"-Cprofile-generate={raw}"])
        # `%4m`: up to four files per binary, merged as processes exit, instead of one
        # file per process (hundreds of jaic runs at several MB each).
        train_env = dict(env, LLVM_PROFILE_FILE=str(raw / "%4m.profraw"))
        problems += train(bins, args, train_env, native)
        profile = work / "merged.profdata"
        run([profdata, "merge", "-o", profile, *sorted(raw.glob("*.profraw"))])
        run([profdata, "show", profile], stdout=subprocess.DEVNULL)

    log("building with the profile")
    flags = [f"-Cprofile-use={profile}"]
    if args.bolt:
        # BOLT needs the relocations to move code around.
        flags.append("-Clink-arg=-Wl,--emit-relocs")
    bins = cargo_build(args, work / "optimized", flags)

    if args.bolt:
        llvm_bolt, merge_fdata = bolt_tool("llvm-bolt", args.llvm_bin), bolt_tool("merge-fdata", args.llvm_bin)
        bolt = work / "bolt"
        shutil.rmtree(bolt, ignore_errors=True)
        bolt.mkdir(parents=True)
        instrumented = {}
        for tool, path in bins.items():
            (bolt / tool).mkdir()
            instrumented[tool] = bolt / f"{tool}.instrumented"
            run([llvm_bolt, path, "-instrument", "-o", instrumented[tool],
                 f"--instrumentation-file={bolt / tool / 'prof.fdata'}", "--instrumentation-file-append-pid"])
        log("training the BOLT-instrumented binaries")
        problems += train(instrumented, args, env, native)
        for tool, path in bins.items():
            fdata = bolt / f"{tool}.fdata"
            parts = sorted((bolt / tool).glob("prof.fdata*"))  # one per process: prof.fdata.<pid>
            if not parts:
                sys.exit(f"build_pgo: no BOLT profile was written for {tool}")
            with open(fdata, "w") as f:
                run([merge_fdata, *parts], stdout=f)
            optimized = bolt / tool
            run([llvm_bolt, path, "-o", f"{optimized}.bolt", f"-data={fdata}",
                 "-reorder-blocks=ext-tsp", "-reorder-functions=cdsort", "-split-functions",
                 "-split-all-cold", "-split-eh", "-icf=all", "-use-gnu-stack", "-dyno-stats"])
            bins[tool] = Path(f"{optimized}.bolt")

    out.mkdir(parents=True, exist_ok=True)
    for tool, path in bins.items():
        shutil.copy2(path, out / f"{tool}{EXE}")
    log(f"optimized binaries in {out}")
    if problems and args.strict:
        sys.exit("build_pgo: training steps failed: " + ", ".join(problems))


if __name__ == "__main__":
    main()
