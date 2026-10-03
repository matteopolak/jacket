#!/usr/bin/env python3
"""Normalize platform API contracts without retaining reference implementations.

The only reference material emitted is declarative ABI information: names,
types, fields, defaults, enum/constant values, imports, and procedure signatures.
Every source procedure body is discarded before output. A missing wrapper is
represented by an explicitly unresolved adapter symbol, never its old body.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path
from author_objective_c_dispatch import SDK as MACOS_SDK, author as author_objective_c
from author_platform_completion import author as author_platform_completion

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "stdlib"
REPORT = OUTPUT / ".coverage/platform-sdk-bindings.json"
SNAPSHOT = ROOT / "target/standard-library-snapshots/9508def6f527169083405db10c93d9d377289fecdca749a546eb849ca39d501d/jai-rs"
GROUPS = ("POSIX", "POSIX_old", "Linux", "macos", "Android", "Objective_C")
FILES = ("Windows.jai", "Windows_Utf8.jai", "Windows_Registry.jai", "Windows_Resources.jai")
SYSTEM_LIBRARIES = {
    "libc", "libm", "libdl", "librt", "libpthread", "libexecinfo", "libobjc", "libSystem",
    "CoreFoundation", "CoreServices", "CoreGraphics", "Foundation", "Cocoa", "AppKit",
    "GameController", "QuartzCore", "libandroid", "libEGL", "liblog", "libaaudio",
    "libGLESv1_CM", "libGLESv2", "libGLESv3", "kernel32", "Ntdll", "winmm", "user32",
    "shell32", "msvcrt", "Gdi32", "opengl32", "Dwmapi", "DbgHelp", "Advapi32",
    "shlwapi", "comdlg32", "advapi32", "ole32", "oleaut32",
}
PROC = re.compile(r"\b([A-Za-z_][A-Za-z_0-9]*)\s*::\s*(?:inline\s+)?\(")
DECL = re.compile(r"\b([A-Za-z_][A-Za-z_0-9]*)\s*(?:::|:(?![=:]))")
LIB = re.compile(r'\b([A-Za-z_][A-Za-z_0-9]*)\s*::\s*#(?:system_library|library)([^;]*)\s*;')
OVERRIDES = ROOT / "tools/platform-sdk-contract-overrides"
POSIX_HELPERS = {"IFTODT", "DTTOIF", "S_ISDIR", "S_ISCHR", "S_ISBLK", "S_ISREG", "S_ISFIFO", "S_ISLNK", "S_ISSOCK", "WSTATUS", "WEXITSTATUS", "WTERMSIG", "WSTOPSIG", "WIFEXITED", "WIFSIGNALED", "WIFSTOPPED", "WIFCONTINUED", "WCOREDUMP"}
INDEPENDENT_LOADS = {
    "POSIX/module.jai": [("errno.jai", {"errno", "set_errno"}), ("file-mode-wait.jai", POSIX_HELPERS)],
    "POSIX_old/module.jai": [("../POSIX/errno.jai", {"errno", "set_errno"})],
    "POSIX_old/POSIX.jai": [("../POSIX/file-mode-wait.jai", POSIX_HELPERS)],
    "POSIX/bindings/linux/stdio.jai": [("../../linux-stat.jai", {"stat", "fstat", "lstat"})],
    "POSIX_old/libc_bindings.jai": [("../POSIX/linux-stat.jai", {"stat", "fstat", "lstat"})],
    "Windows.jai": [("Windows/support.jai", {"SUCCEEDED", "FAILED", "HasOverlappedIoCompleted", "MAKELANGID", "IUnknown_QueryInterface", "IUnknown_AddRef", "IUnknown_Release", "vtable", "safe_release", "safe_release_and_reset", "uid", "assert"})],
    "macos/module.jai": [("string-and-vm.jai", {"to_string", "cfstring_from_string", "VM_MAKE_TAG"})],
    "Linux/io_uring.jai": [("io-uring-syscalls.jai", {"io_uring_setup", "io_uring_enter", "io_uring_register"})],
    "Android/File.jai": [("asset-read.jai", {"read_entire_file"})],
    "Android/Log/module.jai": [("logger.jai", {"android_logger"})],
    "POSIX_old/bindings/linux/io_uring.jai": [("../../../Linux/io-uring-syscalls.jai", {"io_uring_setup", "io_uring_enter", "io_uring_register"})],
    "Windows_Resources.jai": [("Windows/resources.jai", {"MAKEINTRESOURCE", "find_windows_kit_root", "set_icon_by_filename", "set_icon_by_data", "generate_manifest", "add_manifest_to_executable", "disable_runtime_console"})],
}


def declaration_inventory(text: str) -> list[dict]:
    """Count declarations outside parameter lists, after discarding source bodies."""
    mask, _ = mask_non_code(text)
    depths = [0] * (len(mask) + 1)
    depth = 0
    for at, char in enumerate(mask):
        depths[at] = depth
        if char in "([":
            depth += 1
        elif char in ")]":
            depth -= 1
    records = []
    for match in DECL.finditer(mask):
        if depths[match.start()] != 0:
            continue
        suffix = mask[match.end():].lstrip()
        if suffix.startswith(("struct", "union", "enum")):
            kind = "record-or-enum"
        elif suffix.startswith(("(", "inline (")):
            kind = "procedure-or-procedure-field"
        else:
            kind = "constant-alias-or-field"
        records.append({"name": match[1], "kind": kind})
    return records


def mask_non_code(text: str) -> tuple[str, list[tuple[int, int]]]:
    """Return a position-preserving lexer mask and raw-string declaration spans."""
    chars = list(text)
    raw_spans = []
    at = 0
    while at < len(text):
        if text.startswith("//", at):
            end = text.find("\n", at)
            if end < 0:
                end = len(text)
            for i in range(at, end):
                chars[i] = " "
            at = end
        elif text.startswith("/*", at):
            depth, end = 1, at + 2
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            for i in range(at, end):
                if chars[i] != "\n":
                    chars[i] = " "
            at = end
        elif text.startswith("#string", at):
            marker = re.match(r"#string\s+([A-Za-z_][A-Za-z_0-9]*)[^\n]*\n", text[at:])
            if marker:
                close = re.search(r"(?m)^" + re.escape(marker[1]) + r"\s*;?\s*$", text[at + marker.end():])
                if not close:
                    raise ValueError("Unterminated #string")
                end = at + marker.end() + close.end()
                line_start = text.rfind("\n", 0, at) + 1
                raw_spans.append((line_start, end))
                for i in range(at, end):
                    if chars[i] != "\n":
                        chars[i] = " "
                at = end
            else:
                at += 1
        elif text[at] in ('"', "'"):
            quote, end = text[at], at + 1
            while end < len(text):
                if text[end] == "\\":
                    end += 2
                elif text[end] == quote:
                    end += 1
                    break
                else:
                    end += 1
            for i in range(at, min(end, len(text))):
                if chars[i] != "\n":
                    chars[i] = " "
            at = end
        else:
            at += 1
    return "".join(chars), raw_spans


def end_of_body(mask: str, opening: int) -> int:
    depth = 1
    at = opening + 1
    while at < len(mask) and depth:
        if mask[at] == "{":
            depth += 1
        elif mask[at] == "}":
            depth -= 1
        at += 1
    if depth:
        raise ValueError("Unterminated procedure body")
    return at


def find_body(mask: str, opening: int) -> int | None:
    nesting = 0
    for at in range(opening, len(mask)):
        char = mask[at]
        if char in "([":
            nesting += 1
        elif char in ")]":
            nesting -= 1
        elif nesting == 0 and char == ";":
            return None
        elif nesting == 0 and char == "{":
            return at
    return None


def without_comments(text: str) -> str:
    # A lexer removes comments without interpreting literals as comments.
    mask, _ = mask_non_code(text)
    out = list(text)
    at = 0
    while at < len(text):
        if text.startswith("//", at) or text.startswith("/*", at):
            end = at
            while end < len(text) and (mask[end] == " " or text[end] == "\n"):
                out[end] = "\n" if text[end] == "\n" else " "
                end += 1
            at = end
        elif text[at] in ('"', "'"):
            quote = text[at]
            at += 1
            while at < len(text):
                if text[at] == "\\":
                    at += 2
                elif text[at] == quote:
                    at += 1
                    break
                else:
                    at += 1
        else:
            at += 1
    return "".join(out)


def normalize_whitespace(text: str) -> str:
    """Normalize code spacing while retaining exact quoted contract values."""
    pattern = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|\s+')
    return pattern.sub(lambda match: match[0] if match[0][0] in ('"', "'") else " ", text).strip()


def normalize(source: str, relative: Path) -> tuple[str, dict]:
    mask, raw_spans = mask_non_code(source)
    changes = []
    removed = []
    blocked = []
    systems = []
    source_body_ranges = []
    implemented = {name: load for load, names in INDEPENDENT_LOADS.get(str(relative), []) for name in names}
    pos = 0
    while match := PROC.search(mask, pos):
        body = find_body(mask, match.end() - 1)
        if body is None:
            pos = match.end()
            continue
        end = end_of_body(mask, body)
        signature = without_comments(source[match.start():body]).strip()
        signature = re.sub(r"(::\s*)inline\s+", r"\1", signature)
        signature = re.sub(r"\s+#expand\b", "", signature)
        signature = normalize_whitespace(signature)
        source_body_ranges.append((match.start(), end, signature + ";"))
        symbol = "jai_stdlib_" + re.sub(r"\W", "_", str(relative)) + "_" + match[1] + "_" + hashlib.sha256(signature.encode()).hexdigest()[:12]
        if match[1] in implemented:
            annotation = re.match(r"\s*@NoProfile\b", source[end:])
            if annotation:
                end += annotation.end()
            changes.append((match.start(), end, ""))
            removed.append({"name": match[1], "signature": signature, "line": source.count("\n", 0, match.start()) + 1, "status": "independent-jai-implementation", "implementation": str(relative.parent / implemented[match[1]])})
        else:
            changes.append((match.start(), end, signature + ' #foreign Native_Adapters "' + symbol + '";'))
            removed.append({"name": match[1], "signature": signature, "line": source.count("\n", 0, match.start()) + 1, "status": "unimplemented-adapter-contract", "adapter_symbol": symbol})
        pos = end
    for start, end in raw_spans:
        if not any(a <= start < b for a, b, _ in changes):
            changes.append((start, end, ""))
            blocked.append({"line": source.count("\n", 0, start) + 1, "reason": "Reference implementation raw-string template excluded"})
    for match in LIB.finditer(source):
        if any(a <= match.start() < b for a, b, _ in changes):
            continue
        literal = re.search(r'"([^\"]+)"', match[2])
        name = literal[1] if literal else ""
        if name in SYSTEM_LIBRARIES:
            changes.append((match.start(), match.end(), match[1] + ' :: #system_library "' + name + '";'))
            systems.append({"alias": match[1], "library": name, "kind": "trusted-system-sdk"})
        else:
            changes.append((match.start(), match.end(), ""))
            blocked.append({"line": source.count("\n", 0, match.start()) + 1, "reason": "Non-SDK library excluded", "alias": match[1], "library": name})
    # Compile-time generated methods are implementations, not ABI contracts.
    for match in re.finditer(r"(?m)^\s*#insert\s+#run[^;]*;", mask):
        if not any(a <= match.start() < b for a, b, _ in changes):
            changes.append((match.start(), match.end(), ""))
            blocked.append({"line": source.count("\n", 0, match.start()) + 1, "reason": "Compile-time implementation insertion excluded"})
    # References to excluded shipped libraries cannot silently survive.
    excluded_aliases = [b["alias"] for b in blocked if "alias" in b]
    for alias in excluded_aliases:
        for match in re.finditer(r"\b([A-Za-z_]\w*)\s*::[^;{}]*#foreign\s+" + re.escape(alias) + r"\b[^;]*;", source):
            if not any(a <= match.start() < b for a, b, _ in changes):
                changes.append((match.start(), match.end(), ""))
                blocked.append({"line": source.count("\n", 0, match.start()) + 1, "name": match[1], "reason": "Foreign declaration used excluded shipped library"})
        for match in re.finditer(r"(?m)^\s*([A-Za-z_]\w*)\s*:[^;]*#elsewhere\s+" + re.escape(alias) + r"\b[^;]*;", source):
            if not any(a <= match.start() < b for a, b, _ in changes):
                changes.append((match.start(), match.end(), ""))
                blocked.append({"line": source.count("\n", 0, match.start()) + 1, "name": match[1], "reason": "Imported data used excluded shipped library"})
    changes.sort(reverse=True)
    out = source
    for start, end, replacement in changes:
        out = out[:start] + replacement + out[end:]
    out = without_comments(out)
    adaptations = []
    if str(relative) in {"POSIX/bindings/macos/arm64/base.jai", "POSIX/bindings/android/arm64/base.jai"}:
        out = out.replace('#import "Basic";', '')
        adaptations.append("Removed unused Basic import from pure SDK declaration file to avoid platform utility dependency cycles")
    if str(relative) == "POSIX/module.jai":
        switch = re.search(r"#if\s+OS\s*==\s*\{", out)
        if switch:
            opening = out.find("{", switch.start())
            closing = end_of_body(mask_non_code(out)[0], opening)
            sections = re.split(r"case\s+\.(LINUX|MACOS|ANDROID)\s*;", out[opening + 1:closing - 1])
            branches = []
            for index in range(1, len(sections), 2):
                branches.append(("#if" if not branches else "else #if") + " OS == ." + sections[index] + " {" + sections[index + 1] + "}")
            out = out[:switch.start()] + "\n".join(branches) + out[closing:]
            adaptations.append("Replaced declarative OS-selection switch with equivalent ordinary conditional load branches for frozen module publication")
    if str(relative) == "Android/module.jai":
        out = re.sub(r"#program_export\s*", "", out)
        adaptations.append("Removed program-export annotation from unresolved bodyless Android entry point")
    if relative.parent == Path("Objective_C"):
        out = re.sub(r"@deprecated\b", "", out)
        out = re.sub(r"_sel\s*:\s*struct\s+#type_info_no_size_complaint", "_Selector_Table :: struct #type_info_no_size_complaint", out)
        if "_Selector_Table ::" in out:
            out = out.replace("_Selector_Table :: struct #type_info_no_size_complaint", "_Selector_Table :: struct")
            out += "\n_sel: _Selector_Table;\n"
            adaptations.append("Named the private selector-table record and omitted unsupported advisory reflection size flag for frozen parser compatibility; table fields retained")
    if str(relative) == "Objective_C/module.jai":
        out = out.replace('#import "Basic";', 'Basic :: #import "Basic";\n#import "Basic";')
        adaptations.append("Added explicit Basic qualifier for independently authored allocation bridges")
    if str(relative) in {"Objective_C/AppKit.jai", "Objective_C/Foundation.jai", "Objective_C/GameController.jai"}:
        out = re.sub(r'(\benum(?:_flags)?\s+)NSUInteger\b', r'\1u64', out)
        out = re.sub(r'(\benum(?:_flags)?\s+)NSInteger\b', r'\1s64', out)
        adaptations.append("Expanded reviewed LP64 NSUInteger/NSInteger enum backing aliases to equivalent u64/s64 representation because frozen early enum discovery cannot resolve imported backing aliases; alias declarations and nominal enums retained")
    if str(relative) == "Objective_C/AppKit.jai":
        out = out.replace("NSModalResponse :: NSInteger;", "NSModalResponse :: s64;").replace("NSWindowLevel :: NSInteger;", "NSWindowLevel :: s64;")
        adaptations.append("Expanded NSModalResponse and NSWindowLevel LP64 aliases to SDK-equivalent s64 for frozen imported-alias discovery")
        events = dict(re.findall(r'\b(NSEventType\w+)\s*::\s*(\d+)\s*;', out))
        out = re.sub(r'1\s*<<\s*(NSEventType\w+)\b', lambda m: '1 << ' + events[m[1]], out)
        out = re.sub(r'\bNSUIntegerMax\b', '0xFFFFFFFFFFFFFFFF', out)
        adaptations.append("Expanded NSEventMask shift operands and NSUIntegerMax enum operands to their configured SDK-verified integer values; frozen enum discovery cannot yet resolve sibling enum members or imported constants")
    if str(relative) == "Objective_C/GameController.jai":
        out = out.replace("GCControllerPlayerIndexUnset :: 0xFFFFFFFFFFFFFFFF;", "GCControllerPlayerIndexUnset :: -1;")
        adaptations.append("Corrected GCControllerPlayerIndexUnset to configured SDK signed NSInteger -1")
    if str(relative) == "Windows.jai":
        out = re.sub(r"Internal\s*:\s*u64\s*;\s*#place Internal\s*;\s*Status\s*:\s*u64\s*;", "union { Internal: u64; Status: u64; }", out)
        out = re.sub(r"InternalHigh\s*:\s*u64\s*;\s*#place InternalHigh\s*;\s*NumberOfBytesTransferred\s*:\s*u64\s*;", "union { InternalHigh: u64; NumberOfBytesTransferred: u64; }", out)
        out = re.sub(r"dwLowDateTime\s*:\s*u32\s*;\s*dwHighDateTime\s*:\s*u32\s*;\s*#place dwLowDateTime\s*;\s*QuadPart\s*:\s*u64\s*=\s*---\s*#align 4\s*;", "union { struct { dwLowDateTime: u32; dwHighDateTime: u32; } QuadPart: u64 #align 4 = ---; }", out)
        out = out.replace("dwFlags: s32, lpWideCharStr", "dwFlags: u32, lpWideCharStr").replace("dwFlags: s32, lpMultiByteStr", "dwFlags: u32, lpMultiByteStr")
        out = out.replace('SetCursor :: (cursor: HCURSOR) #foreign user32;', 'SetCursor :: (cursor: HCURSOR) -> HCURSOR #foreign user32;')
        adaptations.append("Corrected SetCursor native return to SDK HCURSOR; the return is the previous cursor handle")
        adaptations.append("Expressed OVERLAPPED and FILETIME placement aliases as anonymous unions; corrected encoding dwFlags to SDK DWORD")
    if str(relative) == "Windows_Registry.jai":
        out = re.sub(r"cast\(HKEY\)\s*(0x800000[0-9a-fA-F]{2})", r"cast(HKEY) (cast(s64) (cast,trunc(s32) \1))", out)
        adaptations.append("Sign-extended predefined registry handle constants through SDK LONG to pointer width")
    if str(relative) == "Windows_Resources.jai":
        out = out.replace('lpType: *u8, lpName: *u16', 'lpType: *u16, lpName: *u16')
        for name, value in {"COINIT_APARTMENTTHREADED": 2, "COINIT_MULTITHREADED": 0, "COINIT_DISABLE_OLE1DDE": 4, "COINIT_SPEED_OVER_MEMORY": 8}.items():
            out = re.sub(r"\b" + name + r"\s*::\s*\d+\s*;", name + " :: " + str(value) + ";", out)
        adaptations.append("Corrected UpdateResourceW resource type parameter to SDK LPCWSTR and COINIT values to current objbase.h contract")
    if str(relative) == "macos/module.jai":
        out = re.sub(r"FSEventStreamContext\s*::\s*struct\s*\{[^{}]*\}", "FSEventStreamContext :: struct { version: CFIndex; info: *void; retain: CFAllocatorRetainCallBack; release: CFAllocatorReleaseCallBack; copyDescription: CFAllocatorCopyDescriptionCallBack; }", out)
        adaptations.append("Corrected FSEventStreamContext field order to configured macOS 27.0 SDK header")
    if str(relative) == "POSIX_old/POSIX.jai":
        clock = re.compile(r"using clockid_t\s*::\s*enum s32\s*\{\s*CLOCK_REALTIME\s*::\s*0\s*;\s*#if OS == \.LINUX\s*\{(.*?)\}\s*else #if OS == \.MACOS\s*\{(.*?)\}\s*\}", re.S)
        out = clock.sub(lambda m: "#if OS == .LINUX { using clockid_t :: enum s32 { CLOCK_REALTIME :: 0;" + m[1] + "} } else #if OS == .MACOS { using clockid_t :: enum s32 { CLOCK_REALTIME :: 0;" + m[2] + "} }", out)
        adaptations.append("Moved platform selection outside legacy clockid_t enum; values retained")
    contract_source = source
    for start, end, signature in reversed(source_body_ranges):
        contract_source = contract_source[:start] + signature + contract_source[end:]
    inventory = declaration_inventory(contract_source)
    output_inventory = declaration_inventory(out)
    output_inventory.extend({"name": body["name"], "kind": "independently-implemented-procedure"} for body in removed if body["status"] == "independent-jai-implementation")
    lines = [normalize_whitespace(line) for line in out.splitlines()]
    normalized = "\n".join(line for line in lines if line)
    # Retain readable nesting without carrying original formatting.
    depth = 0
    formatted = []
    for line in normalized.splitlines():
        line_mask, _ = mask_non_code(line)
        initial_close = int(line_mask.startswith("}"))
        formatted.append("    " * max(0, depth - initial_close) + line)
        depth = max(0, depth + line_mask.count("{") - line_mask.count("}"))
    out = "// Normalized API contracts; reference procedure bodies were discarded.\n" + "\n".join(formatted) + "\n"
    for load, _ in INDEPENDENT_LOADS.get(str(relative), []):
        out += '\n#scope_export\n#load "' + load + '";\n'
    if any(record["status"] == "unimplemented-adapter-contract" for record in removed):
        out += '\n#scope_file\nNative_Adapters :: #system_library "jai-stdlib-native-adapters";\n'
    report = {
        "source_lines": len(source.splitlines()),
        "source_api_declaration_occurrences": len(inventory),
        "api_inventory": inventory,
        "normalized_output_contract_declarations": len(output_inventory),
        "source_procedure_bodies_discarded": len(removed),
        "system_foreign_declarations": len(re.findall(r"#foreign\s+(?!Native_Adapters\b)", mask_non_code(out)[0])),
        "trusted_system_libraries": systems,
        "unimplemented_adapters": removed, "excluded": blocked,
        "reference_body_reuse": False, "behavior_verified": False,
        "syntax_or_sdk_adaptations": adaptations,
    }
    return out, report


def paths():
    all_paths = []
    for group in GROUPS:
        all_paths.extend(sorted((REFERENCE / group).rglob("*.jai")))
    all_paths.extend(REFERENCE / name for name in FILES)
    return all_paths


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parse", action="store_true", help="Parse emitted sources using the frozen rewrite snapshot")
    args = parser.parse_args()
    if shutil.disk_usage(ROOT).free < 2 * 1024 ** 3:
        raise SystemExit("Disk floor: less than 2 GiB free")
    records, excluded_files = [], []
    for path in paths():
        relative = path.relative_to(REFERENCE)
        if any(part in ("examples", "tests") for part in relative.parts) or relative.name in ("tests.jai", "build.jai", "compare_bindings.jai") or relative.name.startswith("generate"):
            continue
        source = path.read_text()
        out, record = normalize(source, relative)
        out, dispatch, dispatch_gaps = author_objective_c(str(relative), out)
        if dispatch:
            record["independent_objective_c_dispatch"] = dispatch
            by_symbol = {item["adapter_symbol"]: item for item in dispatch}
            for adapter in record["unimplemented_adapters"]:
                if adapter.get("adapter_symbol") in by_symbol:
                    adapter["status"] = "independent-jai-implementation"
                    adapter["implementation"] = "tools/author_objective_c_dispatch.py"
                    adapter["dispatch"] = by_symbol[adapter["adapter_symbol"]]
                    adapter.pop("adapter_symbol", None)
        if dispatch_gaps:
            record["objective_c_dispatch_gaps"] = dispatch_gaps
        out, completion = author_platform_completion(str(relative), out)
        if completion:
            record["independent_platform_completion"] = completion
            by_symbol = {item["adapter_symbol"]: item for item in completion}
            for adapter in record["unimplemented_adapters"]:
                if adapter.get("adapter_symbol") in by_symbol:
                    implementation = by_symbol[adapter["adapter_symbol"]]
                    adapter["status"] = "independent-jai-implementation"
                    adapter["implementation"] = implementation["implementation"]
                    adapter.pop("adapter_symbol", None)
            record["system_foreign_declarations"] = len(re.findall(r"#foreign\s+(?!Native_Adapters\b)", mask_non_code(out)[0]))
        override = OVERRIDES / relative
        if override.exists():
            out = override.read_text()
            record["independent_full_file_override"] = str(override.relative_to(ROOT))
            for adapter in record["unimplemented_adapters"]:
                adapter["status"] = "independent-jai-implementation"
                adapter["implementation"] = record["independent_full_file_override"]
                adapter.pop("adapter_symbol", None)
        dest = OUTPUT / relative
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(out)
        record["output_sha256"] = hashlib.sha256(out.encode()).hexdigest()
        if args.parse:
            result = subprocess.run([str(SNAPSHOT), "parse", str(dest)], capture_output=True, text=True, timeout=30)
            record["parse"] = {"success": result.returncode == 0, "exit_code": result.returncode, "diagnostic": (result.stdout + result.stderr)[-2500:] if result.returncode else ""}
        records.append(record)
    for path in sorted(OVERRIDES.rglob("*.jai")):
        relative = path.relative_to(OVERRIDES)
        if any(record["output"] == "stdlib/" + str(relative) for record in records):
            continue
        dest = OUTPUT / relative
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(path.read_text())
        record = {"source": str(path.relative_to(ROOT)), "output": "stdlib/" + str(relative), "source_lines": 0, "source_api_declaration_occurrences": 0, "source_procedure_bodies_discarded": 0, "system_foreign_declarations": len(re.findall(r"#foreign\s+(?!Native_Adapters\b)", mask_non_code(path.read_text())[0])), "unimplemented_adapters": [], "status": "independent-jai-implementation", "behavior_verified": False, "output_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        if args.parse:
            result = subprocess.run([str(SNAPSHOT), "parse", str(dest)], capture_output=True, text=True, timeout=30)
            record["parse"] = {"success": result.returncode == 0, "exit_code": result.returncode, "diagnostic": (result.stdout + result.stderr)[-2500:] if result.returncode else ""}
        records.append(record)
    report = {
        "schema_version": 1, "status": "declarative-sdk-contracts-with-independent-wrappers-native-unverified",
        "reference_access": "Read-only source API contracts; no source procedures or shipped native files were executed, loaded, linked, or copied",
        "extractor": "tools/rewrite_platform_sdk_contracts.py",
        "frozen_parser": str(SNAPSHOT.relative_to(ROOT)), "frozen_parser_sha256": hashlib.sha256(SNAPSHOT.read_bytes()).hexdigest(), "native_behavior_verified": False, "build_inputs_verified": False,
        "source_check_evidence": "stdlib/.coverage/platform-sdk-source-checks.json",
        "authoritative_sources": [
            {"scope": "Configured macOS SDK contract spot-check", "source": "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk", "sdk_version": "27.0", "verified": ["errno TLS entry", "LP64 pthread mutex/condition opaque sizes", "stat64 field order", "timespec/timeval scalar widths", "wait-status macro semantics", "FSEventStreamContext field order", "VM_MAKE_TAG shift"], "not_verified": "No complete header-to-binding ABI comparison or native linkage test"},
            {"scope": "Windows encoding APIs", "source": "https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar", "verified": ["DWORD flags", "query-before-allocation", "explicit versus terminated length", "Kernel32 SDK mapping"]},
            {"scope": "Windows encoding APIs", "source": "https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte", "verified": ["UTF8 null default-character arguments", "length includes terminator for -1 input", "Kernel32 SDK mapping"]},
            {"scope": "Windows registry", "source": "https://raw.githubusercontent.com/microsoft/win32metadata/main/generation/WinSDK/RecompiledIdlHeaders/um/winreg.h", "verified": ["sign-extended predefined handles", "LSTATUS/REGSAM widths", "RegQueryValueExW signature"]},
            {"scope": "Windows registry reads", "source": "https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw", "verified": ["byte-count query", "caller must supply termination for nonterminated values"]},
            {"scope": "Darwin wait macros", "source": "https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/bsd/sys/wait.h", "verified": ["continued/stopped distinction", "128 core flag"]},
            {"scope": "Linux wait macros", "source": "https://raw.githubusercontent.com/bminor/glibc/master/bits/waitstatus.h", "verified": ["exit signal/stop masks", "65535 continued marker"]},
            {"scope": "Linux public libc stat wrappers", "source": "https://raw.githubusercontent.com/bminor/glibc/master/sysdeps/unix/sysv/linux/stat.c", "verified": ["public stat returns C int", "independent wrapper preserves supplied s64 return width"], "not_verified": "Runtime symbol availability and historical Linux x64 stat_t ABI were not tested; public stat family requires modern libc (glibc 2.33+ on glibc targets)"},
            {"scope": "Linux public libc stat symbol availability", "source": "https://raw.githubusercontent.com/bminor/glibc/master/io/Versions", "verified": ["public stat/fstat/lstat version node GLIBC_2.33"]},
            {"scope": "Linux io_uring", "source": "https://raw.githubusercontent.com/torvalds/linux/master/include/uapi/linux/io_uring.h", "verified": ["io_uring_getevents_arg layout and extended-argument contract"], "not_verified": "Remaining source-compatible io_uring structs/enums are a historical API subset, not a full current Linux UAPI declaration set"},
            {"scope": "Linux io_uring invocation", "source": "https://man7.org/linux/man-pages/man2/io_uring_enter.2.html", "verified": ["extended-argument flag and size"]},
            {"scope": "Objective-C messaging", "source": str(MACOS_SDK / "usr/include/objc/message.h"), "sdk_version": "27.0", "verified": ["fixed correctly typed function pointer dispatch", "objc_msgSend_stret unavailable on arm64", "reviewed plain aggregate x64 structure-return selection"], "not_verified": "No complete per-method SDK signature comparison, compiler native ABI check, or runtime execution"},
            {"scope": "X64 reviewed plain aggregate ABI", "source": "https://raw.githubusercontent.com/llvm/llvm-project/main/clang/lib/CodeGen/Targets/X86.cpp", "verified": ["Clang record classifier sends reviewed aligned non-vector multi-field records larger than 128 bits to Memory"], "not_verified": "Jai C-call ABI lowering and original/native wrapper execution have not been validated"},
            {"scope": "Objective-C SDK method corrections", "source": str(MACOS_SDK / "System/Library/Frameworks"), "sdk_version": "27.0", "verified": ["NSView addSubview selector spelling", "four GameController snapshot instancetype pointer returns", "NSApplication/NSWindow/NSOpenGLContext void methods retain compatible Jai null result", "NSOpenGLView openGLContext property and NSOpenGLContext flushBuffer"], "not_verified": "Original custom view subclass callbacks and Metal-layer behavior are not recreated"},
            {"scope": "Windows resource update transaction", "source": "https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-updateresourcew", "verified": ["LPCWSTR resource type and name", "resource type identifiers", "SDK transaction accumulates before commit"]},
            {"scope": "Windows resource commit and rollback", "source": "https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-endupdateresourcew", "verified": ["fDiscard TRUE rolls back", "fDiscard FALSE commits", "native result checked"]},
            {"scope": "Windows ICO resource encoding", "source": "https://devblogs.microsoft.com/oldnewthing/20120720-00/?p=7083", "verified": ["group icon image offset replaced by WORD resource identifier", "separate RT_ICON images"]},
            {"scope": "Windows application manifests", "source": "https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests", "verified": ["assembly identity fields", "UAC execution level", "DPI and SegmentHeap namespaces"], "not_verified": "Generated manifest and executable resource edits were not exercised on Windows"},
            {"scope": "Windows runtime console", "source": "https://learn.microsoft.com/windows/console/freeconsole", "verified": ["SDK FreeConsole detaches current process"]},
            {"scope": "Objective-C runtime reflection and encoding", "source": "https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html", "verified": ["native scalar, pointer, object, class, selector, fixed-array and record encoding forms", "runtime class/method/ivar APIs reviewed in configured SDK objc/runtime.h"], "not_verified": "Native class registration, Jai reflection constant storage and generated method stubs have not been executed"},
            {"scope": "EGL context creation", "source": "https://registry.khronos.org/EGL/specs/eglspec.1.5.pdf", "verified": ["ES 2 window configuration and context attributes", "chosen sample-count query", "failed context/surface/display cleanup"], "not_verified": "No Android EGL driver was run"},
            {"scope": "Visual Studio Setup Configuration", "source": "https://raw.githubusercontent.com/Kitware/CMake/master/Utilities/cmvssetup/Setup.Configuration.h", "authority": "Microsoft-authored Setup Configuration SDK header distributed by maintained CMake", "verified": ["COM class UUID and configuration/instance/enumerator vtable contracts"], "not_verified": "Visual Studio discovery was not executed on Windows"},
            {"scope": "Visual Studio MSVC path discovery", "source": "https://learn.microsoft.com/en-us/cpp/overview/acquire-msvc?view=msvc-170", "verified": ["VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt", "VC/Tools/MSVC versioned tools directory"], "not_verified": "Only modern versioned MSVC layouts are supported; VS 2015 and older layouts require a separate implementation"},
            {"scope": "Windows COM initialization", "source": "https://learn.microsoft.com/en-us/windows/win32/api/objbase/ne-objbase-coinit", "verified": ["COINIT SDK constant values", "balanced successful CoInitializeEx", "existing apartment retained on RPC_E_CHANGED_MODE"]},
        ],
        "abi_verification_boundary": "Source API compatibility retained for the supplied target branches. Configured macOS SDK and selected maintained online declarations were spot-checked; remaining Linux, Android, Windows, Objective-C and macOS layouts have no blanket current-ABI verification.",
        "files": records, "excluded_files": excluded_files,
        "totals": {
            "output_files": len(records), "excluded_source_files": len(excluded_files),
            "source_lines": sum(r["source_lines"] for r in records),
            "source_api_declaration_occurrences": sum(r["source_api_declaration_occurrences"] for r in records),
            "system_foreign_declarations": sum(r["system_foreign_declarations"] for r in records),
            "source_procedure_bodies_discarded": sum(r["source_procedure_bodies_discarded"] for r in records),
            "independent_jai_implementations": sum(a["status"] == "independent-jai-implementation" for r in records for a in r["unimplemented_adapters"]),
            "unimplemented_adapter_contracts": sum(a["status"] == "unimplemented-adapter-contract" for r in records for a in r["unimplemented_adapters"]),
            "parse_passed": sum(r.get("parse", {}).get("success", False) for r in records),
        },
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report["totals"], indent=2))


if __name__ == "__main__":
    main()
