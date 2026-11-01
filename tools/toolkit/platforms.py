"""The platforms the tools build for: their `<os>-<cpu>` names and the compiler target each one means."""

__all__ = [
    "DESKTOP_POSIX",
    "MINGW_PREFIXES",
    "PLATFORMS",
    "TRIPLES",
    "WINDOWS",
    "WINDOWS_TRIPLES",
]

# Each platform's Clang/LLVM target. A tool that needs a minimum OS version spells it on top of this.
TRIPLES = {
    "macos-x64": "x86_64-apple-macos",
    "macos-arm64": "arm64-apple-macos",
    "linux-x64": "x86_64-unknown-linux-gnu",
    "linux-arm64": "aarch64-unknown-linux-gnu",
    "windows-x64": "x86_64-pc-windows-msvc",
    "windows-arm64": "aarch64-pc-windows-msvc",
}
PLATFORMS = tuple(TRIPLES)
WINDOWS = tuple(name for name in PLATFORMS if name.startswith("windows-"))
DESKTOP_POSIX = tuple(name for name in PLATFORMS if name not in WINDOWS)
WINDOWS_TRIPLES = {name: TRIPLES[name] for name in WINDOWS}
# The tool prefix of each MinGW cross platform (`windows-<cpu>-mingw`, built on Linux or macOS).
MINGW_PREFIXES = {
    "windows-x64-mingw": "x86_64-w64-mingw32",
    "windows-arm64-mingw": "aarch64-w64-mingw32",
}
