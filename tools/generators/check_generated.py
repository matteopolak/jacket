#!/usr/bin/env python3
"""Run every generator of checked-in files with --check: each must reproduce its output exactly.

    python3 tools/generators/check_generated.py [--formatter <jaifmt>] [--offline]

Generators declare themselves with module-level constants, read from the source without importing it:

    OUTPUTS = ["stdlib/internal/basic/powers-of-five.jai"]   # required: the repo-relative files it writes
    FORMATTED = True    # it formats its output with the repository jaifmt (takes --formatter)
    DOWNLOADS = True    # it downloads its pinned, hash-checked input (skipped by --offline)

Any script under tools/ that defines `OUTPUTS` is run; a script in tools/generators/ with a `__main__`
guard (other than this one and the tests) is a generator and must define a non-empty one. The SDL3 and
native bindings generators are checked by their own CI steps, since they need fetched sources or their
own working directory.
"""

import argparse
import ast
from dataclasses import dataclass
from pathlib import Path
import subprocess
import sys
from toolkit.repo import ROOT, jaifmt_path

SKIPPED_FOLDERS = {"__pycache__", "node_modules", "target"}


@dataclass(frozen=True)
class Generator:
    script: str
    outputs: tuple
    formatted: bool = False
    downloads: bool = False


def module_constants(source):
    """The literal module-level assignments of `source`, by name."""
    found = {}
    for node in ast.parse(source).body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                found[node.targets[0].id] = ast.literal_eval(node.value)
            except ValueError:
                found[node.targets[0].id] = NotImplemented
    return found


def has_main_guard(source):
    return "__name__" in source and "__main__" in source


def read_script(root, path, in_generators_folder):
    """(Generator or None, problems) for one script."""
    rel = path.relative_to(root).as_posix()
    source = path.read_text(encoding="utf-8")
    constants = module_constants(source) if "OUTPUTS" in source else {}
    needs = in_generators_folder and has_main_guard(source)
    if "OUTPUTS" not in constants and not needs:
        return None, []
    outputs = constants.get("OUTPUTS")
    if not isinstance(outputs, list) or not outputs:
        return None, [f"{rel} declares no OUTPUTS (a non-empty module-level list of repo-relative paths)"]
    if not all(isinstance(item, str) for item in outputs):
        return None, [f"{rel}: OUTPUTS must be a list of string literals"]
    problems = [f"{rel}: declared output {item} does not exist" for item in outputs if not (root / item).exists()]
    flags = {name: constants.get(name, False) for name in ("FORMATTED", "DOWNLOADS")}
    for name, value in flags.items():
        if not isinstance(value, bool):
            problems.append(f"{rel}: {name} must be True or False")
    return Generator(rel, tuple(outputs), flags["FORMATTED"] is True, flags["DOWNLOADS"] is True), problems


def discover(root=ROOT):
    """Every generator under `root`/tools and the problems found in their declarations."""
    tools = root / "tools"
    generators, problems, owners = [], [], {}
    for path in sorted(tools.rglob("*.py")):
        relative = path.relative_to(tools)
        if SKIPPED_FOLDERS & set(relative.parts) or path.name.startswith("test_") or path == Path(__file__).resolve():
            continue
        generator, found = read_script(root, path, relative.parts[:1] == ("generators",))
        problems += found
        if generator is None:
            continue
        generators.append(generator)
        for output in generator.outputs:
            if owners.setdefault(output, generator.script) != generator.script:
                problems.append(f"{output} is declared by both {owners[output]} and {generator.script}")
    return generators, problems


def commands(generators, formatter, offline):
    """The `--check` command line of each generator that runs."""
    lines = []
    for generator in generators:
        if offline and generator.downloads:
            continue
        line = [generator.script]
        if generator.formatted:
            line += ["--formatter", formatter]
        lines.append(line)
    return lines


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--formatter", type=Path, default=jaifmt_path(),
                        help="repository jaifmt built from source (default: <cargo target>/jaifmt)")
    parser.add_argument("--offline", action="store_true", help="skip the generators that download their input")
    args = parser.parse_args()
    generators, problems = discover()
    if problems:
        print("generator declarations:\n  " + "\n  ".join(problems), file=sys.stderr)
        return 1
    formatter = str(args.formatter.resolve(strict=True)) if any(g.formatted for g in generators) else ""
    runs = commands(generators, formatter, args.offline)
    failed = []
    for script, *options in runs:
        result = subprocess.run([sys.executable, script, "--check", *options], cwd=ROOT)
        if result.returncode != 0:
            failed.append(script)
    if failed:
        print("stale generated files; rerun without --check: " + ", ".join(failed), file=sys.stderr)
        return 1
    print(f"all {len(runs)} generators reproduce their checked-in files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
