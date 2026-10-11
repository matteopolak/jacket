#!/usr/bin/env python3
"""Developer commands behind the Justfile: build, format, lint and test the repository.

    python3 tools/dev.py build [--release] [-j N]
    python3 tools/dev.py fmt  [--check] [--lang jai|rust|all] [--staged] [-j N] [paths...]
    python3 tools/dev.py lint [--fix] [--lang jai|rust|all] [--staged] [-j N] [paths...]
    python3 tools/dev.py editor
    python3 tools/dev.py nvim [--unit] [-j N]
    python3 tools/dev.py test [--suite cargo|tools|all] [-p CRATE] [-j N] [filter...]

The recipes in the Justfile are thin wrappers; this file holds the logic so that CI and a local
shell run the same commands. Runs on Python 3.9 (macOS /usr/bin/python3): no tomllib here.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXE = ".exe" if os.name == "nt" else ""

# What CI formats and lints: every Jai tree the repository owns.
JAI_DIRS = ["stdlib", "tests", "benchmarks", "tools", "jaifmt", "examples"]
JAI_LINT_DIRS = ["stdlib", "examples", "tools", "jaifmt", "tests", "benchmarks"]
JAIFMT_SOURCES = ["jaifmt", "stdlib/Extensions/Jai_Format", "stdlib/Extensions/Args"]


def target_directory() -> Path:
    """Cargo's real target directory (CARGO_TARGET_DIR and .cargo/config.toml are honoured)."""
    if os.environ.get("CARGO_TARGET_DIR"):
        return Path(os.environ["CARGO_TARGET_DIR"]).resolve()
    out = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
                         cwd=ROOT, capture_output=True, text=True)
    if out.returncode:
        return ROOT / "target"
    return Path(json.loads(out.stdout)["target_directory"])


TARGET = target_directory()


def run(cmd: list[str], cwd: Path = ROOT) -> int:
    print("+ " + " ".join(cmd), file=sys.stderr, flush=True)
    return subprocess.run(cmd, cwd=cwd).returncode


def cargo_build(args: list[str], jobs: str) -> int:
    return run(["cargo", "build", "-j", jobs, "--locked", *args])


def staged(lang: str) -> list[str]:
    out = subprocess.run(["sh", str(ROOT / "tools" / "staged-files.sh"), lang],
                         cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return [line for line in out.splitlines() if line]


def pick(paths: list[str], use_staged: bool) -> tuple[list[str], list[str], bool]:
    """The Jai and Rust files to act on, and whether the whole tree was asked for."""
    if use_staged:
        if paths:
            sys.exit("--staged and explicit paths cannot be combined")
        return staged("jai"), staged("rust"), False
    if paths:
        return ([p for p in paths if p.endswith(".jai") or (ROOT / p).is_dir()],
                [p for p in paths if p.endswith(".rs") or (ROOT / p).is_dir()], False)
    return [], [], True


def wants(lang: str, which: str) -> bool:
    return lang in ("all", which)


def newest(paths: list[str]) -> float:
    latest = 0.0
    for p in paths:
        base = ROOT / p
        files = [base] if base.is_file() else [f for f in base.rglob("*") if f.is_file()]
        for f in files:
            if f.suffix in (".jai", ".toml", ".rs"):
                latest = max(latest, f.stat().st_mtime)
    return latest


def ensure_jailint(jobs: str) -> Path:
    if cargo_build(["-p", "jailint"], jobs):
        sys.exit("could not build jailint")
    return TARGET / "debug" / ("jailint" + EXE)


def ensure_jaifmt(jobs: str) -> Path:
    """target/jaifmt, built by jaic as in CI; rebuilt when jaic or jaifmt's sources are newer."""
    if cargo_build(["-p", "jaic-cli"], jobs):
        sys.exit("could not build jaic")
    jaic = TARGET / "debug" / ("jaic" + EXE)
    out = TARGET / ("jaifmt" + EXE)
    if not out.exists() or out.stat().st_mtime < max(newest(JAIFMT_SOURCES), jaic.stat().st_mtime):
        if run([str(jaic), "build", "jaifmt/main.jai", "-O2", "-o", str(out)]):
            sys.exit("could not build jaifmt")
    return out


def cmd_build(args) -> int:
    flags = ["--release"] if args.release else []
    if cargo_build([*flags, "-p", "jaic-cli", "-p", "jai-language-server", "-p", "jailint"],
                   args.jobs):
        return 1
    jaic = TARGET / ("release" if args.release else "debug") / ("jaic" + EXE)
    return run([str(jaic), "build", "jaifmt/main.jai", "-O2", "-o", str(TARGET / ("jaifmt" + EXE))])


def cmd_fmt(args) -> int:
    jai, rust, whole = pick(args.paths, args.staged)
    failed = False
    if wants(args.lang, "jai") and (whole or jai):
        jaifmt = ensure_jaifmt(args.jobs)
        failed |= bool(run([str(jaifmt), *(["--check"] if args.check else []), *(jai or JAI_DIRS)]))
    if wants(args.lang, "rust") and (whole or rust):
        check = ["--check"] if args.check else []
        spacing = [sys.executable, str(ROOT / "tools" / "rust_item_spacing.py"), *check]
        if whole and args.check and sys.version_info >= (3, 11):
            # CI's check: also fails when an authored crate is outside the formatted workspace.
            failed |= bool(run([sys.executable, str(ROOT / "tools" / "check_rust_format.py")]))
        elif whole:
            failed |= bool(run(["cargo", "fmt", "--all", *(["--", "--check"] if args.check else [])]))
            failed |= bool(run(spacing))
        else:
            failed |= bool(run(["rustfmt", "--edition", "2024", *check, *rust]))
            failed |= bool(run([*spacing, *rust]))
    return int(failed)


def cmd_lint(args) -> int:
    jai, rust, whole = pick(args.paths, args.staged)
    failed = False
    if wants(args.lang, "jai") and (whole or jai):
        jailint = ensure_jailint(args.jobs)
        failed |= bool(run([str(jailint), "-D", "warnings", "-j", "2",
                            *(["--fix"] if args.fix else []), *(jai or JAI_LINT_DIRS)]))
    if wants(args.lang, "rust") and (whole or rust):
        # Clippy works on whole crates; the file list only decides whether it runs at all.
        fix = ["--fix", "--allow-dirty", "--allow-staged"] if args.fix else []
        failed |= bool(run(["cargo", "clippy", "-j", args.jobs, "--workspace", "--all-targets",
                            "--locked", *fix, "--", "-D", "warnings"]))
    return int(failed)


def cmd_editor(args) -> int:
    """Check that the VS Code extension's generated grammar and schemas are current (needs node)."""
    node = shutil.which("node")
    if node is None:
        print("node not found: skipping the VS Code grammar and schema checks (CI runs them)")
        return 0
    version = subprocess.run([node, "--version"], capture_output=True, text=True).stdout.strip()
    try:
        major = int(version.lstrip("v").split(".")[0])
    except ValueError:
        major = 0
    if major < 24:
        # The scripts are TypeScript run by node itself, which needs type stripping.
        print(f"node {version} cannot run the .ts scripts (needs 24): skipping the VS Code checks")
        return 0
    failed = False
    for script in ("build-grammar.ts", "build-schemas.ts"):
        failed |= bool(run([node, f"scripts/{script}", "--check"], cwd=ROOT / "editors" / "vscode"))
    return int(failed)


def cmd_nvim(args) -> int:
    """Run the Neovim plugin's headless tests (needs nvim >= 0.11); --unit leaves out the jailsp ones."""
    nvim = shutil.which("nvim")
    if nvim is None:
        print("nvim not found: skipping the Neovim plugin tests (CI runs them)")
        return 0
    banner = subprocess.run([nvim, "--version"], capture_output=True, text=True).stdout.splitlines()
    found = re.match(r"NVIM v(\d+)\.(\d+)", banner[0]) if banner else None
    if found is None or (int(found.group(1)), int(found.group(2))) < (0, 11):
        print(f"{banner[0] if banner else 'nvim'} is older than 0.11: skipping the Neovim plugin tests")
        return 0
    env = {k: v for k, v in os.environ.items() if k != "JAILSP"}
    if not args.unit:
        if cargo_build(["-p", "jai-language-server"], args.jobs):
            return 1
        env["JAILSP"] = str(TARGET / "debug" / ("jailsp" + EXE))
    print("+ nvim --headless --clean -u tests/minimal_init.lua -c 'luafile tests/run.lua'", file=sys.stderr, flush=True)
    return subprocess.run([nvim, "--headless", "--clean", "-u", "tests/minimal_init.lua", "-c", "luafile tests/run.lua"],
                          cwd=ROOT / "editors" / "nvim", env=env).returncode


def cmd_test(args) -> int:
    failed = False
    narrowed = bool(args.crate or args.filter)
    if args.suite in ("cargo", "all"):
        scope = ["-p", args.crate] if args.crate else ["--workspace"]
        # CI's skips: they need the separately supplied reference sources, or run in their own step.
        skips = [] if narrowed else [
            "--skip", "lex_entire_reference_without_executing_it",
            "--skip", "every_stdlib_module_checks_for_every_target",
            "--skip", "selectors_take_the_arguments_they_are_sent_with",
            "--skip", "selectors_exist_in_the_objective_c_runtime"]
        failed |= bool(run(["cargo", "test", "-j", args.jobs, "--locked", "--no-fail-fast",
                            *scope, "--", *args.filter, *skips]))
    if args.suite in ("tools", "all") and not narrowed:
        failed |= bool(run([sys.executable, "-m", "unittest", "discover", "-s", "tools",
                            "-p", "test_*.py"]))
    return int(failed)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    for name, fn in (("build", cmd_build), ("fmt", cmd_fmt), ("lint", cmd_lint), ("editor", cmd_editor),
                     ("nvim", cmd_nvim), ("test", cmd_test)):
        p = sub.add_parser(name)
        p.set_defaults(fn=fn)
        p.add_argument("-j", "--jobs", default="3")
        if name == "build":
            p.add_argument("--release", action="store_true")
        if name in ("fmt", "lint"):
            p.add_argument("--lang", choices=["jai", "rust", "all"], default="all")
            p.add_argument("--staged", action="store_true")
            p.add_argument("paths", nargs="*")
            p.add_argument("--check" if name == "fmt" else "--fix", action="store_true")
        if name == "nvim":
            p.add_argument("--unit", action="store_true")
        if name == "test":
            p.add_argument("--suite", choices=["cargo", "tools", "all"], default="all")
            p.add_argument("-p", "--crate", default="")
            p.add_argument("filter", nargs="*")
    args = parser.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
