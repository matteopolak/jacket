#!/usr/bin/env python3
"""Put the native helpers Bindings_Generator needs into a release archive's native-libs/<platform>.

    python3 tools/ci/package_bindings_helpers.py --platform linux-x64 --llvm-prefix DIR --out DIR

- `libgenerator_platform`, shared (for `jaic -interp`) and static (for built programs), on every platform: library loading, symbol lookup and environment access
  for `generate_bindings` (stdlib/internal/developer-tools/generator-platform.jai).
- `libclang` (Linux and Windows): from the same pinned LLVM release as the compiler, with the
  libraries it links beside it. macOS ships none: the Xcode toolchain every `generate_bindings` run
  already needs for the SDK's headers holds one, and jaic searches it (docs/tools/releases.md,
  "Bindings_Generator"). Prints what was added and its size.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))
# developer_checks/, whose modules import each other unqualified
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1] / "developer_checks"))

import argparse
import shutil
import subprocess
from pathlib import Path

from developer_checks import build_generator_platform
from native.build_native_libs import static_name
from toolkit.repo import ROOT


def build_static(prefix: Path, platform: str, out: Path) -> Path:
    """The helper as a static archive, which a built program (`jaic build.jai`) links like the other native libraries."""
    source = ROOT / "stdlib/internal/developer-tools/generator-platform-native.c"
    obj = out / "generator_platform.o"
    # Windows objects are position independent already, and clang rejects -fPIC for MSVC targets.
    pic = [] if platform.startswith("windows") else ["-fPIC"]
    subprocess.run(
        [str(prefix / "bin/clang"), "-std=c11", "-O2", *pic, "-c", str(source), "-o", str(obj)],
        check=True,
    )
    archive = out / static_name("generator_platform", platform)
    archive.unlink(missing_ok=True)
    subprocess.run([str(prefix / "bin/llvm-ar"), "rcs", str(archive), str(obj)], check=True)
    obj.unlink()
    return archive


def needed(library: Path) -> list[str]:
    """The shared libraries an ELF image names (DT_NEEDED)."""
    text = subprocess.run(
        ["readelf", "-d", str(library)], check=True, capture_output=True, text=True
    ).stdout
    return [
        line.split("[", 1)[1].split("]", 1)[0]
        for line in text.splitlines()
        if "(NEEDED)" in line
    ]


def copy_linux_libclang(prefix: Path, out: Path) -> list[Path]:
    """libclang.so and the LLVM libraries beside it in the release, found through $ORIGIN."""
    source = (prefix / "lib/libclang.so").resolve(strict=True)
    copied = [out / "libclang.so"]
    shutil.copy2(source, copied[0])
    pending = needed(source)
    seen = set()
    while pending:
        name = pending.pop()
        if name in seen:
            continue
        seen.add(name)
        found = prefix / "lib" / name
        if not found.exists():
            continue  # the system's (libc, libstdc++, libm...)
        target = out / name
        shutil.copy2(found.resolve(), target)
        copied.append(target)
        pending += needed(target)
    for image in copied:
        # The release's own run path points into its install tree.
        subprocess.run(["patchelf", "--set-rpath", "$ORIGIN", str(image)], check=True)
    return copied


def copy_windows_libclang(prefix: Path, out: Path) -> list[Path]:
    source = prefix / "bin/libclang.dll"
    if not source.is_file():
        raise SystemExit(f"{prefix} has no bin/libclang.dll")
    shutil.copy2(source, out / source.name)
    return [out / source.name]


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--platform", required=True)
    parser.add_argument("--llvm-prefix", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    added = [
        build_generator_platform.build(args.llvm_prefix, args.out, record_nm=False),
        build_static(args.llvm_prefix, args.platform, args.out),
    ]
    if args.platform.startswith("linux"):
        added += copy_linux_libclang(args.llvm_prefix, args.out)
    elif args.platform.startswith("windows"):
        added += copy_windows_libclang(args.llvm_prefix, args.out)
    total = 0
    for path in added:
        size = path.stat().st_size
        total += size
        print(f"{path.name}: {size / 1024:.0f} KiB")
    print(f"Bindings_Generator helpers for {args.platform}: {total / 1048576:.2f} MiB")


if __name__ == "__main__":
    main()
