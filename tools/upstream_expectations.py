#!/usr/bin/env python3
"""Derive output expectations for upstream sweep cases from a project's own documented output.

The_Way_to_Jai writes the output of its examples next to the printing line, as `// => text`
(on the same line or on a comment line right after it). The author ran these programs with
Jai, so the annotations are third-party evidence of what Jai prints. This
tool turns them into `expect.stdout_ordered` records in tools/upstream-cases.json, which
jaic-sweep.py checks.

Every annotation whose text is not in jaic's output was reviewed by hand: it is either a jaic
bug (fixed, with a regression test) or listed in REJECTED with the reason it is not evidence
(nondeterministic, platform-specific, written for an older Jai, or a typo in the book).

Usage:
  upstream_expectations.py            report which cases would change
  upstream_expectations.py --write    rewrite the generated `expect` records
  upstream_expectations.py --verify JAIC
                                      run the annotated cases and list annotations missing from
                                      stdout (review each before rejecting it)
"""
import argparse, json, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CASES = ROOT / "tools/upstream-cases.json"
UPSTREAM = ROOT / "corpus/upstream"
PROJECT = "Ivo-Balbaert--The_Way_to_Jai/"
SOURCE_PREFIX = "The_Way_to_Jai `// =>` annotations"

# Lines whose code prints (the annotation then describes that output).
PRINTS = re.compile(r"\b(print|print_\w+|write_\w+|log)\s*\(")
ANNOTATION = re.compile(r"//\s*=>\s?(.*)$")
# Output that differs per run or machine.
ADDRESS = re.compile(r"\b[0-9a-f]{1,4}(_[0-9a-f]{4}){2,}\b|0x[0-9a-f_]{6,}")
UNSTABLE = re.compile(r"\b\d+(\.\d+)? ms\b|took|seconds", re.I)
# Annotations that describe a diagnostic or a crash rather than stdout.
NOT_STDOUT = re.compile(r"rror|crash|panic|warning|stack trace|assert|bounds check|no output", re.I)

# Reviewed annotations that are not evidence about current Jai. Key: path in the project:line.
REJECTED = {
    "examples/04/4.2_intrinsics.jai:8": "typo: the text ends in ';' (the line prints 'y is 42')",
    "examples/05/5.1_literals.jai:18": "typo: drops the comma of \"Hello, Sailor!\"",
    "examples/05/5.3_variable_declarations.jai:22": "shows the empty string as \"\" (print writes nothing)",
    "examples/05/5.3_variable_declarations.jai:28": "shows the string quoted; print writes it bare",
    "examples/05/5.3_variable_declarations.jai:41": "value of an uninitialized (---) variable",
    "examples/06/6.1_bools.jai:16": "commentary, not output",
    "examples/06/6.3_numbers.jai:31": "stale: -5_069_105 + 10 is -5069095 (open-jai corrected it too)",
    "examples/06/6B/6B.2_measuring_time.jai:17": "current time",
    "examples/09/9.1_types.jai:12": "commentary '(bytes)' after the value",
    "examples/09/9.1_types.jai:17": "commentary after the value",
    "examples/09/9.1_types.jai:30": "capitalised 'The'; the format string says 'the'",
    "examples/10/10.1_pointers.jai:34": "commentary after the value",
    "examples/10/10.4_dangling_pointers.jai:11": "reads freed memory (whatever the allocator left there)",
    "examples/12/12.1_struct_declarations.jai:65": "commentary '(bytes)' after the value",
    "examples/12/12.13_anonymous_struct.jai:28": "older Jai printed a Type value as u64; the format has no newline either",
    "examples/12/12.9_struct_parameters.jai:14": "shows the string quoted; print writes it bare",
    "examples/13/13.1_unions.jai:50": "float 3.0 prints as 3 (the book writes the literal)",
    "examples/13/13.1_unions.jai:54": "commentary ('same as (6)')",
    "examples/14/14.2_ifx.jai:59": "typo: labels y5 as 'y4' (value 7 is right, see how_to 025_ifx implicit then)",
    "examples/15/15.1_while.jai:39": "commentary ('printed out 3 times')",
    "examples/15/15.8_for_reverse.jai:12": "predates beta 0.1.094: `for < f..2` counted down; now it is an empty range reversed",
    "examples/16/16.2_type_info_proc.jai:15": "stale: the format string has a comma after the first %",
    "examples/16/16.2_type_info_proc.jai:18": "typo: 'infor'; the format string says 'ti'",
    "examples/16/16.2_type_info_proc.jai:42": "elided with '...'",
    "examples/16/16.3_enum_specified.jai:25": "the line's output continues on the next print ('specified.')",
    "examples/16/16.4_check_#as.jai:33": "stray opening quote",
    "examples/17/17.14_reflection_procedure.jai:9": "output continues on the same line; checked by the next annotation",
    "examples/17/17.5_multiple_return.jai:29": "leading space in the annotation",
    "examples/18/18.10_var_args.jai:12": "shows the string quoted; print writes it bare",
    "examples/18/18.11_array_of_structs.jai:21": "value of an uninitialized (---) array",
    "examples/18/18.2_static_arrays.jai:75": "older Print wording for an empty Any",
    "examples/18/18.3_dynamic_arrays.jai:30": "stale: written before the pop() above it was added",
    "examples/18/18.3_dynamic_arrays.jai:34": "stale: written before the pop() above it was added",
    "examples/18/18.3_dynamic_arrays.jai:36": "stale: written before the pop() above it was added",
    "examples/18/18.8_array_view_misuse.jai:13": "prints a dangling view (heap garbage)",
    "examples/18/18B_ordered_remove.jai:16": "annotation of commented-out code",
    "examples/19/19.1_strings.jai:93": "drops one of the two spaces the format produces",
    "examples/19/19.1_strings.jai:103": "format string says 'after for'",
    "examples/19/19.1_strings.jai:114": "format string says 'after for'",
    "examples/19/19.1_strings.jai:122": "format string says 'after for'",
    "examples/19/19.2_bytes.jai:5": "commentary after the value",
    "examples/19/19.2_bytes.jai:6": "commentary after the value",
    "examples/19/19.4_string_operations.jai:31": "only for other input (the comment says str3 == \"abc\")",
    "examples/19/19.7_linux_input.jai:10": "reads stdin",
    "examples/20/20.5_array_of_structs.jai:19": "value of an uninitialized (---) array",
    "examples/21/21.1_temp_storage.jai:14": "drops one of the two spaces the format produces",
    "examples/25/25.4_check_stack.jai:20": "depends on stack layout of locals (unspecified)",
    "examples/26/26.1_type_table.jai:10": "type table size depends on the compiler and modules",
    "examples/26/26.19_modify4.jai:34": "spacing differs from the format string",
    "examples/26/26.5_if.jai:49": "Windows-only output",
    "examples/27/27.5_paths.jai:12": "machine-specific path",
    "examples/28/28.5_get_cpu_info_feature_flags.jai:7": "machine-specific CPU",
    "examples/28/28.5_get_cpu_info_feature_flags.jai:9": "x64-only (AVX2)",
    "examples/29/29.1_call_c_linux.jai:13": "libc rand() sequence differs per platform",
    "examples/29/29.1_call_c_linux.jai:14": "process id",
    "examples/29/29.4_get_computer_name.jai:52": "Windows-only output",
    "examples/29/29.4_get_computer_name.jai:53": "machine-specific",
    "examples/29/29.4_get_computer_name.jai:56": "machine-specific",
    "examples/30/30.12_placeholder.jai:13": "wording differs from the format string ('is it a constant')",
    "examples/30/30.2_location.jai:12": "machine-specific path",
    "examples/30/30.2_location.jai:14": "machine-specific path",
    "examples/30/30.2_location.jai:17": "machine-specific path",
    "examples/30/30.3_build.jai:10": "wording differs from the format string ('workspace w')",
    "examples/31/31.5_num_threads.jai:13": "machine-specific CPU count",
}

# Cases whose annotations are not in output order (output from #run, from procedures declared
# after main, or from loops). Their expectation uses `stdout_contains` instead.
UNORDERED = {
    "examples/26/26.7B_macros_basics.jai": "macros declared above main annotate their output first",
}


def annotations(path):
    """[(line, text)] for the `// =>` annotations that describe the output of a printing line."""
    out, prev_code, depth = [], "", 0
    for number, line in enumerate(path.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
        stripped = line.strip()
        if depth or stripped.startswith("/*"):
            depth += line.count("/*") - line.count("*/")
            continue
        code = line.split("//", 1)[0]
        m = ANNOTATION.search(line)
        if m:
            # Trailing `// (2)` remarks and a leading "prints:" are the author's commentary.
            text = re.sub(r"^prints:\s*", "", m.group(1).split(" //", 1)[0].rstrip())
            if code.strip():
                printing = PRINTS.search(code)
            else:
                # A comment line continues the previous printing line only when it is just the
                # annotation (not commented-out code with one).
                printing = re.match(r"//\s*=>", stripped) and PRINTS.search(prev_code)
            if (printing and text and not NOT_STDOUT.search(text) and not ADDRESS.search(text)
                    and not UNSTABLE.search(text)):
                out.append((number, text))
        if code.strip():
            prev_code = code
    return out


def expectation(case):
    rel = case["path"][len(PROJECT):]
    found = [(n, t) for n, t in annotations(UPSTREAM / case["path"]) if f"{rel}:{n}" not in REJECTED]
    if not found:
        return None
    key = "stdout_contains" if rel in UNORDERED else "stdout_ordered"
    return {key: [t for _, t in found],
            "source": f"{SOURCE_PREFIX}: {rel} lines {', '.join(str(n) for n, _ in found)}"}


def generated(case):
    return case.get("expect", {}).get("source", "").startswith(SOURCE_PREFIX)


def candidates(cases):
    return [c for c in cases if c["path"].startswith(PROJECT) and c["mode"] == "run"]


def write(cases):
    lines = ",\n".join("  " + json.dumps(c, ensure_ascii=False) for c in cases)
    CASES.write_text("[\n" + lines + "\n]\n")


def verify(cases, jaic):
    def run(c):
        path = UPSTREAM / c["path"]
        anns = annotations(path)
        r = subprocess.run([jaic, "run", str(path), *c.get("args", [])], cwd=path.parent,
                           capture_output=True, stdin=subprocess.DEVNULL, timeout=300)
        out = r.stdout.decode(errors="replace")
        rel = c["path"][len(PROJECT):]
        return [(f"{rel}:{n}", t, f"{rel}:{n}" in REJECTED) for n, t in anns if t not in out]
    with ThreadPoolExecutor(3) as pool:
        for missing in pool.map(run, candidates(cases)):
            for key, text, rejected in missing:
                if not rejected:
                    print(f"{key}: {text!r}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--verify", metavar="JAIC")
    a = ap.parse_args()
    if not UPSTREAM.is_dir():
        sys.exit("corpus/upstream is missing; run tools/fetch_upstreams.py")
    cases = json.loads(CASES.read_text())
    if a.verify:
        verify(cases, a.verify)
        return
    changed = 0
    for c in candidates(cases):
        new = expectation(c)
        if generated(c) or (new and "expect" not in c):
            if c.get("expect") != new:
                changed += 1
                if new:
                    c["expect"] = new
                else:
                    c.pop("expect", None)
    if a.write:
        write(cases)
    print(f"{changed} case(s) {'updated' if a.write else 'would change'}")


if __name__ == "__main__":
    main()
