#!/usr/bin/env python3
"""Verify every normal IEEE power-of-two interval using independent exact rationals."""
import argparse
from fractions import Fraction
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def verify(output):
    rows = output.splitlines()
    expected = [(64, exponent) for exponent in range(1, 2047)]
    expected += [(32, exponent) for exponent in range(1, 255)]
    if len(rows) != len(expected):
        raise AssertionError(f"expected {len(expected)} rows, got {len(rows)}")
    for row, (width, exponent) in zip(rows, expected):
        first, second, text = row.split(",")
        if (int(first), int(second)) != (width, exponent):
            raise AssertionError(f"wrong power order: {row}")
        mantissa, bias, maximum = (52, 1023, 17) if width == 64 else (23, 127, 9)
        value = Fraction(2) ** (exponent - bias)
        spacing = Fraction(2) ** (exponent - bias - mantissa)
        lower = value - spacing / (2 if exponent == 1 else 4)
        upper = value + spacing / 2
        if not lower <= Fraction(text) <= upper:
            raise AssertionError(f"outside even-mantissa rounding interval: {row}")
        decimal_exponent = len(str(value.numerator)) - len(str(value.denominator))
        while value < Fraction(10) ** decimal_exponent:
            decimal_exponent -= 1
        while value >= Fraction(10) ** (decimal_exponent + 1):
            decimal_exponent += 1
        minimum = None
        for precision in range(1, maximum + 1):
            grid = Fraction(10) ** (decimal_exponent - precision + 1)
            ratio = value / grid
            below = ratio.numerator // ratio.denominator
            if any(lower <= coefficient * grid <= upper for coefficient in (below, below + 1)):
                minimum = precision
                break
        significant = text.split("e")[0].replace(".", "").lstrip("0").rstrip("0")
        if minimum is None or len(significant) != minimum:
            raise AssertionError(f"nonminimal significant digits (need {minimum}): {row}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--wasm", action="store_true")
    interpreter = parser.add_mutually_exclusive_group()
    interpreter.add_argument("--interp", action="store_true", help="also verify interpreter output")
    interpreter.add_argument("--interp-only", action="store_true", help="verify only interpreter output")
    args = parser.parse_args()
    if args.interp_only and args.wasm:
        parser.error("--interp-only cannot select --wasm")
    compiler = args.jaic.resolve(strict=True)
    if not compiler.is_relative_to((ROOT / "target").resolve()):
        parser.error("use a source-built independent compiler under target")
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve(strict=True)))
    directory = ROOT / "target/foundations-tests"
    directory.mkdir(exist_ok=True, parents=True)
    if args.interp or args.interp_only:
        result = subprocess.run([compiler, ROOT / "stdlib/Basic/tests/float-power-sweep.jai", "-interp"],
                                cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
        if result.returncode or result.stderr:
            raise RuntimeError(f"interpreter execution failed: {result}")
        verify(result.stdout)
        print("verified 2300 shortest power intervals: interpreter", flush=True)
        if args.interp_only:
            return
    for target in (("native", "wasm") if args.wasm else ("native",)):
        for level in ("O0", "O2"):
            suffix = ".wasm" if target == "wasm" else ""
            binary = directory / f"float-power-sweep-{target}-{level}{suffix}"
            extra = ["-os", "wasm"] if target == "wasm" else []
            command = [compiler, ROOT / "stdlib/Basic/tests/float-power-sweep.jai", f"-{level}", *extra, "-exe", binary]
            compiled = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
            if compiled.returncode or compiled.stdout or compiled.stderr:
                raise RuntimeError(f"compile failed: {compiled}")
            execute = ["node", "--no-warnings", ROOT / "tools/run_foundations_wasm.mjs", binary] if extra else [binary]
            result = subprocess.run(execute, cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
            if result.returncode or result.stderr:
                raise RuntimeError(f"execution failed: {result}")
            verify(result.stdout)
            print(f"verified 2300 shortest power intervals: {target} {level}", flush=True)


if __name__ == "__main__":
    main()
