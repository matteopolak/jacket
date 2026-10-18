#!/usr/bin/env python3
"""Materialize the native Socket declarations of docs/specs/stdlib/modules/socket.md.

Each target OS gets one file, stdlib/internal/net/socket-abi-<os>.jai, that the Socket module
loads straight into its own scope, so the declarations are the public names. Authored helpers
(field accessors, ancillary-data macros, the FD_* wrappers) are never emitted; they live in the
hand-written socket-*.jai files next to the generated ones.
"""
from collections import defaultdict
import argparse
import json
from pathlib import Path
import re
import subprocess

from portable_api_spec import parse_spec

ROOT = Path(__file__).resolve().parents[1]
OSES = ["macos", "linux", "android", "windows"]


def condition(text):
    return text.replace("else branch", "OS != .WINDOWS")


def route(guard):
    """The target file of a spec condition, and the condition left to test inside that file."""
    simple = re.fullmatch(r"OS == \.(\w+)", guard)
    if simple:
        return simple[1].lower(), ""
    if guard == "CPU == .X64 && OS == .MACOS":
        return "macos", "CPU == .X64"
    if guard == "OS != .WINDOWS && OS == .MACOS":
        return "macos", ""
    raise RuntimeError(f"no target file for condition {guard!r}")


def guarded(code, guard):
    return f"#if {guard} {{\n{code}\n}}\n" if guard else code + "\n"


def declaration_group(declaration):
    """Our organization: layout types, ABI calls, mutable data, then constants.

    Stable sorting retains source order within a same-named overload family.
    Aggregate bodies and enum numbering are never rearranged.
    """
    code = declaration.code
    if re.search(r"::\s*(?:struct|union|enum(?:_flags)?|#type)(?:\s|$|,)", code):
        group = 0
    elif "#foreign" in code or "#elsewhere" in code:
        group = 1
    elif re.match(r"^\w+\s*:(?!:)", code):
        group = 2
    else:
        group = 3
    return group, declaration.name.casefold(), declaration.name


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--formatter", type=Path, required=True,
                        help="source-built repository jaifmt under target")
    parser.add_argument("--output-root", type=Path, default=ROOT,
                        help="write generated stdlib/docs under this owned checkout root (default: checkout)")
    args = parser.parse_args()
    formatter = args.formatter.resolve(strict=True)
    if not formatter.is_relative_to((ROOT / "target").resolve()):
        parser.error("formatter must be a source build under this checkout's target directory")
    output_root = args.output_root.resolve()
    if not output_root.is_relative_to(ROOT):
        parser.error("generation output must remain in this owned checkout")
    page = ROOT / "docs/specs/stdlib/modules/socket.md"
    spec = parse_spec(page.read_text())
    shapes = defaultdict(list)
    for shape in spec.shapes:
        shapes[shape.name].append(shape)
    files, seen = defaultdict(list), set()
    manual = {"Socket", "INVALID_SOCKET", "SOCKET_ERROR", "SOCKET_WOULDBLOCK", "SOCKET_WOULDBLOCK_CONNECT",
              "FD_BITS_ELEMENT_TYPE", "BITS_PER_FD_SET_MASK"}
    for declaration in sorted(spec.declarations, key=declaration_group):
        code, name = declaration.code.rstrip(";"), declaration.name
        guard = condition(declaration.condition)
        if (guard, code) in seen:
            continue
        seen.add((guard, code))
        if name == "in_addr" and guard == "OS == .MACOS":
            continue  # shape supplemented from the Apple SDK netinet/in.h
        if "#char ," in code:
            continue  # incomplete character literal; FIONBIO has an authored wrapper
        procedure = re.match(r"^\w+\s*::\s*(?:inline\s+)?\((.*?)\)\s*(?:->|#|$)", code)
        if name in manual or (procedure and (":" in procedure[1] or not procedure[1]) and "#foreign" not in code and "#elsewhere" not in code):
            continue  # authored wrapper or accessor
        if re.search(r"::\s*(?:struct|union|enum(?:_flags)?)(?:\s|$)", code):
            candidates = [s for s in shapes[name] if condition(s.condition) == guard]
            if not candidates:
                candidates = [s for s in shapes[name] if not s.condition]
            if not candidates and len(shapes[name]) == 1:
                candidates = shapes[name]
            if not candidates:
                continue
            bodies = []
            for shape in candidates:
                body = shape.code
                if shape.member_list:
                    body = re.sub(r"(\w+):\s*([^;{}]+?)\s+#overlay\s+(\w+);", r"#overlay (\3) \1: \2;", body)
                    body = re.sub(r"([^;{}\s])\s+}", r"\1; }", body)
                if body.startswith(name + " :: ") and "{" in body:
                    header = body[:body.index("{")]
                    if "(" not in header and not re.search(r"#(?:align|no_padding)", header):
                        body = code + " " + body[body.index("{"):]
                bodies.append(guarded(body, condition(shape.condition) if condition(shape.condition) != guard else ""))
            emitted = "\n".join(bodies)
        else:
            emitted = code + ";"
        target, residual = route(guard)
        files[target].append(guarded(emitted, residual))
    # Only the Windows names these declarations use are imported by name: a wildcard import would
    # let same-named types (notably the Windows GUI MSG versus the Socket flag enum) capture them.
    windows_spec = parse_spec((ROOT / "docs/specs/stdlib/modules/windows.md").read_text())
    external_names = {declaration.name for declaration in windows_spec.declarations}
    local_names = {declaration.name for declaration in spec.declarations}
    used_names = set(re.findall(r"\b[A-Za-z_]\w*\b", "\n".join(files["windows"])))
    dependencies = sorted((used_names & external_names) - local_names)
    generator = "tools/generate_socket_abi.py"
    written = []
    for target in OSES:
        header = f"// Native socket declarations for {target}, generated by {generator} from\n"
        header += "// docs/specs/stdlib/modules/socket.md. Edit the generator, not this file.\n"
        if target == "windows":
            header += '#scope_file\nWindows_Native :: #import "Windows";\n\n'
            header += "".join(f"{name} :: Windows_Native.{name};\n" for name in dependencies)
            header += 'ws2_32 :: #system_library "ws2_32";\n'
        else:
            header += '#scope_file\n#import "POSIX";\n\n'
            header += "// Unix select takes the native timeval through a private spelling.\n"
            header += "TIMEVAL :: timeval;\n"
            header += 'libc :: #system_library "libc";\n'
        header += "#scope_export\n\n"
        path = output_root / f"stdlib/internal/net/socket-abi-{target}.jai"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(header + "\n".join(files[target]))
        written.append(path)
    subprocess.run([str(formatter), "--config", str(ROOT / "jaifmt.toml"), *map(str, written)],
                   cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
