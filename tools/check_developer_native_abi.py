#!/usr/bin/env python3
"""Compare reviewed native model layouts/enum values with official public C headers.

Only installed clang-c public headers supply the native oracle. Generated probes
are durable products of this source under our own target directory.
"""
import json
import ctypes
import hashlib
import os
import build_developer_bindings
from pathlib import Path
import re
import subprocess
import sys
import generate_developer_binding_model as model

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True,
                            text=True, timeout=60)
    if result.returncode or result.stderr:
        raise RuntimeError(f"Native ABI probe failed: {command}: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    prefix = build_developer_bindings.llvm_prefix()
    spec = model.SPEC.read_text()
    inventory = spec.split("### Unix-specific declarations", 1)[1].split("### Windows-specific declarations", 1)[0]
    declarations = dict(re.findall(r"^- `([A-Za-z_][A-Za-z_0-9]*) :: ([^`]+)`", inventory, re.MULTILINE))
    header_paths = sorted((prefix / "include/clang-c").glob("*.h"))
    headers = "\n".join(path.read_text() for path in header_paths)
    public_c = re.sub(r"/\*.*?\*/|//[^\n]*", "", headers, flags=re.DOTALL)
    native_enums = dict(re.findall(r"\benum\s+(\w+)\s*\{(.*?)\}", public_c, re.DOTALL))
    anonymous_enums = {name: body for body, name in re.findall(r"typedef\s+enum\s*\{(.*?)\}\s*(\w+)\s*;", public_c, re.DOTALL)}
    native_enums.update(anonymous_enums)
    expressions = []
    for name, kind in sorted(declarations.items()):
        if not kind.startswith(("struct", "enum")):
            continue
        shape = model.aggregate(spec, name, "Unix")
        if re.search(r"struct\s*\{\s*\}", shape):
            continue  # Publicly opaque pointees have no native by-value layout.
        native_name = f"enum {name}" if kind.startswith("enum") and name not in anonymous_enums else name
        if name == "CXUnsavedFile":
            native_name = "struct CXUnsavedFile"  # Public C tag has no typedef.
        expressions.append((f"sizeof({native_name})", f"size_of(B.{name})"))
        expressions.append((f"_Alignof({native_name})", f"align_of(B.{name})"))
        if kind.startswith("struct"):
            for field in re.findall(r"^\s+(\w+)\s*:", shape, re.MULTILINE):
                if name == "CXIndexOptions" and field == "__bitfield":
                    continue  # Named Jai storage for C bit views; dedicated effect probe covers it.
                native_field = "context" if name == "CXCursorAndRangeVisitor" and field == "_context" else field
                expressions.append((f"offsetof({native_name}, {native_field})", f'offset_of(B.{name}, "{field}")'))
        else:
            if name not in native_enums:
                raise RuntimeError(f"Native public enum definition unavailable: {name}")
            # Obtain declarations independently of their initializer expressions.
            constants = re.findall(r"(?:^|,)\s*([A-Za-z_]\w*)\s*(?==|,|$)", native_enums[name])
            for member in re.findall(r"^\s+(\w+)\s*::", shape, re.MULTILINE):
                candidates = [value for value in constants if value == member or value.endswith("_" + member)]
                if len(candidates) != 1:
                    raise RuntimeError(f"Cannot uniquely match reviewed/native enum member: {name}.{member}: {candidates}")
                expressions.append((candidates[0], f"cast(s64) B.{name}.{member}"))
    source = ['#include <clang-c/Index.h>', '#include <clang-c/BuildSystem.h>', '#include <stddef.h>', '#include <stdio.h>', 'int main(void) {']
    for expression, _ in expressions:
        source.append(f'    printf("%lld\\n", (long long) ({expression}));')
    source.append('    return 0;\n}')
    c_path = ROOT / "target/developer-tools-native-abi.c"
    c_path.write_text("\n".join(source) + "\n")
    executable = ROOT / "target/developer-tools-native-abi"
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include", c_path, "-o", executable])
    values = run([executable]).splitlines()
    if len(values) != len(expressions):
        raise RuntimeError("Native ABI probe did not return every requested value")
    jai = ['#import "Basic";', 'B :: #import "Bindings_Generator";', 'main :: () {']
    for (_, expression), value in zip(expressions, values):
        jai.append(f'    assert(({expression}) == {int(value)});')
    jai += ['    print("ok\\n");', '}']
    jai_path = ROOT / "target/developer-tools-native-abi.jai"
    jai_path.write_text("\n".join(jai) + "\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", jai_path])
    library_path = prefix / ("bin/libclang.dll" if sys.platform == "win32" else
                             "lib/libclang.dylib" if sys.platform == "darwin" else "lib/libclang.so")
    library = ctypes.CDLL(str(library_path))
    exports = set()
    for path in [ROOT / "stdlib/Bindings_Generator/native-parser-api.jai", ROOT / "stdlib/Bindings_Generator/native-extended-api.jai"]:
        exports.update(re.findall(r"^\s*(clang_\w+)\s*::", path.read_text(), re.MULTILINE))
    missing = []
    for name in sorted(exports):
        try:
            getattr(library, name)
        except AttributeError:
            missing.append(name)
    if missing:
        raise RuntimeError(f"Reviewed native bindings absent from installed official SDK: {missing}")
    evidence = {"scope": "host official SDK layouts and enum values; opaque pointees and direct procedure invocation excluded",
                "assertions": len(expressions), "sdk_prefix": str(prefix),
                "sdk_headers_sha256": {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in header_paths},
                "compiler_version": run([prefix / "bin/clang", "--version"]),
                "probe_generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "public_symbol_lookups": len(exports), "missing_public_symbols": missing,
                "native_expressions": [{"c": c, "jai": j, "value": int(value)} for (c, j), value in zip(expressions, values)]}
    (ROOT / "docs/native/developer-tools-native-abi-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print(f"Native SDK oracle prepared {len(expressions)} layout/enum comparisons; {len(exports)} public symbols resolve (not invoked)")


if __name__ == "__main__":
    main()
