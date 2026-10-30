#!/usr/bin/env python3
"""Write the reference pages under docs/reference/ from the tools' own definitions.

    python3 tools/generators/generate_reference_docs.py [--check]

- `environment-variables.md`: the table in `crates/jacket-config/src/env.rs` (`cargo run --example env_reference`).
- `lint-rules.md`: the rule table of `crates/jailint/src/rules/` (`cargo run --example lint_reference`).
- `command-line.md`: the `--help` of `jaic`, `jailint` and `jacket` and each `jacket` command that takes its own flags.

The hand-written pages (docs/tools/*.md) keep the reasons and link here, so a variable, rule or flag is
described once, where the code defines it. The script builds what it runs with cargo; it needs no jaifmt.
"""

import argparse
import re
import subprocess
import sys
from toolkit.repo import ROOT, cargo_binary

OUTPUTS = [
    "docs/reference/environment-variables.md",
    "docs/reference/lint-rules.md",
    "docs/reference/command-line.md",
]

# `jacket` commands whose arguments belong to a tool: their `--help` is that tool's.
PASSTHROUGH = {"build", "check", "run", "lint", "fmt", "lsp", "jaic", "jailint", "jailsp", "jaifmt"}


def cargo(*args):
    subprocess.run(["cargo", *args], cwd=ROOT, check=True)


def example(package, name):
    """The stdout of `cargo run --example name` in `package`."""
    result = subprocess.run(["cargo", "run", "--locked", "-q", "-p", package, "--example", name],
                            cwd=ROOT, check=True, capture_output=True, text=True)
    return result.stdout


def help_text(program, *args):
    """The help `program` prints, trailing blanks stripped."""
    result = subprocess.run([str(cargo_binary(program)), *args], cwd=ROOT, check=True, capture_output=True, text=True)
    return "\n".join(line.rstrip() for line in result.stdout.strip().splitlines())


def jacket_commands(top_help):
    """The `jacket` commands listed under `Commands:` in its help."""
    lines = top_help.splitlines()
    start = lines.index("Commands:") + 1
    names = []
    for line in lines[start:]:
        if not line.startswith("  "):
            break
        names.append(line.split()[0])
    return names


def block(text):
    return f"```text\n{text}\n```\n"


def command_line():
    cargo("build", "--locked", "-q", "-p", "jaic-cli", "-p", "jacket", "-p", "jailint")
    jaic = re.sub(r"^jaic \d[^\s:]*:", "jaic:", help_text("jaic", "-help"))
    jacket = help_text("jacket", "--help")
    parts = [
        "# Command line\n",
        "Generated from the tools' own help by `tools/generators/generate_reference_docs.py`; "
        "the sources are the option tables in `crates/jaic-cli/src/`, `crates/jailint/src/cli.rs` and "
        "`crates/jacket/src/cli.rs`. [The `jaic` command line](../tools/cli.md), [jailint](../tools/jailint.md) "
        "and [jacket](../tools/jacket.md) say how the flags fit together.\n",
        "## jaic\n", block(jaic),
        "## jailint\n", block(help_text("jailint", "--help")),
        "## jacket\n", block(jacket),
    ]
    for name in jacket_commands(jacket):
        if name in PASSTHROUGH or name == "help":
            continue
        parts += [f"### jacket {name}\n", block(help_text("jacket", name, "--help"))]
    return "\n".join(parts)


def render():
    """Each output path with the text it must hold."""
    return {
        OUTPUTS[0]: example("jacket-config", "env_reference"),
        OUTPUTS[1]: example("jailint", "lint_reference"),
        OUTPUTS[2]: command_line(),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true", help="fail if a checked-in file differs instead of writing it")
    args = parser.parse_args()
    stale = []
    for rel, text in render().items():
        path = ROOT / rel
        if args.check:
            if not path.is_file() or path.read_text() != text:
                stale.append(rel)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    if stale:
        print("stale generated files: " + ", ".join(stale), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
