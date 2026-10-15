#!/usr/bin/env python3
"""Build audio-owned static/shared libraries and an opaque pinned C decoder.

Only this worktree's output directories are written. Six desktop release targets
are described in audio_native_plan.json; cross SDKs must be supplied explicitly.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shlex
import shutil
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PLAN = json.loads((ROOT / "tools/audio_native_plan.json").read_text())


def host_platform():
    osname = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}[platform.system()]
    cpu = {"arm64": "arm64", "ARM64": "arm64", "aarch64": "arm64", "AMD64": "x64", "x86_64": "x64"}[platform.machine()]
    return osname + "-" + cpu


def command(arguments, owned=True):
    result = subprocess.run(list(map(str, arguments)), cwd=ROOT, capture_output=True, text=True)
    if result.returncode or (owned and result.stderr):
        errors = [line for line in result.stderr.splitlines() if "error:" in line or "warning:" in line]
        raise RuntimeError(f"native build failed: {arguments[0]} exit {result.returncode}; " + "; ".join(errors[:6]))
    warnings = result.stderr.count("warning:")
    if warnings:
        print(f"Official C decoder build reported {warnings} compiler warning(s); no suppression applied.")


def verify_object(path, entry):
    data = path.read_bytes()[:64]
    expected = entry["cpu"]
    if data[:4] == b"\x7fELF":
        if data[4:6] != bytes((2, 1)):
            raise RuntimeError("Desktop audio requires a 64-bit little-endian ELF object")
        osname = "linux"
        machine = int.from_bytes(data[18:20], "little")
        cpu = {62: "x64", 183: "arm64"}.get(machine)
    elif data[:4] == b"\xcf\xfa\xed\xfe":
        osname = "macos"
        machine = int.from_bytes(data[4:8], "little")
        cpu = {0x1000007: "x64", 0x100000c: "arm64"}.get(machine)
    else:
        osname = "windows"
        machine = int.from_bytes(data[:2], "little")
        cpu = {0x8664: "x64", 0xaa64: "arm64"}.get(machine)
    if (osname, cpu) != (entry["os"], expected):
        raise RuntimeError(f"Object target mismatch for {path.name}: requested {entry['os']}-{expected}, produced {osname}-{cpu}")


def opaque_decoder(out):
    source = json.loads((ROOT / "tools/native-libs.json").read_text())["sources"]["stb"]
    cache = ROOT / "target/audio-native/official-c-dependencies"
    cache.mkdir(parents=True, exist_ok=True)
    for filename, destination in [("stb_vorbis.c", cache / "stb_vorbis.c"),
                                  ("LICENSE", out / "licenses/stb-LICENSE.txt")]:
        if not destination.exists():
            data = urllib.request.urlopen(
                f"https://raw.githubusercontent.com/{source['repository']}/{source['revision']}/{filename}", timeout=30).read()
            if hashlib.sha256(data).hexdigest() != source["files"][filename]:
                raise RuntimeError("official decoder checksum mismatch")
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        if hashlib.sha256(destination.read_bytes()).hexdigest() != source["files"][filename]:
            raise RuntimeError("cached official decoder checksum mismatch")
    return cache / "stb_vorbis.c"


def build(target, out, cc, ar, mingw=False, runtime=False, require_device=False):
    entry = PLAN["platforms"][target]
    osname = entry["os"]
    windows = osname == "windows"
    out.mkdir(parents=True, exist_ok=True)
    flags = ["-std=c11", "-O2"]
    if osname == "macos":
        flags += ["-arch", entry["arch"], "-mmacosx-version-min=11.0", "-fPIC"]
        libraries = [part for name in entry["libraries"] for part in ("-framework", name)]
    else:
        flags += [] if windows else ["-fPIC"]
        libraries = ["-l" + name for name in entry["libraries"]]
    objext = ".obj" if windows and not mingw else ".o"
    libext = ".lib" if windows and not mingw else ".a"
    ar_flags = ["--format=coff"] if windows and not mingw else []
    prefix = "" if windows and not mingw else "lib"
    shared_name = "jai_audio_output.dll" if windows else "libjai_audio_output." + ("dylib" if osname == "macos" else "so")
    shared_flags = ["-dynamiclib"] if osname == "macos" else ["-shared"]
    objects = []
    for filename in ["output.c"] + (["focus.m"] if osname == "macos" else []):
        obj = out / (Path(filename).stem + objext)
        command([*cc, *flags, "-Wall", "-Wextra", "-Werror", "-c",
                 ROOT / "stdlib/internal/audio/native" / filename, "-o", obj])
        verify_object(obj, entry)
        objects.append(obj)
    archive = out / (prefix + "jai_audio_output" + libext)
    archive.unlink(missing_ok=True)
    command([*ar, *ar_flags, "rcs", archive, *objects])
    extra = ["-Wl,-install_name,@rpath/" + shared_name] if osname == "macos" else []
    command([*cc, *flags, *shared_flags, *objects, "-o", out / shared_name, *extra, *libraries])
    decoder = opaque_decoder(out)
    decoder_obj = out / ("stb_vorbis" + objext)
    command([*cc, *flags, "-c", decoder, "-o", decoder_obj], owned=False)
    verify_object(decoder_obj, entry)
    decoder_archive = out / (prefix + "stb_vorbis" + libext)
    decoder_archive.unlink(missing_ok=True)
    command([*ar, *ar_flags, "rcs", decoder_archive, decoder_obj])
    decoder_shared = "stb_vorbis.dll" if windows else "libstb_vorbis." + ("dylib" if osname == "macos" else "so")
    extra = ["-Wl,-install_name,@rpath/" + decoder_shared] if osname == "macos" else []
    command([*cc, *flags, *shared_flags, decoder_obj, "-o", out / decoder_shared, *extra,
             *([] if windows and not mingw else ["-lm"])], owned=False)
    artifacts = [archive, out / shared_name, decoder_archive, out / decoder_shared]
    metadata = {"platform": target, "native_execution": False,
                "linkage": "static native executable; shared interpreter",
                "compiler": cc, "libraries": entry["libraries"],
                "artifacts": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in artifacts}}
    (out / "audio-build.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"{target}: owned static/shared SDK builds passed warnings denied; opaque decoder built; execution not implied.", flush=True)
    executable = out / ("native-output-test.exe" if windows else "native-output-test")
    command([*cc, *flags, "-Wall", "-Wextra", "-Werror", "-I", ROOT / "stdlib/internal/audio/native",
             ROOT / "tests/audio/native-output.c", archive, "-o", executable, *libraries])
    print(f"{target}: native SDK harness linked (execution not implied).", flush=True)
    if runtime:
        result = subprocess.run([str(executable), *(["--require-device"] if require_device else [])],
                                cwd=out, capture_output=True, text=True, timeout=30)
        record = {"platform": target, "kind": "exact-host SDK harness execution",
                  "exit_code": result.returncode, "device_opened": "native-device-opened:" in result.stdout,
                  "physical_focus_hotplug_sleep": "not tested", "require_device": require_device,
                  "diagnostic_lines": len(result.stderr.splitlines()),
                  "manual_pause_unsupported": "native-manual-pause-unsupported" in result.stdout}
        (out / "audio-runtime.json").write_text(json.dumps(record, indent=2) + "\n")
        if result.returncode or result.stderr:
            raise RuntimeError(f"native SDK harness failed: exit {result.returncode}")
        print(result.stdout.strip(), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=PLAN["platforms"])
    parser.add_argument("--cc", help="compiler driver and explicit cross SDK flags")
    parser.add_argument("--ar", help="LLVM/GNU ar-compatible archiver")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--mingw", action="store_true", help="produce GNU Windows archive names")
    parser.add_argument("--check-windows", action="store_true", help="also cross-build Windows x64 using installed MinGW-w64")
    parser.add_argument("--test-runtime", action="store_true", help="execute the silent SDK harness on this exact host target")
    parser.add_argument("--require-device", action="store_true", help="fail runtime tests when no native device opens")
    args = parser.parse_args()
    target = args.platform or host_platform()
    if args.test_runtime and target != host_platform():
        parser.error("runtime tests require the exact host platform; cross builds are not runtime evidence")
    apple_cross = target.startswith("macos-") and host_platform().startswith("macos-")
    if target != host_platform() and not args.cc and not apple_cross:
        parser.error("cross builds require --cc with the target SDK; no host substitution")
    out = (args.out or (ROOT / "target/audio-native" / target if args.platform else ROOT / "target/audio-native")).resolve()
    if not out.is_relative_to(ROOT):
        parser.error("--out must remain inside this worktree")
    cc = shlex.split(args.cc) if args.cc else ["clang"]
    ar = shlex.split(args.ar) if args.ar else [shutil.which("llvm-ar") or "ar"]
    build(target, out, cc, ar, args.mingw, args.test_runtime, args.require_device)
    if args.check_windows:
        compiler = shutil.which("x86_64-w64-mingw32-gcc")
        archiver = shutil.which("x86_64-w64-mingw32-ar")
        if not compiler or not archiver:
            parser.error("--check-windows requires the public MinGW-w64 SDK")
        build("windows-x64", ROOT / "target/audio-native/windows-x64-mingw", [compiler], [archiver], True)


if __name__ == "__main__":
    main()
