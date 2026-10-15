#!/usr/bin/env python3
"""Materialize only reviewed native Socket declarations, never procedure stubs."""
from collections import defaultdict
import argparse
import json
from pathlib import Path
import re
import subprocess

from portable_api_spec import parse_spec

ROOT = Path(__file__).resolve().parents[1]


def condition(text):
    return text.replace("else branch", "OS != .WINDOWS")


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
    if not formatter.is_relative_to(ROOT / "target"):
        parser.error("formatter must be a source build under this checkout's target directory")
    output_root = args.output_root.resolve()
    if not output_root.is_relative_to(ROOT):
        parser.error("generation output must remain in this owned checkout")
    page = ROOT / "docs/specs/stdlib/modules/socket.md"
    spec = parse_spec(page.read_text())
    shapes = defaultdict(list)
    for shape in spec.shapes:
        shapes[shape.name].append(shape)
    output, ledger, exported, seen = [], [], [], set()
    fields_facade = ROOT / "stdlib/Socket/fields.jai"
    authored_fields = set(re.findall(r"^\s*(\w+)\s*::\s*Fields\.", fields_facade.read_text(), re.MULTILINE)) if fields_facade.exists() else set()
    scope_facade = ROOT / "stdlib/Socket/scope-fields.jai"
    authored_scope = set(re.findall(r"^\s*(\w+)\s*::\s*Fields\.", scope_facade.read_text(), re.MULTILINE)) if scope_facade.exists() else set()
    linux_facade = ROOT / "stdlib/Socket/tcp-unix-fields.jai"
    authored_linux = set(re.findall(r"^\s*(\w+)\s*::\s*Fields\.", linux_facade.read_text(), re.MULTILINE)) if linux_facade.exists() else set()
    cmsg_facade = ROOT / "stdlib/Socket/cmsg.jai"
    authored_cmsg = set(re.findall(r"^\s*(\w+)\s*::\s*Cmsg\.", cmsg_facade.read_text(), re.MULTILINE)) if cmsg_facade.exists() else set()
    manual = {"Socket", "INVALID_SOCKET", "SOCKET_ERROR", "SOCKET_WOULDBLOCK", "SOCKET_WOULDBLOCK_CONNECT"}
    for declaration in sorted(spec.declarations, key=declaration_group):
        code, name = declaration.code.rstrip(";"), declaration.name
        guard = condition(declaration.condition)
        if (guard, code) in seen:
            continue
        seen.add((guard, code))
        record = {"name": name, "condition": guard, "spec_line": declaration.line}
        if name == "in_addr" and guard == "OS == .MACOS":
            record["status"] = "native shape supplemented from Apple SDK netinet/in.h"
            ledger.append(record)
            continue
        if "#char ," in code:
            if name == "FIONBIO":
                record["status"] = "incomplete reviewed literal resolved by official Winsock2 ABI; authored IOW/FIONBIO wrapper"
            else:
                record["status"] = "unresolved incomplete character literal in reviewed declaration"
            ledger.append(record)
            continue
        procedure = re.match(r"^\w+\s*::\s*(?:inline\s+)?\((.*?)\)\s*(?:->|#|$)", code)
        if name in manual or (procedure and (":" in procedure[1] or not procedure[1]) and "#foreign" not in code and "#elsewhere" not in code):
            if name in authored_fields and guard == "OS == .MACOS":
                record["status"] = "authored native field accessor; native SDK vector verification separate"
            elif name in authored_scope and guard == "OS == .WINDOWS":
                record["status"] = "authored native scope accessor; cross-target compile-time vectors verified, native Windows pending"
            elif name in authored_linux and guard == "OS == .LINUX":
                record["status"] = "authored native TCP accessor; cross-target compile-time vectors verified, native Linux pending"
            elif name in authored_linux and guard == "OS == .ANDROID" and "tcpi_" not in name:
                record["status"] = "authored shared TCP accessor; bionic header ABI checked on host, Android target unavailable"
            elif name in authored_cmsg:
                record["status"] = "authored native ancillary helper; host SDK vectors verified, other native platforms pending"
            else:
                record["status"] = "authored wrapper or pending helper"
            ledger.append(record)
            continue
        if re.search(r"::\s*(?:struct|union|enum(?:_flags)?)(?:\s|$)", code):
            candidates = [s for s in shapes[name] if condition(s.condition) == guard]
            if not candidates:
                candidates = [s for s in shapes[name] if not s.condition]
            if not candidates and len(shapes[name]) == 1:
                candidates = shapes[name]
            if not candidates:
                record["status"] = "unresolved shape"
                ledger.append(record)
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
        output.append(guarded(emitted, guard))
        # Immutable declarations alias values/types/overload sets. Mutable native
        # storage must be reexported by name resolution, never read into a :: copy.
        if re.match(rf"{re.escape(name)}\s*:(?!:)", code):
            public = f'using,only({name}) #import "internal/net/socket-abi";'
        else:
            public = f"{name} :: ABI.{name};"
        exported.append(guarded(public, guard))
        record["status"] = "native declaration; runtime/ABI verification separate"
        ledger.append(record)
    # Import only external reviewed Windows names needed by these declarations.
    # Wildcard imports make native declarations vulnerable to same-named types
    # (notably the Windows GUI MSG versus the Socket flag enum).
    windows_spec = parse_spec((ROOT / "docs/specs/stdlib/modules/windows.md").read_text())
    external_names = {declaration.name for declaration in windows_spec.declarations}
    local_names = {declaration.name for declaration in spec.declarations}
    used_names = set(re.findall(r"\b[A-Za-z_]\w*\b", '\n'.join(output)))
    dependencies = sorted((used_names & external_names) - local_names)
    windows_import = '#if OS == .WINDOWS {\n    Windows_Native :: #import "Windows";\n'
    windows_import += ''.join(f"    {name} :: Windows_Native.{name};\n" for name in dependencies)
    windows_import += '}\n'
    preamble = """// Generated exclusively from reviewed public declarations.
#scope_file
#if OS_IS_UNIX #import "POSIX";
""" + windows_import + """
libc :: #system_library "libc";
ws2_32 :: #system_library "ws2_32";
#scope_export
#load "socket-native-completions.jai";
"""
    target = output_root / "stdlib/internal/net/socket-abi.jai"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(preamble + "\n".join(output))
    exported.append(guarded("in_addr :: ABI.in_addr;", "OS == .MACOS"))
    facade = output_root / "stdlib/Socket/native.jai"
    facade.parent.mkdir(parents=True, exist_ok=True)
    facade.write_text("// Generated public native aliases.\n" + "\n".join(exported) + '\n#scope_file\nABI :: #import "internal/net/socket-abi";\n')
    (output_root / "docs").mkdir(parents=True, exist_ok=True)
    (output_root / "docs/socket-native-ledger.json").write_text(json.dumps(ledger, indent=2) + "\n")
    subprocess.run([str(formatter), "--config", str(ROOT / "jaifmt.toml"), str(target), str(facade)],
                   cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
