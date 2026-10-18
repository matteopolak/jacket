#!/usr/bin/env python3
"""One benchmark harness for the compiler, the language server, the linter and the formatter.

    python3 tools/bench/main.py compile [--compiler NAME=PATH ...] [--only TEXT] [--repeat N]
    python3 tools/bench/main.py lsp     [--server NAME=COMMAND ...]
    python3 tools/bench/main.py lint
    python3 tools/bench/main.py fmt

Every subcommand writes JSON and a Markdown summary (default: target/bench/<subcommand>.{json,md}) and,
with --compare OLD.json, exits 1 when something got slower or bigger. See docs/tools/benchmarks.md.
"""
from __future__ import annotations

import argparse
import sys

# The modules here import each other by name: running this file puts its directory on sys.path.
import cmd_compile
import cmd_fmt
import cmd_lint
import cmd_lsp

COMMANDS = {'compile': cmd_compile, 'lsp': cmd_lsp, 'lint': cmd_lint, 'fmt': cmd_fmt}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    subparsers = parser.add_subparsers(dest='command', required=True)
    for name, module in COMMANDS.items():
        sub = subparsers.add_parser(name, help=module.__doc__.splitlines()[0])
        module.add_arguments(sub)
        sub.set_defaults(handler=module.run)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return args.handler(args)


if __name__ == '__main__':
    sys.exit(main())
