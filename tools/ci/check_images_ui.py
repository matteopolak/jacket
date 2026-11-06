#!/usr/bin/env python3
"""Check cross-target macros and runtime dispatch used by images and UI.

    PYTHONPATH=tools python3 tools/ci/check_images_ui.py --jaic target/release/jaic --jacket target/release/jacket

The focused Intel CI job combines these checks with the existing portable producer
suite and native UI image runner. It does not replace the full stdlib matrix.
"""

import argparse
from pathlib import Path
import platform
import subprocess

from repo_output import output_dir
from toolkit.repo import ROOT, cargo_binary

DISPATCH = "stdlib/Extensions/SIMD/tests/cpu-dispatch.jai"
WEB_DOCS = (
    "doc: Extensions/Web.html#1",
    "doc: Extensions/Web#1",
    "doc: Extensions/Web.render_parallel#1",
)


def run(*command):
    print("+", *map(str, command), flush=True)
    # Jacket owns its case budgets and process groups; the CI job bounds the aggregate.
    subprocess.run(list(map(str, command)), cwd=ROOT, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=cargo_binary("jaic", "release"))
    parser.add_argument(
        "--jacket", type=Path, default=cargo_binary("jacket", "release")
    )
    parser.add_argument(
        "--output-root", type=Path, default=output_dir("images-ui-intel")
    )
    args = parser.parse_args()
    compiler = args.jaic.resolve(strict=True)
    jacket = args.jacket.resolve(strict=True)
    output = args.output_root.resolve()
    output.mkdir(parents=True, exist_ok=True)
    print("image portability host:", platform.system(), platform.machine(), flush=True)

    for target in ("linux", "macos", "windows"):
        for cpu in ("x64", "arm64"):
            run(compiler, "-check", "-os", target, "-cpu", cpu, DISPATCH)
    run(compiler, "-interp", DISPATCH)
    for mode in ("O0", "O2"):
        executable = output / f"dispatch-{mode}"
        run(compiler, f"-{mode}", DISPATCH, "-exe", executable)
        run(executable)
    for target in ("linux", "macos", "windows"):
        run(
            jacket,
            "test",
            "--doc",
            "--check-only",
            "--os",
            target,
            "--cpu",
            "x64",
            "-j",
            "2",
            "--exact",
            *WEB_DOCS,
        )
    print("image portability dispatch/docs: passed", flush=True)


if __name__ == "__main__":
    main()
