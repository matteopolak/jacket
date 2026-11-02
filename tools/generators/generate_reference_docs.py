#!/usr/bin/env python3
"""Write the reference pages under docs/reference/ from the tools' own definitions.

    python3 tools/generators/generate_reference_docs.py [--check]

- `lint-rules.md`: the rule table of `crates/jailint/src/rules/` (`cargo run --example lint_reference`).
- `tools.md`: a page per tool, with the `--help` of `jaic`, `jailint`, `jaifmt`, `jailsp` and each `jacket` command,
  and the environment variables of `crates/jacket-config/src/env.rs` (`cargo run --example env_reference`).

The hand-written pages (docs/tools/*.md) keep the reasons and link here, so a variable, rule or flag is
described once, where the code defines it. The script builds what it runs (cargo, and `jaic` for jaifmt).
"""

import argparse
import re
import subprocess
import sys
from toolkit.repo import ROOT, cargo_binary, cargo_target_dir, stdlib_env

OUTPUTS = [
    "docs/reference/lint-rules.md",
    "docs/reference/tools.md",
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


# The first column's name, by the start of the help's heading.
COLUMN = [("command", "Command"), ("subcommand", "Command"), ("argument", "Argument"), ("environment", "Variable")]


def inline(text):
    """Help text as Markdown: `<`, `>` and `|` escaped outside code spans."""
    parts = text.split("`")
    for i in range(0, len(parts), 2):
        parts[i] = parts[i].replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("|", "\\|")
    return "`".join(parts)


def block(text, level=4):
    """One `--help` as Markdown: the usage line as code, each `heading:` and its rows as a table,
    anything else as paragraphs. A row is indented at most eight spaces; deeper lines continue the
    row above."""
    out = []
    para = []
    rows = []
    heading = None

    def flush():
        nonlocal para, rows
        if para:
            out.append(inline(" ".join(para)) + "\n")
            para = []
        if rows:
            kind = (heading or "").lower()
            first = next((label for prefix, label in COLUMN if kind.startswith(prefix)), "Option")
            out.append(f"| {first} | Meaning |\n| --- | --- |\n" + "".join(
                f"| `{name.replace('|', '\\|')}` | {inline(' '.join(text))} |\n" for name, text in rows))
            rows = []

    for line in text.splitlines():
        stripped = line.strip()
        indent = len(line) - len(line.lstrip())
        usage = re.match(r"^[Uu]sage: (.*)$", line)
        if not stripped:
            if para:
                flush()
            continue
        if usage:
            flush()
            out.append(f"```text\n{usage.group(1)}\n```\n")
        elif indent == 0 and stripped.endswith(":") and len(stripped) < 60:
            flush()
            heading = stripped[:-1]
            out.append(f"{'#' * level} {heading[0].upper()}{heading[1:]}\n")
        elif heading is not None and indent > 8 and rows:
            rows[-1][1].append(stripped)
        elif heading is not None and indent > 0:
            if para:
                flush()
            name, meaning = (re.split(r"\s{2,}", stripped, maxsplit=1) + [""])[:2]
            rows.append((name, [meaning.strip()] if meaning.strip() else []))
        else:
            if rows:
                flush()
            para.append(stripped)
    flush()
    return "\n".join(out)


def command_rows(top_help, names):
    """The rows of `jacket --help`'s command table for `names`, as a table."""
    rows = {}
    lines = top_help.splitlines()
    for line in lines[lines.index("Commands:") + 1:]:
        if not line.startswith("  "):
            break
        name, meaning = (re.split(r"\s{2,}", line.strip(), maxsplit=1) + [""])[:2]
        rows[name] = meaning
    return "| Command | Meaning |\n| --- | --- |\n" + "".join(
        f"| `jacket {name}` | {inline(rows[name])} |\n" for name in names)


def build_jaifmt():
    """A jaifmt built from this checkout, for its `--help`."""
    out = cargo_target_dir() / "reference-docs" / "jaifmt"
    out.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(cargo_binary("jaic")), "jaifmt/main.jai", "-exe", str(out), "-quiet"],
                   cwd=ROOT, check=True, env=stdlib_env())
    return out


def tools_page():
    """`tools.md`: one `###` page per tool, grouped by what it is for."""
    cargo("build", "--locked", "-q", "-p", "jaic-cli", "-p", "jacket", "-p", "jailint", "-p", "jai-language-server")
    jaic = re.sub(r"^jaic \d[^\s:]*:", "jaic:", help_text("jaic", "-help"))
    jacket = help_text("jacket", "--help")
    jaifmt = subprocess.run([str(build_jaifmt()), "--help"], check=True, capture_output=True, text=True).stdout
    jaifmt = "\n".join(line.rstrip() for line in jaifmt.strip().splitlines())
    own = [n for n in jacket_commands(jacket) if n not in PASSTHROUGH and n != "help"]
    expected = {"new", "init", "test", "bench", "doc", "config", "clean", "explain", "version", "upgrade"}
    if set(own) != expected:
        raise SystemExit(f"jacket commands changed ({sorted(set(own) ^ expected)}): place them in tools_page()")

    def sub(name, level=4):
        return block(help_text("jacket", name, "--help"), level)

    env = example("jacket-config", "env_reference")
    env = re.sub(r"^# .*\n\n", "", env).replace("\n## ", "\n#### ")
    parts = [
        "# Tools\n",
        "`jacket` is the one command for the toolchain: it fronts the compiler (`jaic`), the linter "
        "(`jailint`), the formatter (`jaifmt`) and the language server (`jailsp`), and adds projects, "
        "tests, benchmarks and documentation. Each page lists a command's flags from its own `--help`.\n",
        "```text\njacket <COMMAND>\n```\n",
        "A command `jacket` does not know runs `jacket-<command>` from `PATH`, as `cargo` does.\n",
        "## Building\n",
        "### Compiling\n",
        "`jaic` compiles, checks and runs Jai programs. `jacket build`, `check` and `run` pass the flags "
        "of `jaic` through untouched (single-dash, as in `jacket build -o -exe app main.jai`) and add the "
        "project: with no `.jai` file named they build `[project] entry` from the nearest `jacket.toml` "
        "(else `build.jai`, `first.jai`, `main.jai`, `src/main.jai` or `module.jai`), and "
        "`[experimental]` settings become `jaic` flags unless you pass them. "
        "[The `jaic` command line](../tools/cli.md) says how the flags fit together.\n",
        "#### Commands\n",
        command_rows(jacket, ["build", "check", "run", "clean", "jaic"]),
        "#### jaic\n",
        block(jaic, 5),
        "### Creating a project\n",
        "`jacket new` makes a directory with a `jacket.toml` and a `main.jai`; `jacket init` does the "
        "same in the current directory.\n",
        "#### jacket new\n", sub("new", 5),
        "#### jacket init\n", sub("init", 5),
        "## Checking\n",
        "### Linting\n",
        "`jacket lint` runs `jailint` with the project's `[lint]` settings; its flags are those of "
        "`jailint`. [Lint rules](lint-rules.md) lists every rule, and [jailint](../tools/jailint.md) "
        "says how to configure and silence them.\n",
        block(help_text("jailint", "--help")),
        "### Formatting\n",
        "`jacket fmt` runs `jaifmt` on the project with its `[format]` settings; its flags are those of "
        "`jaifmt`. [Format rules](../tools/jaifmt-rules.md) shows what each setting does.\n",
        block(jaifmt),
        "### Language server\n",
        "`jacket lsp` runs `jailsp`, which editors start for diagnostics, completion and navigation.\n",
        block(help_text("jailsp", "--help")),
        "### Explaining diagnostics\n",
        "Every diagnostic carries a code; `jacket explain <code>` says what it means and how to fix it.\n",
        sub("explain"),
        "## Testing and documenting\n",
        "### Testing\n",
        "`jacket test` builds the `@test` procedures once and runs each in its own process. "
        "[jacket test](../tools/jacket-test.md) covers filters, coverage and the test library.\n",
        sub("test"),
        "### Benchmarking\n",
        "`jacket bench` builds the `@bench` procedures optimized and measures them. "
        "[jacket bench](../tools/jacket-bench.md) covers baselines and comparisons.\n",
        sub("bench"),
        "### Documentation\n",
        "`jacket doc` builds API documentation as a static website, like this one. "
        "[jacket doc](../tools/jacket-doc.md) covers the `[doc]` settings and themes.\n",
        sub("doc"),
        "## Setup\n",
        "### Configuration\n",
        "Settings live in the nearest `jacket.toml`; [jacket](../tools/jacket.md) lists its tables.\n",
        sub("config"),
        "### Environment variables\n",
        env,
        "### Installing\n",
        "`jacket version` prints the version of `jacket` and every tool it fronts; `jacket upgrade` "
        "installs the latest release over this one.\n",
        "#### jacket version\n", sub("version", 5),
        "#### jacket upgrade\n", sub("upgrade", 5),
    ]
    return "\n".join(parts)


def render():
    """Each output path with the text it must hold."""
    return {
        OUTPUTS[0]: example("jailint", "lint_reference"),
        OUTPUTS[1]: tools_page(),
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
