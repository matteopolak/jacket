"""Inspect real SDK import/runtime identities and require both Jai execution modes.

No DLL is downloaded, renamed, copied, substituted or silently skipped. Metadata
inspection is portable; runtime probing requires an actual Windows host.
"""

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess


def dll_name(data):
    name = data.split(b"\0", 1)[0].decode("ascii")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+\.dll", name, re.IGNORECASE):
        raise ValueError("invalid import DLL basename")
    return name


def object_dlls(data):
    """Microsoft short import records or GNU COFF .idata$7 DLL-name sections."""
    if len(data) < 20:
        return set()
    first, second, version, machine = struct.unpack_from("<HHHH", data)
    if first == 0 and second == 0xFFFF:
        if version != 0:  # BigObj is not a short import record.
            return set()
        size = struct.unpack_from("<I", data, 12)[0]
        if size > len(data) - 20:
            raise ValueError("truncated short import data")
        parts = data[20 : 20 + size].split(b"\0")
        if len(parts) < 3 or not parts[0] or not parts[1]:
            raise ValueError("missing short import symbol/DLL terminator")
        if machine not in (0x14C, 0x8664, 0xAA64):
            raise ValueError("unsupported short import machine")
        return {dll_name(parts[1])}
    machine, sections = first, second
    if machine not in (0x14C, 0x8664, 0xAA64):
        return set()
    optional = struct.unpack_from("<H", data, 16)[0]
    start = 20 + optional
    if start + sections * 40 > len(data):
        raise ValueError("truncated COFF section table")
    found = set()
    for index in range(sections):
        header = start + index * 40
        name = data[header : header + 8].rstrip(b"\0")
        if name != b".idata$7":
            continue
        size, pointer = struct.unpack_from("<II", data, header + 16)
        if pointer > len(data) or size > len(data) - pointer:
            raise ValueError("truncated COFF import section")
        payload = data[pointer : pointer + size]
        # Other members have an integer reference to the descriptor here.
        if len(payload) > 4 and payload[0] and b"\0" in payload:
            found.add(dll_name(payload))
    return found


def import_library_dlls(data):
    """Read archive member boundaries; do not infer providers from arbitrary strings."""
    if not data.startswith(b"!<arch>\n"):
        raise ValueError("not a COFF import archive")
    cursor = 8
    result = set()
    while cursor < len(data):
        if len(data) - cursor < 60:
            raise ValueError("truncated archive member header")
        header = data[cursor : cursor + 60]
        if header[58:] != b"`\n":
            raise ValueError("invalid archive member trailer")
        try:
            size = int(header[48:58].decode("ascii").strip())
        except (ValueError, UnicodeDecodeError) as error:
            raise ValueError("invalid archive member length") from error
        cursor += 60
        if size < 0 or size > len(data) - cursor:
            raise ValueError("truncated archive member")
        name = header[:16].rstrip()
        if name not in (b"/", b"//", b"/SYM64/"):
            result.update(object_dlls(data[cursor : cursor + size]))
        cursor += size
        if size % 2:
            if cursor >= len(data) or data[cursor : cursor + 1] != b"\n":
                raise ValueError("missing archive member padding")
            cursor += 1
    return sorted(result, key=str.casefold)


def runtime_probe():
    if os.name != "nt":
        raise RuntimeError("actual Windows required; metadata is not native evidence")
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetModuleFileNameW.argtypes = [
        ctypes.c_void_p,
        ctypes.c_wchar_p,
        ctypes.c_uint32,
    ]
    kernel.GetModuleFileNameW.restype = ctypes.c_uint32
    rows = []
    handles = []  # Retain both real modules for the complete identity check.
    for name in ("d3dcompiler.dll", "d3dcompiler_47.dll"):
        row = {"requested": name}
        try:
            library = ctypes.WinDLL(name, use_last_error=True)
            handles.append(library)
            path = ctypes.create_unicode_buffer(32768)
            length = kernel.GetModuleFileNameW(library._handle, path, len(path))
            if length == 0 or length >= len(path):
                raise ctypes.WinError(ctypes.get_last_error())
            row.update(
                loaded=True,
                path=path.value,
                sha256=hashlib.sha256(Path(path.value).read_bytes()).hexdigest(),
                exports={
                    symbol: ctypes.cast(getattr(library, symbol), ctypes.c_void_p).value
                    for symbol in ("D3DCompile", "D3DCreateBlob")
                },
            )
        except OSError as error:
            row.update(loaded=False, winerror=error.winerror, error=str(error))
        except AttributeError as error:
            row.update(loaded=True, missing_export=str(error))
        rows.append(row)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--import-library", type=Path, action="append", default=[])
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--jaic", type=Path)
    parser.add_argument("--stdlib", type=Path)
    parser.add_argument("--metadata-only", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    args.report.parent.mkdir(parents=True, exist_ok=True)
    report = {"archives": [], "runtime": [], "commands": [], "native_verified": False}
    for path in args.import_library:
        data = path.read_bytes()
        report["archives"].append(
            {
                "path": str(path.resolve()),
                "sha256": hashlib.sha256(data).hexdigest(),
                "dlls": import_library_dlls(data),
            }
        )
    if args.metadata_only:
        if not args.import_library:
            parser.error("--metadata-only requires an actual --import-library")
        args.report.write_text(json.dumps(report, indent=2) + "\n")
        print(args.report)
        return 0
    if os.name != "nt" or not args.jaic or not args.stdlib:
        parser.error(
            "runtime requires Windows, --jaic and --stdlib; no success fallback"
        )
    report["runtime"] = runtime_probe()
    report["compiler"] = {
        "path": str(args.jaic.resolve()),
        "sha256": hashlib.sha256(args.jaic.read_bytes()).hexdigest(),
    }
    work = args.report.parent / (args.report.stem + "-outputs")
    work.mkdir(parents=True, exist_ok=True)
    env = dict(
        os.environ, JAIC_STDLIB=str(args.stdlib.resolve()), TMPDIR=str(work.resolve())
    )
    args.report.write_text(json.dumps(report, indent=2) + "\n")

    def execute(name, mode, command, timeout):
        try:
            result = subprocess.run(
                list(map(str, command)),
                env=env,
                cwd=root,
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            row = dict(
                case=name,
                mode=mode,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=result.returncode == 0
                and not result.stdout
                and not result.stderr,
            )
        except (subprocess.TimeoutExpired, OSError) as error:
            row = dict(case=name, mode=mode, passed=False, failure=str(error))
        report["commands"].append(row)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
        return row["passed"]

    for name in ("windows-import-library-required", "windows-d3dcompiler-provider"):
        source = root / "tests/native-bindings" / (name + ".jai")
        report.setdefault("sources", {})[str(source)] = hashlib.sha256(
            source.read_bytes()
        ).hexdigest()
        exe = work / (name + ".exe")
        commands = [
            ("interp", [args.jaic, source, "-interp"]),
            ("build", [args.jaic, source, "-no_backtrace_on_crash", "-exe", exe]),
        ]
        for mode, command in commands:
            passed = execute(name, mode, command, 120)
            if mode == "build" and passed:
                execute(name, "native", [exe], 30)
    report["native_verified"] = (
        len(report["commands"]) == 6
        and all(row["passed"] for row in report["commands"])
        and all(
            report["runtime"][1].get("exports", {}).get(name)
            for name in ("D3DCompile", "D3DCreateBlob")
        )
    )
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(args.report)
    return 0 if report["native_verified"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
