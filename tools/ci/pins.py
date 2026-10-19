#!/usr/bin/env python3
"""Keep the tool versions that the workflows spell out equal to .github/pins.json.

GitHub Actions cannot read a file for `with:` inputs or job-level `env:`, so each pin appears as a
literal in the workflows. pins.json is the one place to change; `sync` rewrites the literals and `check`
(run by the tools tests) fails when one differs or a rule matches nothing. The Rust nightly is not
here: rust-toolchain.toml is its source, and tools/ci/install_rust_toolchain.sh reads it.

    python3 tools/ci/pins.py check
    python3 tools/ci/pins.py sync
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from toolkit.repo import ROOT

PINS = ROOT / ".github" / "pins.json"
WORKFLOWS = ".github/workflows/*.yml"
ACTIONS = ".github/actions/*/action.yml"

# pin -> [(glob, regex whose group 1 is the pinned text)]
RULES = {
    "node": [(WORKFLOWS, r"node-version: '(\d+)'"), (ACTIONS, r"node-version: '(\d+)'")],
    "python": [(WORKFLOWS, r"python-version: '([\d.]+)'"), (ACTIONS, r"python-version: '([\d.]+)'")],
    "just": [(WORKFLOWS, r"just-version: '([\d.]+)'")],
    "cargo_fuzz": [(WORKFLOWS, r"tool: cargo-fuzz@([\d.]+)")],
    "ruff": [(WORKFLOWS, r"ruff==([\d.]+)")],
    "llvm": [
        (WORKFLOWS, r"LLVM_VERSION: ([\d.]+)"),
        (WORKFLOWS, r"llvm: (?:LLVM|clang\+llvm)-([\d.]+)-"),
        ("tools/ci/install_ci_llvm_macos.sh", r"llvm_version=([\d.]+)"),
    ],
    "llvm_mingw": [(WORKFLOWS, r"LLVM_MINGW_VERSION: '(\d+)'")],
    "mesa": [(WORKFLOWS, r"MESA_VERSION: ([\d.]+)")],
    "mesa_sha256_x64": [(WORKFLOWS, r"MESA_SHA256_X64: ([0-9a-f]{64})")],
    "mesa_sha256_arm64": [(WORKFLOWS, r"MESA_SHA256_ARM64: ([0-9a-f]{64})")],
    "nvim": [(WORKFLOWS, r"NVIM_VERSION: (v[\d.]+)")],
    "nvim_sha256": [(WORKFLOWS, r"NVIM_SHA256: ([0-9a-f]{64})")],
}

# pin -> the script that reads it from pins.json itself (so no literal is spelled out elsewhere).
READ_BY = {
    "nextest": "tools/ci/install_nextest.sh",
    "nextest_sha256_linux_x64": "tools/ci/install_nextest.sh",
    "nextest_sha256_linux_arm64": "tools/ci/install_nextest.sh",
    "nextest_sha256_macos": "tools/ci/install_nextest.sh",
    "mini_test_commit": "tools/dev/fetch_mini_test.py",
}


def load(path=PINS):
    return json.loads(Path(path).read_text())


def rewrite(text, regex, value):
    """(new text, number of matches, matches that differed)."""
    differing = 0

    def replace(match):
        nonlocal differing
        if match.group(1) != value:
            differing += 1
        return match.group(0)[: match.start(1) - match.start(0)] + value + match.group(0)[match.end(1) - match.start(0) :]

    new, count = re.subn(regex, replace, text)
    return new, count, differing


def run(root, pins, write):
    """Problems found; with `write`, the differing literals are rewritten instead of reported."""
    problems = []
    for pin, rules in RULES.items():
        if pin not in pins:
            problems.append(f"pins.json has no `{pin}`")
            continue
        found = 0
        for glob, regex in rules:
            for path in sorted(root.glob(glob)):
                text = path.read_text()
                new, count, differing = rewrite(text, regex, pins[pin])
                found += count
                if differing and write:
                    path.write_text(new)
                elif differing:
                    problems.append(f"{path.relative_to(root)}: `{pin}` differs from pins.json ({pins[pin]}); run tools/ci/pins.py sync")
        if not found:
            problems.append(f"no workflow mentions `{pin}`: drop it from pins.json or fix its rule in tools/ci/pins.py")
    for pin, script in READ_BY.items():
        if pin not in pins:
            problems.append(f"pins.json has no `{pin}`")
        elif not (root / script).is_file() or pin not in (root / script).read_text():
            problems.append(f"{script} does not read `{pin}`: fix READ_BY in tools/ci/pins.py")
    for pin in pins:
        if pin not in RULES and pin not in READ_BY:
            problems.append(f"pins.json `{pin}` has no rule in tools/ci/pins.py")
    return problems


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in ("check", "sync"):
        print(__doc__, file=sys.stderr)
        return 2
    problems = run(ROOT, load(), write=sys.argv[1] == "sync")
    for problem in problems:
        print(problem, file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
