#!/usr/bin/env python3
"""Fail when a tracked Rust file is too long or a Rust or Python file carries a banner comment.

Length: a `.rs` file may have at most MAX_LINES lines. Files already over the limit are listed in
ALLOWED_LONG with their current line count. The list only shrinks: a listed file may not grow past
its recorded count, and a listed file that is back under the limit (or gone) must be removed from
the list. Never add a new entry; split the file instead.

Banners: a comment made only of rule characters (`// ----`, `# ====`) is rejected. Put the section
name on a single plain comment line, or split the file. Jai files are covered by the jailint
`banner_comment` rule.

Build paths: a Python script under tools/ may not read `CARGO_TARGET_DIR` from an environment or build
`ROOT / "target"...` by hand. Use `toolkit.repo.cargo_target_dir()` / `cargo_binary()` for cargo's output
and `repo_output.output_dir()` for scratch. Setting the variable for a child process is fine.
`BUILD_PATH_ALLOWED` lists the few files that really resolve these paths themselves.

Repo root: a Python script under tools/ may not work the root out from `__file__` (`Path(__file__).resolve().parents[2]`).
Import `toolkit.repo.ROOT`. `ROOT_ALLOWED` lists the files that run without `tools/` on the import path.
"""

import ast
import re
import sys

from toolkit.repo import ROOT, git_files

MAX_LINES = 1200

# path -> line count it may not grow past. Remove an entry once the file is split.
ALLOWED_LONG = {}

# Files that may read CARGO_TARGET_DIR or build ROOT / "target" themselves, and why.
BUILD_PATH_ALLOWED = {
    "tools/toolkit/repo.py": "the one resolver",
    "tools/repo_output.py": "scratch output directory (JACKET_OUTPUT_DIR, shared across worktrees), not cargo's",
    "tools/browser/cargo_build_paths.py": "release packaging resolver: pinned toolchain, locked metadata and an injected environment",
    "tools/browser/test_build_scripting_wasm.py": "asserts on the environment handed to the cargo child process",
    "tools/dev/test_build_pgo.py": "asserts on the environment handed to the cargo child process",
    "tools/stdlib/test_io_failure_wiring.py": "pins CARGO_TARGET_DIR for the code under test so cargo metadata never runs",
    "tools/ci/ci_test_scratch.py": "scratch selection from an injected environment, deliberately without cargo or defaults",
}

# Files that may find the repository root from their own location, and why.
ROOT_ALLOWED = {
    "tools/toolkit/repo.py": "the one resolver",
    "tools/ci/job_inputs.py": "runs from a checkout of any layout, with no import path of its own",
    "tools/ci/check_ci_sources.py": "packaging runs it on a public checkout without tools/ on the import path",
    "tools/dev/check_rust_format.py": "a git hook, run without PYTHONPATH",
    "tools/dev/pre_push_clippy.py": "a git hook, run without PYTHONPATH",
}
TARGET_ENV = "CARGO_TARGET_DIR"

BANNER = re.compile(r"^\s*(//|#)\s*[-=*#~]{3,}\s*$")


def line_count(text):
    return text.count("\n") + (0 if text == "" or text.endswith("\n") else 1)


def check_lengths(counts, allowed=None, limit=MAX_LINES):
    """Errors for `counts` (path -> lines) against the `allowed` over-limit list."""
    allowed = ALLOWED_LONG if allowed is None else allowed
    errors = []
    for path, n in sorted(counts.items()):
        cap = allowed.get(path)
        if cap is None:
            if n > limit:
                errors.append(
                    f"{path}: {n} lines, over the {limit}-line limit; split it into modules"
                )
        elif n > cap:
            errors.append(
                f"{path}: grew to {n} lines, past its allowlisted {cap}; split it (or move part of it into a module) instead of growing it"
            )
        elif n <= limit:
            errors.append(
                f"{path}: now {n} lines; remove it from ALLOWED_LONG in check_source_style.py"
            )
        elif n < cap:
            errors.append(
                f"{path}: shrank to {n} lines; lower its ALLOWED_LONG entry from {cap}"
            )
    for path in sorted(allowed):
        if path not in counts:
            errors.append(
                f"{path}: gone; remove it from ALLOWED_LONG in check_source_style.py"
            )
    return errors


def check_banners(name, text):
    return [
        f"{name}:{i}: banner comment; use one plain comment line or split the file"
        for i, line in enumerate(text.splitlines(), 1)
        if BANNER.match(line)
    ]


def _is_str(node: ast.AST, test) -> bool:
    return isinstance(node, ast.Constant) and isinstance(node.value, str) and test(node.value)


def _is_root(node: ast.AST) -> bool:
    return (isinstance(node, ast.Name) and node.id == "ROOT") or (isinstance(node, ast.Attribute) and node.attr == "ROOT")


def _is_target_part(node: ast.AST) -> bool:
    is_target = lambda text: text == "target" or text.startswith("target/")  # noqa: E731
    if isinstance(node, ast.JoinedStr) and node.values:
        return _is_str(node.values[0], lambda text: text.startswith("target/"))
    return _is_str(node, is_target)


def build_path_problems(source: str, name: str = "") -> list[str]:
    """Reads of CARGO_TARGET_DIR and hand-built `ROOT / "target"` paths in `source` (a file's text)."""
    if name in BUILD_PATH_ALLOWED:
        return []
    problems = []
    for node in ast.walk(ast.parse(source)):
        reads_env = (
            # env.get("CARGO_TARGET_DIR"), os.getenv(...)
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr in ("get", "getenv")
            and any(_is_str(arg, TARGET_ENV.__eq__) for arg in node.args[:1])
            # env["CARGO_TARGET_DIR"] when read
            or isinstance(node, ast.Subscript)
            and isinstance(node.ctx, ast.Load)
            and _is_str(node.slice, TARGET_ENV.__eq__)
            # "CARGO_TARGET_DIR" in env
            or isinstance(node, ast.Compare)
            and _is_str(node.left, TARGET_ENV.__eq__)
            and isinstance(node.ops[0], (ast.In, ast.NotIn))
        )
        if reads_env:
            problems.append(f"line {node.lineno}: reads {TARGET_ENV}; use toolkit.repo.cargo_target_dir() or cargo_binary()")
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div) and _is_root(node.left) and _is_target_part(node.right):
            problems.append(
                'line %d: builds ROOT / "target" by hand; use toolkit.repo.cargo_target_dir()/cargo_binary() or repo_output.output_dir()'
                % node.lineno
            )
    return problems


def root_problems(source: str, name: str = "") -> list[str]:
    """Lines of `source` that find the repository root from `__file__`."""
    if name in ROOT_ALLOWED:
        return []
    return [
        f"line {node.lineno}: finds the repository root from __file__; use toolkit.repo.ROOT"
        for node in ast.walk(ast.parse(source))
        if isinstance(node, ast.Attribute)
        and node.attr == "parents"
        and any(isinstance(inner, ast.Name) and inner.id == "__file__" for inner in ast.walk(node.value))
    ]


def main():
    counts, errors = {}, []
    for path in git_files("*.rs"):
        if (ROOT / path).is_file():
            counts[path] = line_count(
                (ROOT / path).read_text(encoding="utf-8", errors="replace")
            )
    errors += check_lengths(counts)
    for path in git_files("*.rs", "*.py"):
        if (ROOT / path).is_file():
            errors += check_banners(
                path, (ROOT / path).read_text(encoding="utf-8", errors="replace")
            )
    for path in git_files("tools/*.py"):
        if (ROOT / path).is_file():
            text = (ROOT / path).read_text(encoding="utf-8")
            for problem in [*build_path_problems(text, path), *root_problems(text, path)]:
                errors.append(f"{path}: {problem}")
    for e in errors:
        print(e, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
