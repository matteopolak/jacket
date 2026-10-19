#!/usr/bin/env python3
"""Run Miri on the parts of the compiler that handle raw memory and need nothing from the operating system.

    python3 tools/ci/run_miri.py [extra cargo-miri test arguments]

Miri cannot call foreign functions, run inline assembly or switch stacks, so the fiber and native-call
modules, the memory probe (mincore/mmap) and the thread tests are out. What is left is the interpreter's
sandboxed libc, its error cell and the record layouts; the parser and the type checker are safe Rust and
too slow under Miri to be worth it. See docs/tools/sanitizers.md.
"""

import subprocess
import sys

# libtest name filters (any match runs). Add a module here once its tests pass under Miri.
RUN = [
    "interp::sandbox",
    "interp::last_error",
    "records::",
    "wide_float",
    "sema::interval",
]
# Tests Miri cannot judge: it perturbs the result of `powi` by a few ULP on purpose, and this one compares
# against `2f64.powi(-60)`.
SKIP: list[str] = ["extra_precision_is_kept"]


def main() -> int:
    command = [
        "cargo",
        "miri",
        "test",
        "-p",
        "jaic",
        "--lib",
        *sys.argv[1:],
        "--",
        *RUN,
    ]
    for name in SKIP:
        command += ["--skip", name]
    print("+", " ".join(command), flush=True)
    return subprocess.call(command)


if __name__ == "__main__":
    sys.exit(main())
