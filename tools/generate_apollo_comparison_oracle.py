#!/usr/bin/env python3
"""Generate exact signed128 ordering/zero checks using independent Python integers."""

import argparse
from pathlib import Path
import random

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / "tests/portable-stdlib/apollo-comparisons/word-boundary-oracle.jai"


def generate():
    minimum = -(1 << 127)
    maximum = (1 << 127) - 1
    values = [
        minimum,
        minimum + 1,
        -(1 << 64) - 1,
        -(1 << 64),
        -(1 << 64) + 1,
        -(1 << 63),
        -1,
        0,
        1,
        (1 << 63) - 1,
        1 << 63,
        (1 << 64) - 1,
        1 << 64,
        (1 << 64) + 1,
        maximum - 1,
        maximum,
    ]
    pairs = [(a, b) for a in values for b in values]
    rng = random.Random(1282026)
    pairs += [
        (rng.randrange(minimum, maximum + 1), rng.randrange(minimum, maximum + 1))
        for _ in range(96)
    ]

    def literal(value):
        return f"B.S128.{{ 0x{value & ((1 << 64) - 1):016x}, {value >> 64} }}"

    lines = [
        "// Independently generated Python signed-integer oracle: seed1282026.\n",
        'B :: #import "Basic";\n\n',
        'using,only(.["<", "<=", ">", ">=", "==", "!"]) B;\n\n',
        "verify :: (a: B.S128, b: B.S128, comparison: s32, az: bool, bz: bool) {\n",
        "    B.assert((a < b) == (comparison < 0));\n",
        "    B.assert((a <= b) == (comparison <= 0));\n",
        "    B.assert((a > b) == (comparison > 0));\n",
        "    B.assert((a >= b) == (comparison >= 0));\n",
        "    B.assert((a == b) == (comparison == 0));\n",
        "    B.assert((a != b) == (comparison != 0));\n",
        "    B.assert((!a) == az && (!b) == bz);\n",
        "}\n\nmain :: () {\n",
    ]
    for a, b in pairs:
        comparison = (a > b) - (a < b)
        lines.append(
            f"    verify({literal(a)}, {literal(b)}, {comparison}, "
            f"{str(a == 0).lower()}, {str(b == 0).lower()});\n"
        )
    lines.append("}\n")
    return "".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    text = generate()
    if args.check:
        if DESTINATION.read_text() != text:
            raise SystemExit("signed128 oracle does not match deterministic generator")
    else:
        DESTINATION.write_text(text)


if __name__ == "__main__":
    main()
