#!/usr/bin/env python3
"""Developer commands behind the Justfile: build, format, lint and test the repository.

    python3 tools/dev/dev.py build [--release] [--no-native] [-j N]
    python3 tools/dev/dev.py fmt  [--check] [--lang jai|rust|all] [--staged] [-j N] [paths...]
    python3 tools/dev/dev.py lint [--fix] [--lang jai|rust|all] [--os linux|windows|macos|wasm] [--staged] [-j N] [paths...]
    python3 tools/dev/dev.py editor
    python3 tools/dev/dev.py nvim [--unit] [-j N]
    python3 tools/dev/dev.py vscode-try [--debug] [-j N] [folder]
    python3 tools/dev/dev.py test [--suite cargo|tools|all] [-p CRATE] [-j N] [filter...]

The recipes in the Justfile are thin wrappers; this file holds the logic so that CI and a local
shell run the same commands. Runs on Python 3.9 (macOS /usr/bin/python3): no tomllib here.
"""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))


import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from toolkit.process import run as toolkit_run
from toolkit.repo import ROOT, cargo_target_dir, jaifmt_path

EXE = ".exe" if os.name == "nt" else ""

# What CI formats and lints: every Jai tree the repository owns.
JAI_DIRS = ["stdlib", "tests", "benchmarks", "tools", "jaifmt", "examples"]
JAI_LINT_DIRS = ["stdlib", "examples", "tools", "jaifmt", "tests", "benchmarks"]
JAIFMT_SOURCES = ["jaifmt", "stdlib/Extensions/Jai_Format", "stdlib/Extensions/Args"]


TARGET = cargo_target_dir(resolve=True)


def run(cmd: list[str], cwd: Path = ROOT) -> int:
    return toolkit_run(cmd, cwd=cwd, check=False).returncode


def cargo_build(args: list[str], jobs: str) -> int:
    return run(["cargo", "build", "-j", jobs, "--locked", *args])


def staged(lang: str) -> list[str]:
    out = subprocess.run(["sh", str(ROOT / "tools" / "dev" / "staged-files.sh"), lang],
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


def ensure_jailint(jobs: str, release: bool = False) -> Path:
    if cargo_build([*(["--release"] if release else []), "-p", "jailint"], jobs):
        sys.exit("could not build jailint")
    return TARGET / ("release" if release else "debug") / ("jailint" + EXE)


def ensure_jaifmt(jobs: str) -> Path:
    """target/jaifmt, built by jaic as in CI; rebuilt when jaic or jaifmt's sources are newer."""
    if cargo_build(["-p", "jaic-cli"], jobs):
        sys.exit("could not build jaic")
    jaic = TARGET / "debug" / ("jaic" + EXE)
    out = jaifmt_path()
    if not out.exists() or out.stat().st_mtime < max(newest(JAIFMT_SOURCES), jaic.stat().st_mtime):
        if run([str(jaic), "jaifmt/main.jai", "-O2", "-exe", str(out)]):
            sys.exit("could not build jaifmt")
    return out


DEVELOPER_TOOLS = ROOT / "stdlib" / "internal" / "developer-tools"
HELPER_SUFFIX = {"darwin": "dylib", "win32": "dll"}.get(sys.platform, "so")


def stale(output: Path, *sources: Path) -> bool:
    return not output.exists() or output.stat().st_mtime < max(s.stat().st_mtime for s in sources)


LLVM_HELP = ("install LLVM 23.1 and point LLVM_SYS_231_PREFIX at it (macOS: `brew install llvm@23` then "
             "`export LLVM_SYS_231_PREFIX=\"$(brew --prefix llvm@23)\"`); see docs/tools/llvm-setup.md")


def llvm_prefix() -> tuple[Path | None, str | None]:
    """The LLVM that llvm-sys will build against (LLVM_SYS_231_PREFIX, else llvm-config on
    PATH), or why there is none."""
    configured = os.environ.get("LLVM_SYS_231_PREFIX")
    if configured:
        config = Path(configured) / "bin" / ("llvm-config" + EXE)
        if not config.is_file():
            return None, f"LLVM_SYS_231_PREFIX={configured} has no bin/llvm-config: {LLVM_HELP}"
    else:
        found = shutil.which("llvm-config-23") or shutil.which("llvm-config")
        if not found:
            return None, f"LLVM 23.1 not found (LLVM_SYS_231_PREFIX is not set and no llvm-config is on PATH): {LLVM_HELP}"
        config = Path(found)
    try:
        version = subprocess.check_output([str(config), "--version"], text=True).strip()
        prefix = Path(subprocess.check_output([str(config), "--prefix"], text=True).strip())
    except (OSError, subprocess.CalledProcessError) as error:
        return None, f"{config} does not run ({error}): {LLVM_HELP}"
    if not version.startswith("23.1"):
        return None, f"{config} is LLVM {version}, and the compiler needs 23.1: {LLVM_HELP}"
    return prefix, None


def missing_dependencies(native: bool) -> list[str]:
    """Every tool `build` would fail on, each with how to get it, so one run names them all."""
    problems = []
    if not shutil.which("cargo"):
        problems.append("cargo not found: install Rust with rustup (https://rustup.rs)")
    prefix, llvm = llvm_prefix()
    if llvm:
        problems.append(llvm)
    if not native:
        return problems
    compilers = ["clang"] if sys.platform == "win32" else ["cc", "c++"]
    absent = [f"`{tool}`" for tool in compilers if not shutil.which(tool)]
    if absent:
        problems.append(f"{' and '.join(absent)} not found: the native libraries need a C and C++ compiler "
                        "(macOS: `xcode-select --install`; Debian/Ubuntu: `apt install build-essential`)")
    from native import build_native_libs
    if "SDL3" in build_native_libs.missing() and not shutil.which("cmake"):
        problems.append("`cmake` not found: SDL3 builds with CMake (`brew install cmake`, `apt install cmake`), "
                        "or skip the native libraries with `just build --native=false`")
    clang = DEVELOPER_TOOLS / "clang-bindings"
    if (prefix and sys.platform != "win32" and stale(clang / "developer-tools-clang", clang / "main.c")
            and not os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX")
            and not (prefix / "include" / "clang-c" / "Index.h").is_file()):
        problems.append(f"{prefix} has no include/clang-c/Index.h: Bindings_Generator's helper needs an LLVM "
                        "with libclang (Homebrew's llvm@23 and apt's libclang-23-dev have it)")
    return problems


def build_native() -> int:
    """What a release archive carries beside stdlib/ and a checkout builds itself: the
    third-party libraries under target/native-libs (only the missing ones) and the
    Bindings_Generator helpers in stdlib/internal/developer-tools (only when stale)."""
    if run([sys.executable, str(ROOT / "tools" / "native" / "build_native_libs.py"), "--missing"]):
        return 1
    checks = ROOT / "tools" / "developer_checks"
    platform_lib = DEVELOPER_TOOLS / f"libgenerator_platform.{HELPER_SUFFIX}"
    if stale(platform_lib, DEVELOPER_TOOLS / "generator-platform-native.c"):
        if run([sys.executable, str(checks / "build_generator_platform.py")]):
            return 1
    # The clang helper builds on macOS and Linux only; Windows release archives ship libclang.
    clang = DEVELOPER_TOOLS / "clang-bindings"
    if sys.platform != "win32" and stale(clang / "developer-tools-clang", clang / "main.c"):
        if run([sys.executable, str(checks / "build_developer_bindings.py")]):
            return 1
    return 0


def cmd_build(args) -> int:
    problems = missing_dependencies(not args.no_native)
    if problems:
        print("just build: missing dependencies:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1
    flags = ["--release"] if args.release else []
    if cargo_build([*flags, "-p", "jaic-cli", "-p", "jai-language-server", "-p", "jailint"],
                   args.jobs):
        return 1
    jaic = TARGET / ("release" if args.release else "debug") / ("jaic" + EXE)
    if run([str(jaic), "jaifmt/main.jai", "-O2", "-exe", str(jaifmt_path())]):
        return 1
    return 0 if args.no_native else build_native()


def cmd_fmt(args) -> int:
    jai, rust, whole = pick(args.paths, args.staged)
    failed = False
    if wants(args.lang, "jai") and (whole or jai):
        jaifmt = ensure_jaifmt(args.jobs)
        failed |= bool(run([str(jaifmt), *(["--check"] if args.check else []), *(jai or JAI_DIRS)]))
    if wants(args.lang, "rust") and (whole or rust):
        check = ["--check"] if args.check else []
        spacing = [sys.executable, str(ROOT / "tools" / "dev" / "rust_item_spacing.py"), *check]
        if whole and args.check and sys.version_info >= (3, 11):
            # CI's check: also fails when an authored crate is outside the formatted workspace.
            failed |= bool(run([sys.executable, str(ROOT / "tools" / "dev" / "check_rust_format.py")]))
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
        jailint = ensure_jailint(args.jobs, args.release)
        # A platform's modules (Windows, X11, Android) cannot compile on another host: each CI
        # host lints what it can, and a finding in what does compile still fails the run.
        # `platform_leak` compiles every target itself, so a run for one target leaves it out.
        failed |= bool(run([str(jailint), "-D", "warnings", "-j", args.jobs, "--allow-incomplete",
                            *(["-os", args.os, "-A", "platform_leak"] if args.os else []),
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
    for script in (["build-grammar.ts"], ["build.ts", "schemas"]):
        failed |= bool(run([node, f"scripts/{script[0]}", *script[1:], "--check"], cwd=ROOT / "editors" / "vscode"))
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
    if "MINI_TEST_DIR" not in env:
        fetched = subprocess.run([sys.executable, str(ROOT / "tools" / "dev" / "fetch_mini_test.py")], capture_output=True, text=True)
        if fetched.returncode:
            print(fetched.stderr, file=sys.stderr)
            return 1
        env["MINI_TEST_DIR"] = fetched.stdout.strip()
    if not args.unit:
        if cargo_build(["-p", "jai-language-server"], args.jobs):
            return 1
        env["JAILSP"] = str(TARGET / "debug" / ("jailsp" + EXE))
    print("+ nvim -n --headless --clean -u tests/minimal_init.lua -c 'luafile tests/run.lua'", file=sys.stderr, flush=True)
    return subprocess.run([nvim, "-n", "--headless", "--clean", "-u", "tests/minimal_init.lua", "-c", "luafile tests/run.lua"],
                          cwd=ROOT / "editors" / "nvim", env=env).returncode


def vscode_settings(bin_dir: Path, formatter: Path, platform: str) -> dict:
    """Editor commands and new terminals use the same development toolchain."""
    suffix = ".exe" if platform == "win32" else ""
    terminal_platform = {"darwin": "osx", "win32": "windows"}.get(platform, "linux")
    separator = ";" if platform == "win32" else ":"
    return {
        "jai.server.path": str(bin_dir / ("jailsp" + suffix)),
        "jai.compiler.path": str(bin_dir / ("jaic" + suffix)),
        "jai.formatter.path": str(formatter),
        "jai.stdlib.path": str(ROOT / "stdlib"),
        f"terminal.integrated.env.{terminal_platform}": {
            "PATH": separator.join([str(bin_dir), str(formatter.parent), "${env:PATH}"]),
            "JAIC_STDLIB": str(ROOT / "stdlib"),
        },
        # The extension ignores the folder's own tool paths in Restricted Mode.
        "security.workspace.trust.enabled": False,
        "update.mode": "none",
        "extensions.autoCheckUpdates": False,
        "telemetry.telemetryLevel": "off",
    }


def cmd_vscode_try(args) -> int:
    """Build the tools and the VS Code extension, then open it alone in a throwaway VS Code profile.

    `folder` is opened in place; without one, a copy of examples/ is, so edits never touch the repository.
    The profile lives in a new temporary folder (VS Code's IPC socket path must stay under 104 bytes).
    """
    code = shutil.which("code")
    if code is None and sys.platform == "darwin":
        bundled = Path("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code")
        code = str(bundled) if bundled.exists() else None
    if code is None:
        print("VS Code's `code` command not found (Command Palette: Shell Command: Install 'code' command in PATH)", file=sys.stderr)
        return 1
    profile = "debug" if args.debug else "release"
    if cargo_build([*([] if args.debug else ["--release"]), "-p", "jacket", "-p", "jaic-cli", "-p", "jai-language-server", "-p", "jailint"],
                   args.jobs):
        return 1
    bin_dir = TARGET / profile
    jaifmt = jaifmt_path()
    jaic = bin_dir / ("jaic" + EXE)
    if not jaifmt.exists() or jaifmt.stat().st_mtime < max(newest(JAIFMT_SOURCES), jaic.stat().st_mtime):
        if run([str(jaic), "jaifmt/main.jai", "-O2", "-exe", str(jaifmt)]):
            return 1
    extension = ROOT / "editors" / "vscode"
    pnpm = shutil.which("pnpm")
    if pnpm is None:
        print("pnpm not found (corepack enable)", file=sys.stderr)
        return 1
    if run([pnpm, "install", "--frozen-lockfile", "--ignore-scripts"], cwd=extension) or run([pnpm, "run", "build"], cwd=extension):
        return 1

    scratch = Path(tempfile.mkdtemp(prefix="jacket-vscode-"))
    if args.folder:
        workspace = Path(args.folder).resolve()
    else:
        workspace = scratch / "examples"
        shutil.copytree(ROOT / "examples", workspace)
    settings = vscode_settings(bin_dir, jaifmt, sys.platform)
    user = scratch / "user" / "User"
    user.mkdir(parents=True)
    (user / "settings.json").write_text(json.dumps(settings, indent=2) + "\n")
    print(f"+ code {workspace} (profile in {scratch})", file=sys.stderr, flush=True)
    return subprocess.run([code, "--new-window", "--user-data-dir", str(scratch / "user"),
                           "--extensions-dir", str(scratch / "extensions"),
                           f"--extensionDevelopmentPath={extension}", str(workspace)]).returncode


def cmd_test(args) -> int:
    failed = False
    narrowed = bool(args.crate or args.filter)
    if args.suite in ("cargo", "all"):
        scope = ["-p", args.crate] if args.crate else ["--workspace"]
        if shutil.which("cargo-nextest") is None:
            print("cargo-nextest not found: `brew install cargo-nextest` or `cargo install --locked cargo-nextest`", file=sys.stderr)
            return 1
        # The native tests link the pinned third-party libraries; jaic finds them in the default output
        # directory, so build whatever is missing there first (a no-op once they exist, and in CI after
        # tools/native/ci_native_deps.sh).
        if run([sys.executable, str(ROOT / "tools" / "native" / "build_native_libs.py"), "--missing"]):
            print("could not build the native libraries the native tests link", file=sys.stderr)
            return 1
        # The `ci` profile (.config/nextest.toml) leaves out the tests that need the separately supplied
        # reference sources or run in their own CI step; naming a crate or filter runs those too.
        profile = ["--ignore-default-filter"] if narrowed else []
        failed |= bool(run(["cargo", "nextest", "run", "--profile", "ci", "-j", args.jobs, "--locked", "--no-fail-fast",
                            *profile, *scope, *args.filter]))
        # nextest does not run doctests.
        failed |= bool(run(["cargo", "test", "--doc", "-j", args.jobs, "--locked", *scope]))
    if args.suite in ("tools", "all") and not narrowed:
        failed |= bool(run([sys.executable, "tools/run_tests.py"]))
    return int(failed)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    for name, fn in (("build", cmd_build), ("fmt", cmd_fmt), ("lint", cmd_lint), ("editor", cmd_editor),
                     ("nvim", cmd_nvim), ("vscode-try", cmd_vscode_try), ("test", cmd_test)):
        p = sub.add_parser(name)
        p.set_defaults(fn=fn)
        p.add_argument("-j", "--jobs", default="3")
        if name in ("build", "lint"):
            p.add_argument("--release", action="store_true")
        if name == "build":
            p.add_argument("--no-native", action="store_true",
                           help="skip the native libraries and Bindings_Generator helpers")
        if name in ("fmt", "lint"):
            p.add_argument("--lang", choices=["jai", "rust", "all"], default="all")
            p.add_argument("--staged", action="store_true")
            if name == "lint":
                p.add_argument("--os", choices=["linux", "windows", "macos", "wasm"], default="")
            p.add_argument("paths", nargs="*")
            p.add_argument("--check" if name == "fmt" else "--fix", action="store_true")
        if name == "nvim":
            p.add_argument("--unit", action="store_true")
        if name == "vscode-try":
            p.add_argument("--debug", action="store_true")
            p.add_argument("folder", nargs="?", default="")
        if name == "test":
            p.add_argument("--suite", choices=["cargo", "tools", "all"], default="all")
            p.add_argument("-p", "--crate", default="")
            p.add_argument("filter", nargs="*")
    args = parser.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
