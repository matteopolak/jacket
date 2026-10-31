#!/usr/bin/env python3
"""Developer commands behind the Justfile: build, format, lint and test the repository.

    python3 tools/dev/dev.py build [--release] [--no-native] [-j N]
    python3 tools/dev/dev.py fmt  [--check] [--lang jai|rust|all] [--staged] [-j N] [paths...]
    python3 tools/dev/dev.py lint [--fix] [--lang jai|rust|all] [--os linux|windows|macos|wasm] [--staged] [-j N] [paths...]
    python3 tools/dev/dev.py editor
    python3 tools/dev/dev.py repo-checks [style|paths|ci-sources|stdlib-docs ...]
    python3 tools/dev/dev.py precommit
    python3 tools/dev/dev.py hooks
    python3 tools/dev/dev.py nvim [--unit] [-j N]
    python3 tools/dev/dev.py vscode-try [--debug] [-j N] [folder]
    python3 tools/dev/dev.py test [--suite cargo|tools|all] [-p CRATE] [-j N] [filter...]

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
import tempfile
from pathlib import Path
from toolkit import llvm
from toolkit.process import run as toolkit_run
from toolkit.host import exe_name
from toolkit.repo import ROOT, cargo_binary, cargo_target_dir, jaifmt_path

EXE = exe_name("")
TARGET = cargo_target_dir(resolve=True)

# What CI formats and lints: every Jai tree the repository owns.
JAI_DIRS = ["stdlib", "tests", "benchmarks", "tools", "jaifmt", "examples"]
JAI_LINT_DIRS = ["stdlib", "examples", "tools", "jaifmt", "tests", "benchmarks"]
JAIFMT_SOURCES = ["jaifmt", "stdlib/Extensions/Jai_Format", "stdlib/Extensions/Args"]
# What a jailint binary is built from; the pre-commit hook uses an existing one only when cargo is unavailable, and then only if none of these is newer.
JAILINT_SOURCES = ["crates/jailint", "crates/jaic", "crates/jaic-cli", "stdlib/prelude/module.jai", "Cargo.lock"]
# Repository checks that read only the working tree (tools/ci/check_<name>.py), in `repo-checks` order.
REPO_CHECKS = {"style": "check_source_style", "paths": "check_repo_paths", "ci-sources": "check_ci_sources",
               "stdlib-docs": "check_stdlib_docs"}
# Checks that need `jaic` built (they build it when it is not): `repo-checks` runs them, the pre-commit hook does not.
BUILT_REPO_CHECKS = {"stdlib-docs"}


def run(cmd: list[str], cwd: Path = ROOT) -> int:
    return toolkit_run(cmd, cwd=cwd, check=False).returncode


def cargo_build(args: list[str], jobs: str) -> int:
    return run(["cargo", "build", "-j", jobs, "--locked", *args])


def tracked(files: list[str]) -> list[str]:
    """The files git tracks in this checkout, whatever the caller listed. Nothing under `.claude/`
    (agent worktrees live there, each with its own standard library) survives."""
    status, listing = run_captured(["git", "-c", "core.quotepath=off", "ls-files", "--cached"])
    known = set(listing.split("\n")) if status == 0 else None
    return [f for f in files if not f.startswith(".claude/") and (known is None or f in known)]


def staged(lang: str) -> list[str]:
    out = subprocess.run(["sh", str(ROOT / "tools" / "dev" / "staged-files.sh"), lang],
                         cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return tracked([line for line in out.splitlines() if line])


def pick(paths: list[str], use_staged: bool) -> tuple[list[str], list[str], bool]:
    """The Jai and Rust files to act on, and whether the whole tree was asked for."""
    if use_staged:
        if paths:
            sys.exit("--staged and explicit paths cannot be combined")
        return staged("jai"), staged("rust"), False
    if paths:
        paths = [p for p in paths if not os.path.normpath(p).startswith(".claude")]
        return ([p for p in paths if p.endswith(".jai") or (ROOT / p).is_dir()],
                [p for p in paths if p.endswith(".rs") or (ROOT / p).is_dir()], False)
    return [], [], True


def wants(lang: str, which: str) -> bool:
    return lang in ("all", which)


def newest(paths: list[str | Path]) -> float:
    """The latest mtime among the files named, and the sources (.jai, .toml, .rs) under the directories."""
    latest = 0.0
    for p in paths:
        base = ROOT / p
        if base.is_file():
            latest = max(latest, base.stat().st_mtime)
            continue
        for f in base.rglob("*") if base.is_dir() else []:
            if f.suffix in (".jai", ".toml", ".rs") and f.is_file():
                latest = max(latest, f.stat().st_mtime)
    return latest


def stale(output: Path, *sources: str | Path) -> bool:
    """Is `output` missing, or older than any of the files and directories in `sources`?"""
    return not output.exists() or output.stat().st_mtime < newest(list(sources))


def ensure_jailint(jobs: str, release: bool = False) -> Path:
    if cargo_build([*(["--release"] if release else []), "-p", "jailint"], jobs):
        sys.exit("could not build jailint")
    return cargo_binary("jailint", "release" if release else "debug")


def jaifmt_stamp(out: Path) -> Path:
    """Next to a built jaifmt: the checkout it was built in. A shared cargo target directory can hold one
    built in another checkout (a deleted worktree's), whose sources and mtimes mean nothing here."""
    return out.with_name(out.name + ".checkout")


def built_elsewhere(out: Path) -> bool:
    """Was `out` built in a checkout other than this one? (False when it records no checkout.)"""
    stamp = jaifmt_stamp(out)
    return stamp.is_file() and stamp.read_text().strip() != str(ROOT)


def stale_jaifmt(out: Path, jaic: Path) -> bool:
    """Is jaifmt missing, older than `jaic` or its sources, or not recorded as built in this checkout?"""
    return stale(out, jaic, *JAIFMT_SOURCES) or not jaifmt_stamp(out).is_file() or built_elsewhere(out)


def rebuild_jaifmt(jaic: Path, quiet: bool = False) -> bool:
    """Build target/jaifmt with `jaic` as CI does, unless it was built from this checkout after `jaic` and jaifmt's sources."""
    out = jaifmt_path()
    if not stale_jaifmt(out, jaic):
        return True
    cmd = [str(jaic), "jaifmt/main.jai", "-O2", "-exe", str(out)]
    if quiet:
        built = subprocess.run(cmd, cwd=ROOT, capture_output=True).returncode == 0 and out.exists()
    else:
        built = run(cmd) == 0
    if built:
        jaifmt_stamp(out).write_text(str(ROOT))
    return built


def ensure_jaifmt(jobs: str) -> Path:
    """target/jaifmt, built by jaic as in CI; rebuilt when jaic or jaifmt's sources are newer."""
    if cargo_build(["-p", "jaic-cli"], jobs):
        sys.exit("could not build jaic")
    if not rebuild_jaifmt(cargo_binary("jaic")):
        sys.exit("could not build jaifmt")
    return jaifmt_path()


DEVELOPER_TOOLS = ROOT / "stdlib" / "internal" / "developer-tools"
HELPER_SUFFIX = {"darwin": "dylib", "win32": "dll"}.get(sys.platform, "so")


def llvm_help() -> str:
    var = llvm.sys_prefix_var()
    return (f"install LLVM {llvm.major_minor()} and point {var} at it (macOS: `brew install llvm@{llvm.major()}` then "
            f"`export {var}=\"$(brew --prefix llvm@{llvm.major()})\"`); see docs/tools/llvm-setup.md")


def llvm_prefix() -> tuple[Path | None, str | None]:
    """The LLVM that llvm-sys will build against (the llvm-sys variable, else `LLVM_PREFIX`, else llvm-config on
    PATH), or why there is none."""
    var, help_text = llvm.sys_prefix_var(), llvm_help()
    configured = llvm.configured_prefix()
    if configured:
        config = llvm.llvm_config(configured)
        if not config:
            return None, f"{configured} has no bin/llvm-config ({var} or LLVM_PREFIX): {help_text}"
    else:
        config = llvm.llvm_config()
        if not config:
            return None, f"LLVM {llvm.major_minor()} not found ({var} is not set and no llvm-config is on PATH): {help_text}"
    try:
        version = subprocess.check_output([str(config), "--version"], text=True).strip()
        prefix = Path(subprocess.check_output([str(config), "--prefix"], text=True).strip())
    except (OSError, subprocess.CalledProcessError) as error:
        return None, f"{config} does not run ({error}): {help_text}"
    if not version.startswith(llvm.major_minor()):
        return None, f"{config} is LLVM {version}, and the compiler needs {llvm.major_minor()}: {help_text}"
    return prefix, None


def missing_dependencies(native: bool) -> list[str]:
    """Every tool `build` would fail on, each with how to get it, so one run names them all."""
    problems = []
    if not shutil.which("cargo"):
        problems.append("cargo not found: install Rust with rustup (https://rustup.rs)")
    prefix, llvm_problem = llvm_prefix()
    if llvm_problem:
        problems.append(llvm_problem)
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
                        f"with libclang (Homebrew's llvm@{llvm.major()} and apt's libclang-{llvm.major()}-dev have it)")
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
    jaic = cargo_binary("jaic", "release" if args.release else "debug")
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


EDITOR_SCRIPTS = [["build-grammar.ts"], ["build.ts", "schemas"]]


def editor_skip_reason() -> str | None:
    """Why the VS Code generated-file checks cannot run here (they need node 24), or None."""
    node = shutil.which("node")
    if node is None:
        return "node not found: skipping the VS Code grammar and schema checks (CI runs them)"
    version = subprocess.run([node, "--version"], capture_output=True, text=True).stdout.strip()
    try:
        major = int(version.lstrip("v").split(".")[0])
    except ValueError:
        major = 0
    if major < 24:
        # The scripts are TypeScript run by node itself, which needs type stripping.
        return f"node {version} cannot run the .ts scripts (needs 24): skipping the VS Code checks"
    return None


def cmd_editor(args) -> int:
    """Check that the VS Code extension's generated grammar and schemas are current (needs node)."""
    reason = editor_skip_reason()
    if reason:
        print(reason)
        return 0
    failed = False
    for script in EDITOR_SCRIPTS:
        failed |= bool(run(["node", f"scripts/{script[0]}", *script[1:], "--check"], cwd=ROOT / "editors" / "vscode"))
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
        env["JAILSP"] = str(cargo_binary("jailsp"))
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
    if not rebuild_jaifmt(bin_dir / ("jaic" + EXE)):
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
        # CI spreads the tests over several jobs with --partition (nextest's `slice:K/N`, which deals
        # the tests out one at a time across all test binaries) and narrows a job with --expr (a
        # nextest filterset, applied on top of the profile's default filter).
        part = ["--partition", args.partition] if args.partition else []
        filterset = ["-E", args.expr] if args.expr else []
        failed |= bool(run(["cargo", "nextest", "run", "--profile", "ci", "-j", args.jobs, "--locked", "--no-fail-fast",
                            *profile, *part, *filterset, *scope, *args.filter]))
        # nextest does not run doctests; the first partition runs them.
        if not args.partition or args.partition.split(":")[-1].startswith("1/"):
            failed |= bool(run(["cargo", "test", "--doc", "-j", args.jobs, "--locked", *scope]))
    if args.suite in ("tools", "all") and not narrowed:
        failed |= bool(run([sys.executable, "tools/run_tests.py"]))
    return int(failed)


def run_captured(cmd: list[str], cwd: Path = ROOT, stdin: str | None = None) -> tuple[int, str]:
    """Run `cmd`, returning its exit status and its output (stdout and stderr together)."""
    try:
        done = subprocess.run(cmd, cwd=cwd, input=stdin, capture_output=True, text=True)
    except OSError as error:
        return 127, f"{cmd[0]}: {error}\n"
    return done.returncode, done.stdout + done.stderr


def repo_check_failures(names: list[str], skip_missing: bool = False) -> dict[str, str]:
    """Run the named repository checks (all when empty); the output of each one that failed, by script."""
    failures = {}
    for name in names or list(REPO_CHECKS):
        script = f"tools/ci/{REPO_CHECKS[name]}.py"
        if skip_missing and not (ROOT / script).is_file():
            continue
        status, output = run_captured([sys.executable, script])
        if status:
            failures[script] = output
    return failures


def cmd_repo_checks(args) -> int:
    """The text checks CI runs on every push (and the pre-commit hook): file length and banners,
    repo paths named in docs and headers, `include_str!` inputs, and the undocumented public
    standard-library declarations (which builds `jaic`; the hook leaves it out). Every check runs."""
    failures = repo_check_failures(args.names)
    for script, output in failures.items():
        print(f"{script} failed:", file=sys.stderr)
        sys.stderr.write(output)
    return int(bool(failures))


def install_hooks() -> bool:
    """Point core.hooksPath at the relative `.githooks`, so every worktree runs its own tree's hooks.

    Set in the shared config; a worktree-level value (extensions.worktreeConfig) would override it,
    so that one is removed. Does nothing in CI or outside a git checkout."""
    if os.environ.get("CI") or not (ROOT / ".githooks").is_dir():
        return False
    if run_captured(["git", "rev-parse", "--git-dir"])[0]:
        return False
    current = run_captured(["git", "config", "--local", "--get", "core.hooksPath"])[1].strip()
    if run_captured(["git", "config", "--worktree", "--get", "core.hooksPath"])[0] == 0:
        run_captured(["git", "config", "--worktree", "--unset", "core.hooksPath"])
    if current != ".githooks":
        if run_captured(["git", "config", "--local", "core.hooksPath", ".githooks"])[0]:
            return False
        print(f"core.hooksPath set to .githooks (was {current or 'unset'})", file=sys.stderr)
    return True


def cmd_hooks(args) -> int:
    return 0 if install_hooks() else 1


class Hook:
    """State of one pre-commit run: the temp export of the staged files and the failures so far."""

    def __init__(self, tmp: Path):
        self.tmp = tmp
        self.failed = False
        self.fixes: list[str] = []
        self.skip = os.environ.get("JAI_HOOK_SKIP", "").split()
        self.verbose = bool(os.environ.get("JAI_HOOK_VERBOSE"))
        self.no_build = bool(os.environ.get("JAI_HOOK_NO_BUILD"))

    def skipped(self, name: str) -> bool:
        return name in self.skip

    def say(self, message: str) -> None:
        if self.verbose:
            print(f"pre-commit: {message}", file=sys.stderr)

    def fail(self, fix: str, output: str = "") -> None:
        if output:
            sys.stderr.write(output if output.endswith("\n") else output + "\n")
        self.failed = True
        self.fixes.append(fix)


def hook_warn(message: str) -> None:
    print(f"pre-commit: warning: {message}", file=sys.stderr)


def newest_jaic() -> Path | None:
    """The newer of the release and debug jaic builds: a stale one may not compile the current jaifmt."""
    found = [p for p in (cargo_binary("jaic", "release"), cargo_binary("jaic", "debug")) if p.is_file()]
    return max(found, key=lambda p: p.stat().st_mtime, default=None)


def hook_jaic(hook: Hook) -> Path | None:
    """The debug jaic cargo builds from this checkout (a no-op when current), else the newest existing one."""
    if hook.no_build:
        return None
    if shutil.which("cargo"):
        hook.say("building jaic")
        if run_captured(["cargo", "build", "--quiet", "-p", "jaic-cli"])[0] == 0:
            built = cargo_binary("jaic", "debug")
            return built if built.is_file() else newest_jaic()
        hook_warn("could not build jaic; using an existing one, which may not be this checkout's")
    return newest_jaic()


def hook_jaifmt(hook: Hook) -> str | None:
    """target/jaifmt when built from this checkout (rebuilt when stale), else jaifmt on PATH."""
    out = jaifmt_path()
    jaic = hook_jaic(hook)
    if jaic is not None and stale_jaifmt(out, jaic):
        hook.say("rebuilding jaifmt")
        if not rebuild_jaifmt(jaic, quiet=True):
            hook_warn("could not rebuild jaifmt; using the existing one")
    if out.is_file() and built_elsewhere(out):
        hook_warn(f"{out} was built from another checkout and could not be rebuilt here; using jaifmt on PATH")
        return shutil.which("jaifmt")
    return str(out) if out.is_file() else shutil.which("jaifmt")


def hook_jailint(hook: Hook) -> str | None:
    """The debug jailint cargo builds from this checkout, else an existing one newer than its sources, else any
    existing one, else PATH. Cargo links this checkout's build into the target directory even when nothing
    changed, so a shared directory never keeps another checkout's binary under the name."""
    if not hook.no_build and shutil.which("cargo"):
        hook.say("building jailint")
        if run_captured(["cargo", "build", "--quiet", "-p", "jailint"])[0] == 0:
            return str(cargo_binary("jailint", "debug"))
        hook_warn("could not build jailint; using an existing one, which may not be this checkout's")
    binaries = [p for p in (cargo_binary("jailint", "release"), cargo_binary("jailint", "debug")) if p.is_file()]
    fresh = [p for p in binaries if not stale(p, *JAILINT_SOURCES)]
    found = (fresh or binaries or [None])[0]
    return str(found) if found else shutil.which("jailint")


def check_jai(hook: Hook, jai: list[str]) -> None:
    if not hook.skipped("jaifmt"):
        jaifmt = hook_jaifmt(hook)
        if jaifmt is None:
            hook_warn("jaifmt not found (build jaifmt into the cargo target directory, see "
                      "docs/tools/development.md#pre-commit-hook); skipping the Jai format check")
        else:
            hook.say("jaifmt --check")
            # The export of the staged blobs, as it will be committed.
            status, output = run_captured([jaifmt, "--check", *jai], cwd=hook.tmp)
            if status:
                hook.fail("jaifmt <files>", output)
    if not hook.skipped("jailint"):
        jailint = hook_jailint(hook)
        if jailint is None:
            hook_warn("jailint not found (cargo build -p jailint, see "
                      "docs/tools/development.md#pre-commit-hook); skipping the Jai lint check")
            return
        # Linting needs the whole program around a file, so it reads the working tree.
        _, changed = run_captured(["git", "diff", "--name-only", "--", *jai])
        if changed.split():
            hook_warn("partially staged, linted as in the working tree: " + " ".join(changed.split()))
        # A platform's modules (Windows, Linux, X11, ...) cannot compile on another host, as in `just lint`.
        hook.say("jailint -D warnings")
        status, output = run_captured([jailint, "-D", "warnings", "--allow-incomplete", *jai])
        if status:
            hook.fail("jailint --fix <files>   (then fix the rest by hand)", output)


def check_rust(hook: Hook, rust: list[str]) -> None:
    if not hook.skipped("rustfmt"):
        if not shutil.which("rustfmt"):
            hook_warn("rustfmt not found; skipping the Rust format check")
        else:
            bad = False
            for f in rust:
                # Stdin mode: no module following, and the staged text is what is read. It reports a
                # diff but still exits 0, so any output counts as a failure.
                _, blob = run_captured(["git", "show", f":{f}"])
                status, output = run_captured(["rustfmt", "--check", "--edition", "2024"], stdin=blob)
                if status or output.strip():
                    print(f"rustfmt: {f} would change", file=sys.stderr)
                    sys.stderr.write(output.replace("<stdin>", f))
                    bad = True
            if bad:
                hook.failed = True
                hook.fixes.append("cargo fmt --all")
    if not hook.skipped("spacing"):
        spacing = ROOT / "tools" / "dev" / "rust_item_spacing.py"
        if not spacing.is_file():
            hook_warn("tools/dev/rust_item_spacing.py not found; skipping the Rust item spacing check")
        else:
            status, output = run_captured([sys.executable, str(spacing), "--check", *rust], cwd=hook.tmp)
            if status:
                hook.fail("python3 tools/dev/rust_item_spacing.py <files>", output.replace(f"{hook.tmp}/", ""))


def check_shell(hook: Hook, shell: list[str]) -> None:
    if hook.skipped("shellcheck"):
        return
    if not shutil.which("shellcheck"):
        hook_warn("shellcheck not found; skipping the shell lint check")
        return
    hook.say("shellcheck")
    # The staged copies, so the findings are about what will be committed.
    status, output = run_captured(["shellcheck", *shell], cwd=hook.tmp)
    if status:
        hook.fail("shellcheck <files>", output)


def check_locks(hook: Hook) -> None:
    """A manifest change must come with its lock files (CI builds with --locked)."""
    for manifest in ("Cargo.toml", "fuzz/Cargo.toml"):
        if not (ROOT / manifest).is_file():
            continue
        hook.say(f"checking the lock file of {manifest}")
        status, output = run_captured(["cargo", "metadata", "--locked", "--offline", "--format-version", "1",
                                       "--manifest-path", str(ROOT / manifest)])
        if status and "cannot update the lock file" in output:
            print(f"pre-commit: {os.path.dirname(manifest) or '.'}/Cargo.lock is out of date", file=sys.stderr)
            hook.failed = True
            hook.fixes.append(f"cargo metadata --format-version 1 --manifest-path {manifest} >/dev/null")


def check_editor(hook: Hook) -> None:
    """The VS Code grammar and schemas are generated from the compiler and config sources."""
    if editor_skip_reason():
        hook_warn("node 24 not found; skipping the VS Code generated-file check")
        return
    for script in EDITOR_SCRIPTS:
        hook.say(f"running {' '.join(script)} --check")
        status, output = run_captured(["node", f"scripts/{script[0]}", *script[1:], "--check"],
                                      cwd=ROOT / "editors" / "vscode")
        if status:
            hook.fail(f"(cd editors/vscode && node scripts/{' '.join(script)})", output)


def cmd_precommit(args) -> int:
    """Format and lint the staged files only (the git pre-commit hook; docs/tools/development.md#pre-commit-hook)."""
    listed = run_captured(["git", "-c", "core.quotepath=off", "diff", "--cached", "--name-only", "--diff-filter=ACMR"])
    everything = tracked([f for f in listed[1].split("\n") if f]) if listed[0] == 0 else []
    if not everything:
        return 0
    jai, rust, shell = staged("jai"), staged("rust"), staged("shell")
    with tempfile.TemporaryDirectory(prefix="jai-pre-commit.") as tmp:
        hook = Hook(Path(tmp))
        # Check what will be committed, not what is in the working tree: export the staged blobs.
        exported = "".join(f + "\0" for f in (*jai, *rust, *shell))
        done = subprocess.run(["git", "checkout-index", f"--prefix={tmp}/", "-z", "--stdin"], cwd=ROOT,
                              input=exported, text=True, capture_output=True)
        if done.returncode:
            hook_warn("could not export the staged files; skipping the checks")
            return 0
        for cfg in ("jacket.toml", "rustfmt.toml", "rust-toolchain.toml"):
            status, text = run_captured(["git", "show", f":{cfg}"])
            if status == 0:
                (hook.tmp / cfg).write_text(text)
        if jai:
            check_jai(hook, jai)
        if rust:
            check_rust(hook, rust)
        if shell:
            check_shell(hook, shell)
        # They read the working tree, so a partly staged file is checked as it is on disk.
        if not hook.skipped("repo"):
            hook.say("repo checks")
            for script, output in repo_check_failures([n for n in REPO_CHECKS if n not in BUILT_REPO_CHECKS], skip_missing=True).items():
                hook.fail(f"python3 {script}", output)
        staged_text = "\n".join(everything)
        if (not hook.skipped("locks") and shutil.which("cargo")
                and re.search(r"Cargo\.(toml|lock)$", staged_text, re.M)):
            check_locks(hook)
        if (not hook.skipped("editor") and (ROOT / "editors" / "vscode" / "scripts").is_dir()
                and re.search(r"^(crates/|editors/vscode/)", staged_text, re.M)):
            check_editor(hook)
        if hook.failed:
            print("\npre-commit: checks failed. Fix, re-stage (git add) and commit again:" +
                  "".join(f"\n  {fix}" for fix in hook.fixes), file=sys.stderr)
            print("pre-commit: or skip the hook once with: git commit --no-verify", file=sys.stderr)
            return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    for name, fn in (("build", cmd_build), ("fmt", cmd_fmt), ("lint", cmd_lint), ("editor", cmd_editor),
                     ("nvim", cmd_nvim), ("vscode-try", cmd_vscode_try), ("test", cmd_test),
                     ("repo-checks", cmd_repo_checks), ("precommit", cmd_precommit), ("hooks", cmd_hooks)):
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
        if name == "repo-checks":
            p.add_argument("names", nargs="*", choices=list(REPO_CHECKS), help="default: all")
        if name == "precommit":
            p.add_argument("hook_args", nargs="*", help="what git passes the hook (unused)")
        if name == "nvim":
            p.add_argument("--unit", action="store_true")
        if name == "vscode-try":
            p.add_argument("--debug", action="store_true")
            p.add_argument("folder", nargs="?", default="")
        if name == "test":
            p.add_argument("--suite", choices=["cargo", "tools", "all"], default="all")
            p.add_argument("-p", "--crate", default="")
            p.add_argument("--partition", default="", help="run one part of the tests, e.g. slice:1/4")
            p.add_argument("--expr", default="", help="a nextest filterset narrowing the tests")
            p.add_argument("filter", nargs="*")
    args = parser.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
