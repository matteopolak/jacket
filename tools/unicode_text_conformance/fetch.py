#!/usr/bin/env python3
"""Extract the four Unicode_Text conformance suites from the Regex generator's pinned UCD into
`target/unicode-text-conformance/`, where `stdlib/Extensions/Unicode_Text/tests/conformance.jai` reads them.
`tools/portable/build_portable.py data` runs this too.

Example: PYTHONPATH=tools python3 tools/unicode_text_conformance/fetch.py
"""
import argparse
from types import SimpleNamespace
from pathlib import Path
from generators.gen_regex_unicode import open_source
from repo_output import output_dir

SUITES = ("NormalizationTest.txt", "auxiliary/GraphemeBreakTest.txt", "auxiliary/WordBreakTest.txt", "auxiliary/SentenceBreakTest.txt")


def write_suites(output: Path = output_dir("unicode-text-conformance")) -> Path:
    """Write the suites to `output` (downloaded once, then read from the pinned-download cache)."""
    source = open_source(SimpleNamespace(ucd=None, zip=None))
    output.mkdir(parents=True, exist_ok=True)
    for name in SUITES:
        (output / name.split('/')[-1]).write_text(source.read(name), encoding="utf-8")
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=output_dir("unicode-text-conformance"))
    print(write_suites(parser.parse_args().output))


if __name__ == "__main__":
    main()
