#!/usr/bin/env python3
"""Build and inspect real resource-only Windows PE fixtures; never execute them."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

MACHINES = {"x64": 0x8664, "arm64": 0xAA64}
SIZES = (256, 128, 64, 32, 16)
ARTIFACTS = (
    "resource.ico",
    "resource.rc",
    "resource.res",
    "resource-only.dll",
    "inspection.txt",
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unpack(data, offset, fmt):
    require(
        0 <= offset <= len(data) - struct.calcsize(fmt), "truncated resource/PE field"
    )
    return struct.unpack_from(fmt, data, offset)


def make_ico():
    """Five genuine opaque BGRA DIBs, with bottom-up rows and DWORD AND masks."""
    directory = bytearray(struct.pack("<HHH", 0, 1, len(SIZES)))
    images = []
    offset = 6 + 16 * len(SIZES)
    for index, size in enumerate(SIZES):
        pixels = bytearray()
        for y in range(size):
            for x in range(size):
                pixels.extend(
                    (
                        (x + y) & 255,
                        (index * 37 + x) & 255,
                        (y * 3 + index * 19) & 255,
                        255,
                    )
                )
        mask = bytes(((size + 31) // 32) * 4 * size)
        dib = (
            struct.pack(
                "<IiiHHIIiiII", 40, size, size * 2, 1, 32, 0, len(pixels), 0, 0, 0, 0
            )
            + pixels
            + mask
        )
        directory.extend(
            struct.pack(
                "<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(dib), offset
            )
        )
        images.append(dib)
        offset += len(dib)
    return bytes(directory) + b"".join(images)


def expected_resources(icon):
    require(unpack(icon, 0, "<HHH") == (0, 1, 5), "ICO must contain five icon images")
    group = bytearray(struct.pack("<HHH", 0, 1, 5))
    resources = {}
    cursor = 86
    for index, size in enumerate(SIZES):
        w, h, colors, reserved, planes, bits, length, offset = unpack(
            icon, 6 + index * 16, "<BBBBHHII"
        )
        require(
            (w or 256, h or 256, colors, reserved, planes, bits)
            == (size, size, 0, 0, 1, 32),
            "invalid ICO directory fields",
        )
        require(
            offset == cursor and length <= len(icon) - offset,
            "invalid ICO image extent",
        )
        image = icon[offset : offset + length]
        require(
            unpack(image, 0, "<IiiHHI") == (40, size, size * 2, 1, 32, 0),
            "invalid DIB image header",
        )
        require(
            length == 40 + size * size * 4 + ((size + 31) // 32) * 4 * size,
            "invalid DIB pixel/mask length",
        )
        resources[3, index + 1, 1033] = image
        group.extend(
            struct.pack(
                "<BBBBHHIH", w, h, colors, reserved, planes, bits, length, index + 1
            )
        )
        cursor += length
    require(cursor == len(icon), "unexpected ICO trailing data")
    require(len(group) == 76, "five-image group must contain 76 bytes")
    resources[14, 101, 1033] = bytes(group)
    return resources


def inspect_pe(data, cpu, icon):
    """Validate actual linked headers and every resource payload against the ICO."""
    require(cpu in MACHINES, "unsupported target machine")
    require(data[:2] == b"MZ", "missing DOS header")
    (pe,) = unpack(data, 0x3C, "<I")
    require(data[pe : pe + 4] == b"PE\0\0", "missing PE signature")
    machine, count, _, _, _, optional_size, characteristics = unpack(
        data, pe + 4, "<HHIIIHH"
    )
    require(machine == MACHINES[cpu], "linked PE machine differs from requested CPU")
    require(characteristics & 0x2000, "resource-only image must be a DLL")
    optional = pe + 24
    require(
        optional_size >= 136 and optional + optional_size <= len(data),
        "truncated optional header",
    )
    require(unpack(data, optional, "<H")[0] == 0x20B, "PE32+ required")
    require(
        unpack(data, optional + 4, "<I")[0] == 0, "resource-only image contains code"
    )
    (entry,) = unpack(data, optional + 16, "<I")
    require(entry == 0, "resource-only image has an entry point")
    require(unpack(data, optional + 108, "<I")[0] >= 3, "missing PE data directories")
    require(
        unpack(data, optional + 120, "<II") == (0, 0),
        "resource-only image imports native code",
    )
    resource_rva, resource_size = unpack(data, optional + 128, "<II")
    require(resource_rva and resource_size >= 16, "missing PE resource directory")
    sections = []
    for index in range(count):
        offset = optional + optional_size + 40 * index
        _, rva, raw_size, raw_offset = unpack(data, offset + 8, "<IIII")
        (flags,) = unpack(data, offset + 36, "<I")
        require(not flags & 0x20000000, "resource-only image has an executable section")
        require(raw_offset + raw_size <= len(data), "section extends beyond PE file")
        sections.append((rva, raw_size, raw_offset))

    def file_offset(rva, size):
        for start, length, raw in sections:
            if start <= rva and rva + size <= start + length:
                return raw + rva - start
        raise ValueError("resource RVA extends beyond a file-backed section")

    base = file_offset(resource_rva, resource_size)
    resources = {}
    visited = set()

    def relative(offset, size):
        require(
            0 <= offset <= resource_size - size, "resource directory extent is invalid"
        )
        return base + offset

    def walk(offset, path):
        require(
            len(path) < 3 and offset not in visited, "invalid resource tree depth/cycle"
        )
        visited.add(offset)
        named, ids = unpack(data, relative(offset, 16) + 12, "<HH")
        require(named == 0, "fixture requires numeric resource IDs")
        for index in range(ids):
            name, child = unpack(data, relative(offset + 16 + index * 8, 8), "<II")
            require(not name & 0x80000000, "unexpected named resource")
            key = (*path, name)
            if child & 0x80000000:
                walk(child & 0x7FFFFFFF, key)
            else:
                require(
                    len(key) == 3 and key not in resources,
                    "invalid/duplicate resource leaf",
                )
                rva, length, _, reserved = unpack(data, relative(child, 16), "<IIII")
                require(reserved == 0, "invalid resource data entry")
                payload = file_offset(rva, length)
                resources[key] = bytes(data[payload : payload + length])

    walk(0, ())
    expected = expected_resources(icon)
    require(
        resources.keys() == expected.keys(),
        "linked PE resource IDs/languages differ from fixture",
    )
    for key, payload in expected.items():
        require(resources[key] == payload, f"linked resource payload mismatch: {key}")
    return {
        "machine": machine,
        "entry_point": entry,
        "resource_count": len(resources),
        "group_bytes": len(resources[14, 101, 1033]),
        "resources": [
            {
                "type": key[0],
                "id": key[1],
                "language": key[2],
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
            for key, payload in sorted(resources.items())
        ],
    }


def tool_path(explicit, candidates):
    names = [explicit] if explicit else candidates
    for name in names:
        found = shutil.which(str(name))
        if found and Path(found).is_file():
            # Preserve argv[0] symlink names: LLVM's lld-link is a multi-call tool.
            return str(Path(found).absolute())
    raise FileNotFoundError(
        "required SDK tool not found: " + ", ".join(map(str, names))
    )


def build_fixture(output, cpu, rc=None, linker=None, readobj=None):
    require(cpu in MACHINES, "unsupported target CPU")
    output = Path(output).absolute()
    require(
        not any(
            (output / name).exists() for name in (*ARTIFACTS, "fixture-receipt.json")
        ),
        "use a fresh output directory; existing fixture is not replaced",
    )
    output.mkdir(parents=True, exist_ok=True)
    receipt = {
        "schema": 1,
        "status": "building",
        "cpu": cpu,
        "native_verified": False,
        "commands": [],
    }
    receipt_path = output / "fixture-receipt.json"

    def save():
        with receipt_path.open("w") as stream:
            stream.write(json.dumps(receipt, indent=2) + "\n")
            stream.flush()
            os.fsync(stream.fileno())

    save()
    try:
        rc = tool_path(rc, ["llvm-rc", *(["rc.exe"] if os.name == "nt" else [])])
        linker = tool_path(
            linker, ["lld-link", *(["link.exe"] if os.name == "nt" else [])]
        )
        readobj = tool_path(readobj, ["llvm-readobj"])
        receipt["tools"] = {
            name: {
                "path": path,
                "sha256": hashlib.sha256(Path(path).read_bytes()).hexdigest(),
            }
            for name, path in (("rc", rc), ("linker", linker), ("readobj", readobj))
        }
        with tempfile.TemporaryDirectory(
            prefix="resource-build-", dir=output
        ) as directory:
            work = Path(directory)
            icon = make_ico()
            expected_resources(icon)
            (work / "resource.ico").write_bytes(icon)
            (work / "resource.rc").write_text(
                'LANGUAGE 9, 1\n101 ICON "resource.ico"\n'
            )

            def run(command):
                result = subprocess.run(
                    command, cwd=work, capture_output=True, text=True, timeout=120
                )
                receipt["commands"].append(
                    {
                        "argv": command,
                        "exit": result.returncode,
                        "stdout": result.stdout,
                        "stderr": result.stderr,
                    }
                )
                save()
                if result.returncode:
                    raise ValueError(
                        f"SDK command failed ({result.returncode}): {command[0]}\n{result.stdout}{result.stderr}"
                    )
                return result.stdout

            run([rc, "/nologo", "/fo", "resource.res", "resource.rc"])
            deterministic = (
                "/timestamp:0" if Path(linker).stem.lower() == "lld-link" else "/Brepro"
            )
            run(
                [
                    linker,
                    "/dll",
                    "/noentry",
                    f"/machine:{cpu}",
                    "/out:resource-only.dll",
                    deterministic,
                    "resource.res",
                ]
            )
            inspection = run(
                [readobj, "--file-headers", "--coff-resources", "resource-only.dll"]
            )
            (work / "inspection.txt").write_text(inspection)
            receipt["pe"] = inspect_pe(
                (work / "resource-only.dll").read_bytes(), cpu, icon
            )
            receipt["artifacts"] = {
                name: {
                    "sha256": hashlib.sha256((work / name).read_bytes()).hexdigest(),
                    "bytes": (work / name).stat().st_size,
                }
                for name in ARTIFACTS
            }
            for name in ARTIFACTS:
                # Publish the actual verified SDK bytes through a durable write.
                # Renaming SDK-created files alone did not preserve their data
                # across the worker's T7 execution boundary (retained RED).
                with (output / name).open("xb") as stream:
                    stream.write((work / name).read_bytes())
                    stream.flush()
                    os.fsync(stream.fileno())
        for name, identity in receipt["artifacts"].items():
            published = (output / name).read_bytes()
            require(
                len(published) == identity["bytes"]
                and hashlib.sha256(published).hexdigest() == identity["sha256"],
                f"published artifact hash mismatch: {name}",
            )
        require(
            inspect_pe(
                (output / "resource-only.dll").read_bytes(),
                cpu,
                (output / "resource.ico").read_bytes(),
            )
            == receipt["pe"],
            "published PE inspection differs from verified SDK output",
        )
        receipt["publication"] = (
            "durable byte-for-byte SDK output copy, fsync, hashes and PE re-inspection after staging cleanup"
        )
        receipt["status"] = "verified_cross_build"
        save()
        return receipt
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error)
        save()
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out", type=Path, required=True, help="fresh caller-owned fixture directory"
    )
    parser.add_argument("--cpu", choices=tuple(MACHINES), required=True)
    parser.add_argument("--rc", help="actual llvm-rc or Microsoft rc.exe")
    parser.add_argument("--linker", help="actual lld-link or Microsoft link.exe")
    parser.add_argument(
        "--readobj", help="actual llvm-readobj for independent SDK inspection"
    )
    args = parser.parse_args()
    try:
        receipt = build_fixture(args.out, args.cpu, args.rc, args.linker, args.readobj)
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(error, file=sys.stderr)
        return 1
    print(
        f"verified {args.cpu} resource-only PE and five-image ICO: {receipt['artifacts']['resource-only.dll']['sha256']}; native execution unverified"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
