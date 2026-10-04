#!/usr/bin/env python3
"""Run open-jai's @TestProcedure expectations against jaic.

The open-jai test files (corpus/upstream/withlang-dev--open-jai/test/**) declare
expectations as calls like `expect_program_output("test/x.jai", "out\n")`. This
script extracts those calls and checks them with jaic:

  expect_program_output[_contains]   -> `jaic run`, stdout equals / contains
  expect_compile_output[_contains]   -> `jaic check`, stdout equals / contains
  expect_compile_success             -> `jaic check` exits 0
  expect_compile_failure             -> `jaic check` fails, stderr contains message

Expectations about open-jai's own CLI (`expect_compiler_command_*`), example
annotations and output files are skipped.

Usage: tools/openjai-tests.py [--jaic PATH] [--filter SUBSTR] [-v] [-j N]
"""

import argparse
import concurrent.futures
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OPENJAI = os.path.join(ROOT, "corpus/upstream/withlang-dev--open-jai")
DEFAULT_JAIC = "/Volumes/CodexBuilds/targets/jai-dev/debug/jaic"

CALL = re.compile(r"\b(expect_[a-z_]+)\s*\(")
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')
ESCAPES = {"n": "\n", "t": "\t", "r": "\r", "0": "\0", '"': '"', "\\": "\\", "e": "\x1b"}

HANDLED = {
    "expect_program_output": ("run", "equals"),
    "expect_program_output_contains": ("run", "contains"),
    "expect_compile_output": ("check", "equals"),
    "expect_compile_output_contains": ("check", "contains"),
    "expect_compile_success": ("check", "success"),
    "expect_compile_failure": ("check", "failure"),
}


def unescape(s):
    return re.sub(r"\\(.)", lambda m: ESCAPES.get(m.group(1), m.group(1)), s)


def extract(path):
    text = open(path, encoding="utf-8").read()
    for m in CALL.finditer(text):
        # Arguments run to the matching close paren; strings never contain
        # unbalanced parens that matter because we skip over string literals.
        i, depth, args = m.end(), 1, []
        while i < len(text) and depth:
            c = text[i]
            if c == '"':
                s = STRING.match(text, i)
                args.append(unescape(s.group(1)))
                i = s.end()
                continue
            depth += c == "("
            depth -= c == ")"
            i += 1
        yield m.group(1), args, path


def run_case(jaic, kind, args):
    mode, check = HANDLED[kind]
    source = args[0]
    try:
        p = subprocess.run(
            [jaic, mode, source],
            cwd=OPENJAI,
            capture_output=True,
            text=True,
            errors="replace",
            timeout=300,
        )
    except subprocess.TimeoutExpired:
        return False, "timeout"
    out, err = p.stdout, p.stderr
    if check == "failure":
        if p.returncode == 0:
            return False, "compiled, expected failure"
        return (args[1] in err or args[1] in out), err.strip()[:400]
    if p.returncode != 0 and mode == "check":
        return False, err.strip()[:400]
    if check == "success":
        return True, ""
    # open-jai compares output with trailing spaces trimmed from each line.
    trim = lambda s: "\n".join(line.rstrip() for line in s.split("\n"))
    if check == "equals":
        ok = trim(out) == trim(args[1])
    else:
        ok = trim(args[1]) in trim(out)
    return ok, f"exit {p.returncode}\nexpected: {args[1]!r}\nactual:   {out[:400]!r}\n{err.strip()[:300]}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jaic", default=DEFAULT_JAIC)
    ap.add_argument("--filter", default="")
    ap.add_argument("-v", action="store_true")
    ap.add_argument("-j", type=int, default=8)
    a = ap.parse_args()

    cases, skipped = [], 0
    for dirpath, _, files in os.walk(os.path.join(OPENJAI, "test")):
        for f in sorted(files):
            if f.endswith("_tests.jai"):
                for kind, args, path in extract(os.path.join(dirpath, f)):
                    if kind not in HANDLED or not args:
                        skipped += 1
                    elif a.filter in args[0]:
                        cases.append((kind, args))

    failed = 0
    with concurrent.futures.ThreadPoolExecutor(a.j) as ex:
        results = ex.map(lambda c: run_case(a.jaic, *c), cases)
        for (kind, args), (ok, detail) in zip(cases, results):
            if not ok:
                failed += 1
                print(f"FAIL {kind} {args[0]}")
                if a.v:
                    print("  " + detail.replace("\n", "\n  "))
    print(f"{len(cases) - failed} passed, {failed} failed, {skipped} skipped")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
