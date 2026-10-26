#!/usr/bin/env python3
"""Build the inputs of the portable (cross-host) stdlib checks: test fixtures written by independent code, and
the small native helper libraries built from our own C sources. One subcommand per output:

    build_portable.py zip | archive | mpeg | machine         # fixtures under target/portable-tests/
    build_portable.py meow [--verify-native]                 # stage (and optionally re-derive) the Meow vectors
    build_portable.py clipboard --output DIR                 # Linux X11 clipboard service (libX11)
    build_portable.py runtime [--platform P] [--output-dir D]  # private POSIX crash libraries
    build_portable.py watcher [--output-dir D]               # macOS FSEvents backend

None of them fetches sources or runs a supplied reference binary. `runtime` and `watcher` are not steps of
the normal build: `crates/jaic-cli/build.rs` embeds the same backends in jaic (docs/native/crash-runtime-build.md);
these are the same recipes for tools that need the libraries (and a hash receipt) as files.
"""

import argparse
import functools
import json
import os
import platform
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

from repo_output import host_platform
from toolkit.hashing import sha256_bytes, sha256_file
from toolkit.repo import ROOT, cargo_target_dir
from repo_output import output_dir

ARCHIVE_OUT = output_dir("portable-tests", "archive-fixtures")


def member(name, body):
    name = name.encode()
    assert len(name) <= 16
    header = (
        name.ljust(16)
        + b"0".ljust(12)
        + b"0".ljust(6)
        + b"0".ljust(6)
        + b"100644".ljust(8)
        + str(len(body)).encode().ljust(10)
        + b"`\n"
    )
    assert len(header) == 60
    return header + body + (b"\n" if len(body) & 1 else b"")


def archive_fixtures():
    """Build original ar wire fixtures and a genuine host-tool C object archive."""
    ARCHIVE_OUT.mkdir(parents=True, exist_ok=True)
    magic = b"!<arch>\n"
    basic = member("short.o/", b"abc")
    (ARCHIVE_OUT / "classic.a").write_bytes(magic + basic)
    extended = b"long member name.o"
    bsd_member = member(f"#1/{len(extended)}", extended + b"abc")
    for big in [False, True]:
        endian = ">" if big else "<"
        symbol = b"answer\0"
        length = 4 + 8 + 4 + len(symbol)
        # Extended index names also exercise NUL padding in the name prefix.
        index_name = b"__.SYMDEF\0\0\0"
        target = 8 + len(member(f"#1/{len(index_name)}", index_name + b"\0" * length))
        index = struct.pack(endian + "IIII", 8, 0, target, len(symbol)) + symbol
        data = magic + member(f"#1/{len(index_name)}", index_name + index) + bsd_member
        (ARCHIVE_OUT / f"bsd-{'big' if big else 'little'}.a").write_bytes(data)
        if not big:
            corrupt = bytearray(data)
            start = 8 + 60 + len(index_name) + 4
            corrupt[start : start + 4] = struct.pack("<I", len(symbol) + 1)
            (ARCHIVE_OUT / "bad-bsd-string.a").write_bytes(corrupt)
    for coff in [False, True]:
        long_table = extended + (b"\0" if coff else b"/\n")
        names = b"answer\0alias\0"
        first_size = 4 + 8 + len(names)
        second_size = 4 + 4 + 4 + 4 + len(names)
        target = (
            8 + len(member("/", b"\0" * first_size)) + len(member("//", long_table))
        )
        if coff:
            target += len(member("/", b"\0" * second_size))
        first = struct.pack(">III", 2, target, target) + names
        data = magic + member("/", first)
        if coff:
            second = struct.pack("<IIIHH", 1, target, 2, 1, 1) + names
            data += member("/", second)
        data += member("//", long_table) + member("/0", b"abc")
        label = "coff" if coff else "gnu"
        (ARCHIVE_OUT / f"{label}.a").write_bytes(data)
        corrupt = bytearray(data)
        corrupt[72:76] = b"\xff" * 4  # First symbol points outside the archive.
        (ARCHIVE_OUT / f"bad-{label}-offset.a").write_bytes(corrupt)
        corrupt = bytearray(data)
        if coff:
            second_start = 8 + len(member("/", first)) + 60
            corrupt[second_start + 12 : second_start + 14] = b"\0\0"
            (ARCHIVE_OUT / "bad-coff-index.a").write_bytes(corrupt)
        else:
            corrupt[68:72] = b"\xff" * 4
            (ARCHIVE_OUT / "bad-gnu-count.a").write_bytes(corrupt)
            corrupt = bytearray(data)
            corrupt[68 + first_size - 1] = ord("X")
            (ARCHIVE_OUT / "bad-symbol-termination.a").write_bytes(corrupt)
    (ARCHIVE_OUT / "bad-size.a").write_bytes(
        magic + basic[:48] + b"9999999999" + basic[58:]
    )
    (ARCHIVE_OUT / "bad-padding.a").write_bytes(magic + basic[:-1] + b"x")
    (ARCHIVE_OUT / "bad-trailer.a").write_bytes(magic + basic[:58] + b"xx" + basic[60:])
    (ARCHIVE_OUT / "bad-name.a").write_bytes(magic + member("/0", b"abc"))
    (ARCHIVE_OUT / "unsupported64.a").write_bytes(magic + member("/SYM64/", b""))
    source = ARCHIVE_OUT / "original.c"
    source.write_text("int portable_archive_answer(void) { return 42; }\n")
    compiler = shutil.which("clang") or shutil.which("cc")
    # GNU ar does not index objects of a foreign architecture (such as ARM64 COFF); llvm-ar does.
    archive_tool = shutil.which("llvm-ar") or shutil.which("ar")
    if not compiler or not archive_tool:
        raise RuntimeError("source-built archive fixture requires a C compiler and ar")
    subprocess.run(
        [compiler, "-c", str(source), "-o", str(ARCHIVE_OUT / "original.o")], check=True
    )
    native = ARCHIVE_OUT / "native.a"
    native.unlink(missing_ok=True)
    subprocess.run(
        [archive_tool, "rcs", str(native), str(ARCHIVE_OUT / "original.o")], check=True
    )


def zip_fixtures():
    """Construct independent classic ZIP fixtures using Python's archive writer."""
    directory = output_dir("portable-tests", "zip")
    directory.mkdir(parents=True, exist_ok=True)
    valid = directory / "valid.zip"
    with zipfile.ZipFile(valid, "w", compression=zipfile.ZIP_STORED) as archive:
        archive.writestr("first.txt", b"hello")
        archive.writestr("binary.bin", bytes([0, 128, 255]))
        archive.comment = b"comment with false PK\x05\x06 marker"
    with zipfile.ZipFile(directory / "empty.zip", "w"):
        pass
    original = valid.read_bytes()
    end = original.index(b"PK\x05\x06")
    central = struct.unpack_from("<I", original, end + 16)[0]
    variants = {}
    variants["truncated"] = bytearray(original[:15])
    for name in [
        "bounds",
        "zip64",
        "multidisk",
        "count",
        "late-name",
        "signature",
        "descriptor",
    ]:
        variants[name] = bytearray(original)
    struct.pack_into("<I", variants["bounds"], end + 16, len(original))
    struct.pack_into("<I", variants["zip64"], central + 24, 0xFFFFFFFF)
    struct.pack_into("<H", variants["multidisk"], end + 4, 1)
    struct.pack_into("<HH", variants["count"], end + 8, 3, 3)
    second = original.index(b"PK\x01\x02", central + 4)
    struct.pack_into("<H", variants["late-name"], second + 28, 65535)
    struct.pack_into("<I", variants["signature"], central, 0x12345678)
    struct.pack_into("<H", variants["descriptor"], central + 8, 8)
    for name, data in variants.items():
        (directory / f"{name}.zip").write_bytes(data)


CPUID_SOURCE = ROOT / "stdlib/Machine_X64/tests/native/cpuid-features.c"


def machine_fixtures():
    """Write target/portable-machine/cpuid.bin, the CPUID words Machine_X64/tests/basics/main.jai compares with the
    Jai reading: the output of the independent C program stdlib/Machine_X64/tests/native/cpuid-features.c, run on this
    machine. Only x86-64 hosts have CPUID to read; other hosts write nothing (the test is x64-only too)."""
    if platform.machine().lower() not in ("x86_64", "amd64") or sys.platform == "win32":
        return 0
    output = output_dir("portable-machine", "cpuid.bin")
    output.parent.mkdir(parents=True, exist_ok=True)
    compiler = os.environ.get("CC", "cc")
    with tempfile.TemporaryDirectory() as scratch:
        executable = Path(scratch) / "cpuid-features"
        subprocess.run(
            [
                compiler,
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                str(CPUID_SOURCE),
                "-o",
                str(executable),
            ],
            check=True,
        )
        words = subprocess.run(
            [str(executable)], check=True, capture_output=True
        ).stdout
    if len(words) != 48:
        raise RuntimeError(f"the CPUID oracle wrote {len(words)} bytes, expected 48")
    output.write_bytes(words)
    return 0


def meow_fixtures(verify_native):
    """Stage genuine Meow vectors produced by our licensed native C source build; --verify-native rebuilds that
    independent comparison and requires byte-identical results."""
    header = ROOT / "third_party/meow_hash/meow_hash_x64_aesni.h"
    assert (
        sha256_bytes(header.read_bytes().replace(b"\r\n", b"\n"))
        == "2328557212adc48ee3e8e8c9d25820a43b2fc0c48cf4296ab9b0b1ba52d376e2"
    )
    fixture = ROOT / "stdlib/meow_hash/tests/fixtures/vectors.bin"
    data = fixture.read_bytes()
    assert (
        sha256_bytes(data)
        == "b4b8af0a8629e1cdc1a9853486158126e7e5756e72a062682e4eb354765ff639"
    )
    (count,) = struct.unpack_from("<I", data, 4)
    assert data[:4] == b"MCV5" and count == 317 and len(data) == 136 + count * 304
    output = output_dir("portable-tests", "meow-fixtures")
    output.mkdir(parents=True, exist_ok=True)
    if verify_native:
        command = ["clang++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-O2"]
        if platform.system() == "Darwin":
            command += ["-arch", "x86_64"]
        elif platform.machine().lower() not in {"x86_64", "amd64"}:
            raise SystemExit(
                "native AES-NI comparison needs x64 or macOS x64 execution support"
            )
        executable = output / "native-c-vectors"
        command += [
            "-maes",
            "-msse4.1",
            "-mssse3",
            str(ROOT / "stdlib/meow_hash/tests/native/c-check.cpp"),
            "-o",
            str(executable),
        ]
        subprocess.run(command, check=True)
        actual = subprocess.check_output([str(executable)], timeout=45)
        assert actual == data, "native C comparison differs from the reviewed fixture"
        print("Source-built native C reproduced 317 Meow vectors byte-identically")
    shutil.copyfile(fixture, output / "vectors.bin")


@functools.cache
def _declarations():
    spec = (ROOT / "docs/specs/stdlib/modules/pl-mpeg.md").read_text()
    appendix = (ROOT / "docs/specs/stdlib/modules/pl-mpeg-public-tables.md").read_text()
    declarations = {}
    for line in spec.splitlines():
        match = re.match(r"- `([A-Z_]+ :: Vlc\.\[.*?)`", line)
        if match:
            declarations[match[1].split()[0]] = match[1]
    declarations["DCT_COEFF"] = re.search(
        r"```jai\n(DCT_COEFF.*?);\n```", appendix, re.S
    )[1]
    return declarations


def integer(expression):
    parts = expression.strip().split("<<")
    value = int(parts[0].strip(), 0)
    if len(parts) == 2:
        value <<= int(parts[1].strip(), 0)
    else:
        assert len(parts) == 1
    return value


def vlc(name, value):
    nodes = [
        (integer(a), integer(b))
        for a, b in re.findall(r"\.\{\s*([^,]+),\s*([^}]+)\}", _declarations()[name])
    ]

    def find(index, prefix):
        for bit in range(2):
            branch, leaf = nodes[index + bit]
            code = prefix + str(bit)
            if branch == 0 and leaf == value:
                return code
            if branch > 0:
                found = find(branch, code)
                if found is not None:
                    return found
        return None

    result = find(0, "")
    assert result is not None, (name, value)
    return result


class Bits:
    def __init__(self):
        self.bits = ""

    def put(self, value, width):
        assert 0 <= value < (1 << width)
        self.bits += format(value, f"0{width}b")

    def code(self, name, value):
        self.bits += vlc(name, value)

    def finish(self):
        bits = self.bits + "0" * (-len(self.bits) % 8)
        return bytes(int(bits[i : i + 8], 2) for i in range(0, len(bits), 8))


SEQUENCE = bytes([0, 0, 1, 0xB3, 1, 0, 0x10, 0x13, 0, 0, 0x20, 0])


def picture(kind, dc, temporal, ac=False):
    header = Bits()
    header.put(temporal, 10)
    header.put(kind, 3)
    header.put(0xFFFF, 16)
    if kind in (2, 3):
        header.put(0, 1)
        header.put(1, 3)
    if kind == 3:
        header.put(0, 1)
        header.put(1, 3)
    header.put(0, 1)
    slice_bits = Bits()
    slice_bits.put(8, 5)
    slice_bits.put(0, 1)
    slice_bits.code("MACROBLOCK_ADDRESS_INCREMENT", 1)
    table = {
        1: "MACROBLOCK_TYPE_INTRA",
        2: "MACROBLOCK_TYPE_PREDICTIVE",
        3: "MACROBLOCK_TYPE_B",
    }[kind]
    slice_bits.code(table, 1)
    for block in range(6):
        difference = dc - 128 if block == 0 else 0
        size = abs(difference).bit_length()
        slice_bits.code(
            "DCT_SIZE_LUMINANCE" if block < 4 else "DCT_SIZE_CHROMINANCE", size
        )
        if size:
            slice_bits.put(
                difference if difference > 0 else (1 << size) - 1 + difference, size
            )
        if ac and block == 0:
            slice_bits.code("DCT_COEFF", 1)
            slice_bits.put(1, 1)  # Distinguish run=0/level=1 from end-of-block.
            slice_bits.put(0, 1)  # Positive coefficient.
        slice_bits.code("DCT_COEFF", 1)
        slice_bits.put(0, 1)
    return (
        bytes([0, 0, 1, 0])
        + header.finish()
        + bytes([0, 0, 1, 1])
        + slice_bits.finish()
    )


def audio_frame(mode, selected_subband, protected=False):
    # MPEG-1, 128 kbit/s stereo: reviewed table A, 27 subbands. Joint stereo
    # extension zero shares sample codes from subband four onward.
    widths = [4] * 11 + [3] * 12 + [2] * 4
    bound = 4 if mode == 1 else 27
    payload = Bits()
    for sb, width in enumerate(widths):
        payload.put(1 if sb == selected_subband else 0, width)
        if sb < bound:
            payload.put(1 if sb == selected_subband else 0, width)
    for _ in range(2):
        payload.put(2, 2)  # One scalefactor for all three parts.
    protected_bits = payload.bits
    payload.put(3, 6)  # Leave headroom for synthesis ringing.
    payload.put(9, 6)  # Two octave steps: right channel gain is one quarter.
    for _ in range(12):
        payload.put(21, 5)  # Grouped base-three samples [0, 1, 2].
        if selected_subband < bound:
            payload.put(21, 5)
    body = payload.finish()
    header = bytes([0xFF, 0xFC if protected else 0xFD, 0x80, mode << 6])
    if protected:
        crc = 0xFFFF
        control = "".join(format(b, "08b") for b in header[2:]) + protected_bits
        for bit in control:
            feedback = (crc >> 15) ^ int(bit)
            crc = (crc << 1) & 0xFFFF
            if feedback:
                crc ^= 0x8005
        header += crc.to_bytes(2, "big")
    size = 417 - len(header)
    assert len(body) <= size
    return header + body + bytes(size - len(body))


def mpeg_fixtures():
    """Create original elementary test streams from reviewed MPEG VLC tables."""
    output = output_dir("portable-tests", "mpeg")
    output.mkdir(parents=True, exist_ok=True)
    (output / "reordered.m1v").write_bytes(
        SEQUENCE + picture(1, 80, 0) + picture(2, 160, 2) + picture(3, 120, 1)
    )
    (output / "ac.m1v").write_bytes(SEQUENCE + picture(1, 128, 0, ac=True))
    for name, mode, sb in [("stereo", 0, 0), ("dual", 2, 0), ("joint", 1, 5)]:
        frame = audio_frame(mode, sb)
        assert len(frame) == 417
        (output / (name + ".mp2")).write_bytes(frame * 2)
    frame = audio_frame(1, 5, protected=True)
    (output / "joint-crc.mp2").write_bytes(frame * 2)


def clipboard(output):
    """Build the independent Linux X11 clipboard service; never fetch SDK sources."""
    if platform.system() != "Linux":
        raise SystemExit(
            "native clipboard archive requires a Linux SDK/host; source checks are separate"
        )
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source = ROOT / "stdlib/internal/clipboard/native"
    compiler = os.environ.get("CC", "cc")
    commands = []
    for name in ["data", "x11"]:
        command = [
            compiler,
            "-std=c11",
            "-O2",
            "-fPIC",
            "-fvisibility=hidden",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-pthread",
            "-c",
            str(source / (name + ".c")),
            "-o",
            str(output / (name + ".o")),
        ]
        subprocess.run(command, check=True)
        commands.append(command)
    archive = output / "libportable_clipboard.a"
    command = [
        os.environ.get("AR", "ar"),
        "rcs",
        str(archive),
        str(output / "data.o"),
        str(output / "x11.o"),
    ]
    subprocess.run(command, check=True)
    commands.append(command)
    shared = output / "libportable_clipboard.so"
    command = [
        compiler,
        "-shared",
        "-pthread",
        str(output / "data.o"),
        str(output / "x11.o"),
        "-lX11",
        "-o",
        str(shared),
    ]
    subprocess.run(command, check=True)
    commands.append(command)
    manifest = dict(
        provenance="Independent project implementation; system Xlib and pthread ABI only",
        host=platform.platform(),
        commands=commands,
        required_system_libraries=["X11", "pthread"],
        static_consumer_dependencies=["X11", "pthread"],
        static_consumer_retention="Required even for a type-only X11 import; the Jai consumer declares private link_always dependencies.",
        inputs={
            str(p.relative_to(ROOT)): sha256_file(p)
            for p in [source / "data.h", source / "data.c", source / "x11.c"]
        },
        archive_sha256=sha256_file(archive),
        shared_sha256=sha256_file(shared),
    )
    (output / "portable-clipboard-build.json").write_text(
        json.dumps(manifest, indent=2) + "\n"
    )


def build_plan(target, output, temporary, cc, ar):
    if target not in {"macos-x64", "macos-arm64", "linux-x64", "linux-arm64"}:
        raise ValueError("portable crash builder supports desktop POSIX targets only")
    system, cpu = target.split("-")
    directory = output
    obj = temporary / "portable-crash.o"
    archive = directory / "libportable_crash.a"
    shared = directory / (
        "libportable_crash.dylib" if system == "macos" else "libportable_crash.so"
    )
    target_flags = (
        ["-arch", "arm64" if cpu == "arm64" else "x86_64"]
        if system == "macos"
        else [
            "--target="
            + ("aarch64" if cpu == "arm64" else "x86_64")
            + "-unknown-linux-gnu"
        ]
    )
    compile_ = [
        cc,
        *target_flags,
        "-std=c11",
        "-O2",
        "-fno-omit-frame-pointer",
        "-fvisibility=hidden",
        "-fPIC",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-c",
        str(ROOT / "stdlib/internal/runtime/crash-native/runtime.c"),
        "-o",
        str(obj),
    ]
    link = [
        cc,
        *target_flags,
        "-dynamiclib" if system == "macos" else "-shared",
        str(obj),
        "-o",
        str(shared),
        "-pthread",
    ]
    if system == "linux":
        link.extend(["-ldl", "-Wl,-soname,libportable_crash.so"])
    else:
        link.append("-Wl,-install_name,@rpath/libportable_crash.dylib")
    return {
        "commands": [compile_, [ar, "rcs", str(archive), str(obj)], link],
        "object": obj,
        "archive": archive,
        "shared": shared,
    }


def verify_object(path, target):
    with path.open("rb") as source:
        header = source.read(64)
    actual = None
    if len(header) >= 20 and header[:6] == b"\x7fELF\x02\x01":
        cpu = {62: "x64", 183: "arm64"}.get(int.from_bytes(header[18:20], "little"))
        actual = "linux-" + str(cpu)
    elif len(header) >= 8 and header[:4] == b"\xcf\xfa\xed\xfe":
        cpu = {0x1000007: "x64", 0x100000C: "arm64"}.get(
            int.from_bytes(header[4:8], "little")
        )
        actual = "macos-" + str(cpu)
    if actual != target:
        raise ValueError(
            f"compiled crash object target mismatch: requested {target}, found {actual}"
        )


def runtime(target, output_dir):
    """Build matching static/shared private POSIX crash libraries from our C source into `output_dir` (default:
    <target dir>/native-helpers/<platform>/crash-runtime), with a hash receipt. No executable or fault harness runs."""
    target = target or host_platform()
    if target != host_platform():
        raise SystemExit(
            "native preparation requires the actual host; cross SDK builds are separate"
        )
    temporary = cargo_target_dir() / "native-helpers" / target
    output = (output_dir or temporary / "crash-runtime").resolve()
    temporary.mkdir(parents=True, exist_ok=True)
    plan = build_plan(
        target, output, temporary, "clang", shutil.which("llvm-ar") or "ar"
    )
    plan["archive"].parent.mkdir(parents=True, exist_ok=True)
    plan["archive"].unlink(missing_ok=True)
    for index, command in enumerate(plan["commands"]):
        subprocess.run(command, cwd=ROOT, check=True)
        if index == 0:
            verify_object(plan["object"], target)
    artifacts = {}
    for path in [plan["archive"], plan["shared"]]:
        artifacts[path.name] = sha256_file(path)
    source = ROOT / "stdlib/internal/runtime/crash-native/runtime.c"
    receipt = {
        "platform": target,
        "source_sha256": sha256_file(source),
        "artifacts": artifacts,
        "native_execution": False,
    }
    (plan["archive"].parent / "portable-crash-build.json").write_text(
        json.dumps(receipt, indent=2) + "\n"
    )
    print("Built matching private crash static/shared libraries from source:", target)


def watcher(output_dir):
    """Build our native FSEvents callback/lifecycle implementation against the public SDK (macOS only) into
    `output_dir` (default: target/native-helpers/macos-watch)."""
    if platform.system() != "Darwin":
        raise SystemExit("native watcher backend currently supports macOS only")
    output = output_dir or cargo_target_dir() / "native-helpers/macos-watch"
    output.mkdir(parents=True, exist_ok=True)
    directory = ROOT / "stdlib/internal/watching/native"
    subprocess.run(
        [
            "clang",
            "-std=c11",
            "-O2",
            "-fblocks",
            "-fvisibility=hidden",
            "-arch",
            "arm64",
            "-arch",
            "x86_64",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-dynamiclib",
            str(directory / "watch.c"),
            "-Wl,-install_name,@rpath/libportable_watch.dylib",
            "-framework",
            "CoreServices",
            "-pthread",
            "-o",
            str(output / "libportable_watch.dylib"),
        ],
        cwd=ROOT,
        check=True,
    )
    print("Built independent native FSEvents lifecycle/queue backend from source")


def main(argv=None):
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="command", required=True)
    for name, function in (
        ("zip", zip_fixtures),
        ("archive", archive_fixtures),
        ("mpeg", mpeg_fixtures),
        ("machine", machine_fixtures),
    ):
        commands.add_parser(name, help=function.__doc__.split("\n")[0]).set_defaults(
            run=lambda args, f=function: f()
        )
    meow = commands.add_parser("meow", help=meow_fixtures.__doc__.split("\n")[0])
    meow.add_argument("--verify-native", action="store_true")
    meow.set_defaults(run=lambda args: meow_fixtures(args.verify_native))
    board = commands.add_parser("clipboard", help=clipboard.__doc__.split("\n")[0])
    board.add_argument("--output", type=Path, required=True)
    board.set_defaults(run=lambda args: clipboard(args.output))
    crash = commands.add_parser("runtime", help=runtime.__doc__.split("\n")[0])
    crash.add_argument(
        "--platform", choices=["macos-x64", "macos-arm64", "linux-x64", "linux-arm64"]
    )
    crash.add_argument(
        "--output-dir",
        type=Path,
        help="where libportable_crash.* and portable-crash-build.json go",
    )
    crash.set_defaults(run=lambda args: runtime(args.platform, args.output_dir))
    watch = commands.add_parser("watcher", help=watcher.__doc__.split("\n")[0])
    watch.add_argument("--output-dir", type=Path)
    watch.set_defaults(run=lambda args: watcher(args.output_dir))
    args = parser.parse_args(argv)
    args.run(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
