#!/usr/bin/env python3
"""Every native library the shipped standard library binds must come with the release archive.

    python3 tools/ci/check_native_libs.py --stdlib stdlib --platform linux-x64 [--native DIR]

Scans `stdlib/` (not `tests/`) for `#library` names and sorts each into one of:

- built or fetched for the release: a library of tools/native/native-libs.json (including the
  rpmalloc profile libraries), or wgpu-native.
  With `--native`, its static archive must be in that directory (the archive's
  `native-libs/<platform>`), when the library is built for the platform.
- compiled into `jaic` (the crash and file-watcher runtimes), or the Windows crash runtime, which
  release.yml's Package step builds into `stdlib/`.
- `EXTERNAL`: the operating system's, the SDK's or the user's (system and Apple frameworks, X11,
  libcurl, GPU APIs, Android, NVIDIA SDKs).

A name in none of these fails: a new binding must either be added to native-libs.json or be
listed here on purpose (docs/tools/native-libs.md).
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import json
import re
import sys
from pathlib import Path

from native.build_native_libs import MANIFEST, rpmalloc_libraries, static_name, wanted

# Built into jaic by crates/jaic-cli/build.rs.
EMBEDDED = {"portable_crash", "portable_watch"}
# Built by release.yml's Package step into stdlib/internal/runtime/crash-native.
WINDOWS_CRASH = {"platform_io_windows_crash"}
# wgpu-native, fetched by tools/bindings/fetch_wgpu_native.py.
FETCHED = {"wgpu_native"}
EXTERNAL = {
    "c", "c++", "m", "dl", "rt", "pthread", "stdc++", "System", "objc", "evdev", "curl", "vulkan",
    "GL", "GLX", "EGL", "X11", "OpenGL", "z", "zlib1", "log", "android", "aaudio", "camera2ndk",
    "mediandk", "android_native_app_glue", "android-crypto/crypto", "nvtt", "nvidia_aftermath",
    "d3d11", "d3d12", "d3dcompiler", "dxcompiler", "dxgi",
    # Windows system libraries
    "kernel32", "user32", "gdi32", "advapi32", "Advapi32", "shell32", "ole32", "oleaut32",
    "opengl32", "winmm", "ws2_32", "bcrypt", "ntdll", "ucrt", "DbgHelp", "Dwmapi", "Shlwapi",
    "comdlg32", "setupapi", "imm32", "version", "uuid", "cfgmgr32", "dinput8",
    # Linux system libraries and Apple frameworks
    "asound", "AppKit", "Foundation", "Foundation_Framework", "Cocoa", "CoreGraphics",
    "CoreFoundation", "CoreServices", "corefoundation", "CoreAudio", "AudioToolbox", "Metal",
    "QuartzCore", "Carbon", "IOKit", "AVFoundation", "CoreMedia", "CoreVideo", "GameController",
    "ForceFeedback",
}

LIBRARY = re.compile(r'#library[^"\n]*"([^"\n\\]+)"')


def bound_names(stdlib: Path) -> dict[str, Path]:
    """Each library name the shipped sources bind, with a file that does."""
    found: dict[str, Path] = {}
    for path in sorted(stdlib.rglob("*.jai")):
        if "tests" in path.relative_to(stdlib).parts:
            continue
        for raw in LIBRARY.findall(path.read_text(encoding="utf-8")):
            name = raw.rsplit("/", 1)[-1] if raw.startswith(("native/", "crash-native/", "../crash-native/")) else raw
            found.setdefault(name.removeprefix("lib"), path)
    return found


def check(stdlib: Path, platform: str, native: Path | None) -> list[str]:
    libraries = json.loads(MANIFEST.read_text())["libraries"]
    problems = []
    profiles = set(rpmalloc_libraries())
    for name, path in bound_names(stdlib).items():
        if name in libraries or name in FETCHED or name in profiles:
            # The rpmalloc profile libraries are one manifest entry, `rpmalloc`.
            built = name in FETCHED or wanted(libraries["rpmalloc" if name in profiles else name], platform)
            if native is not None and built and not (native / static_name(name, platform)).is_file():
                problems.append(f"{path}: `{name}` is not in {native} ({static_name(name, platform)})")
        elif name not in EMBEDDED | WINDOWS_CRASH | EXTERNAL:
            problems.append(f"{path}: `{name}` is bound but neither built by tools/native/native-libs.json "
                            "nor listed as external in tools/ci/check_native_libs.py")
    return problems


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--platform", required=True)
    parser.add_argument("--native", type=Path, help="the archive's native-libs/<platform> directory")
    args = parser.parse_args()
    problems = check(args.stdlib, args.platform, args.native)
    if problems:
        sys.exit("\n".join(problems))
    print(f"checked: every native library the stdlib binds is accounted for on {args.platform}")


if __name__ == "__main__":
    main()
