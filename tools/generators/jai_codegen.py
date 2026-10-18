"""Shared plumbing of the generators that emit checked-in Jai: arguments and formatting.

Every generated file goes through the repository formatter (target/jaifmt, built from
jaifmt/main.jai), so the checked-in text is exactly what `--check` recomputes.
"""
import argparse
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_FORMATTER = ROOT / "target/jaifmt"


def generator_parser(description):
    """An argument parser with the --formatter and --check options every generator takes."""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--formatter", type=Path, default=DEFAULT_FORMATTER,
                        help="repository jaifmt built from source (default: target/jaifmt)")
    parser.add_argument("--check", action="store_true",
                        help="fail if a checked-in file differs from the output instead of writing it")
    return parser


def write_or_check(path, text, check=False):
    """Write `text` to `path`, or with `check` exit if `path` holds something else."""
    path = Path(path).resolve()
    if check:
        if not path.is_file() or path.read_text() != text:
            shown = path.relative_to(ROOT) if path.is_relative_to(ROOT) else path
            raise SystemExit(f"stale generated source: {shown}")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


def emit_generated(path, source, formatter=DEFAULT_FORMATTER, check=False):
    """Format `source` and write it to `path`, or with `check` exit if `path` differs."""
    formatter = Path(formatter).resolve(strict=True)
    command = [str(formatter), "--stdin", "--config", str(ROOT / "jacket.toml")]
    formatted = subprocess.run(command, input=source, text=True, capture_output=True, check=True).stdout
    repeated = subprocess.run(command, input=formatted, text=True, capture_output=True, check=True).stdout
    if repeated != formatted:
        raise ValueError(f"repository formatter is not idempotent for {path}")
    write_or_check(path, formatted, check)
