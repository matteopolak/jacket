#!/usr/bin/env python3
"""Fail when a tools/**/*.py cannot resolve its imports the way CI and the test runner load it.

Each file is loaded twice, in a fresh interpreter started from the repository root:

  script  `python3 tools/<folder>/<file>.py`: only the file's own folder is on sys.path, so every
          cross-folder import must put tools/ on the path itself (see docs/tools/development.md).
  runner  `tools/run_tests.py`: the file's folder (unittest discover's start directory) and tools/ (PYTHONPATH).

A load executes the module's top-level statements up to its last top-level import, nothing after, so a
script that does work at module level is not run. Imports nested in functions are checked statically against
the same two search paths. A missing module that is not part of tools/ is an optional third-party dependency
and is skipped. Other exceptions raised while the imports run are not import errors and are ignored.

    python3 tools/ci/check_tool_imports.py                          # every file, both loads
    python3 tools/ci/check_tool_imports.py tools/toolkit/download.py
"""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))


import argparse
import ast
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from toolkit.repo import ROOT

TOOLS = ROOT / "tools"
MISSING = 3  # the child's exit status for an import error; its stdout is "MISSING <module>" or "FAILED <detail>"

# Runs in the child interpreter. argv: <file> <mode> <tools dir>.
LOADER = r"""
import ast, sys
from pathlib import Path

path = Path(sys.argv[1]).resolve()
sys.path[0:1] = [str(path.parent)] if sys.argv[2] == "script" else [str(path.parent), sys.argv[3]]
tree = ast.parse(path.read_text(encoding="utf-8"), str(path))


def imports_at_module_level(node):
    if isinstance(node, (ast.Import, ast.ImportFrom)):
        return True
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
        return False
    return any(imports_at_module_level(child) for child in ast.iter_child_nodes(node))


last = max((i for i, node in enumerate(tree.body) if imports_at_module_level(node)), default=-1)
prefix = ast.Module(body=tree.body[: last + 1], type_ignores=[])
namespace = {"__name__": "tool_import_check", "__file__": str(path), "__builtins__": __builtins__}
try:
    exec(compile(prefix, str(path), "exec"), namespace)
except ModuleNotFoundError as error:
    print("MISSING " + str(error.name))
    sys.exit(3)
except ImportError as error:
    print("FAILED " + str(error))
    sys.exit(3)
except BaseException:
    pass  # not an import problem: a guard or a module-level action between the imports
"""


def tool_files(selected: list[str]) -> list[Path]:
    if selected:
        return [(ROOT / name).resolve() for name in selected]
    return sorted(p for p in TOOLS.rglob("*.py") if "__pycache__" not in p.parts)


def folders() -> set[str]:
    return {p.name for p in TOOLS.iterdir() if p.is_dir() and not p.name.startswith(("_", "."))}


def top_level_names() -> set[str]:
    return {p.stem for p in TOOLS.glob("*.py")}


def project_names() -> set[str]:
    """Module and folder names under tools/: a missing import with one of these names is a bug, not an optional dependency."""
    return {p.stem for p in TOOLS.rglob("*.py")} | folders()


def load(path: Path, mode: str) -> tuple[str, str]:
    """Returns (status, detail): status is ok, skip or fail."""
    env = {k: v for k, v in os.environ.items() if k != "PYTHONPATH"}
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    if mode == "runner":
        env["PYTHONPATH"] = str(TOOLS)
    command = [sys.executable, "-c", LOADER, str(path), mode, str(TOOLS)]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True, timeout=120, stdin=subprocess.DEVNULL)
    out = result.stdout.strip()
    if result.returncode == 0:
        return "ok", ""
    if result.returncode == MISSING and out.startswith("MISSING "):
        missing = out.split(None, 1)[1]
        if missing.split(".")[0] not in project_names():
            return "skip", f"optional dependency {missing}"
        return "fail", f"No module named {missing!r}"
    if result.returncode == MISSING:
        return "fail", out
    return "fail", "\n    ".join((out + "\n" + result.stderr).strip().splitlines()[-6:])


def nested_import_problems(path: Path) -> list[str]:
    """Static check of every import (including ones in functions) against the script/runner search path."""
    tree = ast.parse(path.read_text(encoding="utf-8"), str(path))
    own = {p.stem for p in path.parent.glob("*.py")} | {p.name for p in path.parent.iterdir() if p.is_dir()}
    visible = own | top_level_names() | folders()
    project = project_names() - set(sys.stdlib_module_names)
    problems = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            targets = [(a.name, []) for a in node.names]
        elif isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
            targets = [(node.module, [a.name for a in node.names])]
        else:
            continue
        for name, members in targets:
            head, _, rest = name.partition(".")
            if head not in project:
                continue
            if head not in visible:
                problems.append(f"line {node.lineno}: `{name}` is not importable from {path.parent.name}/ (qualify it by folder)")
            elif head in folders() and head not in own:
                folder = TOOLS / head
                wanted = [rest.split(".")[0]] if rest else members
                for module in wanted:
                    if module != "*" and not (folder / f"{module}.py").exists() and not (folder / module).is_dir():
                        problems.append(f"line {node.lineno}: tools/{head}/ has no module `{module}`")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("files", nargs="*", help="files to check (default: every tools/**/*.py)")
    parser.add_argument("-j", "--jobs", type=int, default=4)
    args = parser.parse_args()
    files = tool_files(args.files)
    jobs = [(path, mode) for path in files for mode in ("script", "runner")]
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        results = list(pool.map(lambda job: load(*job), jobs))
    failed = skipped = 0
    for (path, mode), (status, detail) in zip(jobs, results):
        name = path.relative_to(ROOT).as_posix()
        if status == "fail":
            failed += 1
            print(f"FAIL {mode:6} {name}\n    {detail}")
        elif status == "skip":
            skipped += 1
            print(f"skip {mode:6} {name}: {detail}")
    for path in files:
        for problem in nested_import_problems(path):
            failed += 1
            print(f"FAIL static {path.relative_to(ROOT).as_posix()}\n    {problem}")
    print(f"check_tool_imports: {len(files)} files, {len(jobs) - skipped} loads, {skipped} skipped, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
