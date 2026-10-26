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
"""

import ast
import re
import subprocess
import sys

from toolkit.repo import ROOT

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
    "tools/ci/ci_test_scratch.py": "scratch selection from an injected environment, deliberately without cargo or defaults",
}
TARGET_ENV = "CARGO_TARGET_DIR"

BANNER = re.compile(r"^\s*(//|#)\s*[-=*#~]{3,}\s*$")


def tracked(patterns):
    out = subprocess.check_output(["git", "ls-files", "-z", "--", *patterns], cwd=ROOT)
    return [p.decode() for p in out.split(b"\0") if p]


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


def main():
    counts, errors = {}, []
    for path in tracked(["*.rs"]):
        if (ROOT / path).is_file():
            counts[path] = line_count(
                (ROOT / path).read_text(encoding="utf-8", errors="replace")
            )
    errors += check_lengths(counts)
    for path in tracked(["*.rs", "*.py"]):
        if (ROOT / path).is_file():
            errors += check_banners(
                path, (ROOT / path).read_text(encoding="utf-8", errors="replace")
            )
    for path in tracked(["tools/*.py"]):
        if (ROOT / path).is_file():
            for problem in build_path_problems(
                (ROOT / path).read_text(encoding="utf-8"), path
            ):
                errors.append(f"{path}: {problem}")
    for e in errors:
        print(e, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
