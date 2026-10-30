"""Who provides each native library the standard library binds.

    bindings(stdlib)                every library binding the shipped sources hold, with its file, line and `@native` note
    bound_names(stdlib)             name -> a file that binds it
    providers(stdlib)               (name -> Provider, problems) for everything but the libraries built from native-libs.json
    provider_of(name, ...)          the Provider of one bound name, or None when nothing declares it
    on_platform(provider, platform) whether a declaration applies to a release archive

Every `#library`, `#system_library`, `#foreign_library` and `#foreign_system_library` line the stdlib binds a name on
says who provides it in a note after the `;`:

    libm :: #library,system "m"; @native(external)
    backend :: #library "native/portable_watch"; @native(embedded)
    native_runtime :: #library "crash-native/platform_io_windows_crash"; @native(runtime,windows)

`@native(<kind>[,<os>...])`: the kinds are `KINDS`, and the OS names limit the archives a declaration applies to. A name
that tools/native/native-libs.json builds carries no note. One note per name is enough, and notes of one name must agree.
The packaging recipes of `fetched`, `runtime` and `packaged` libraries are `RECIPES` below. The check is
`tools/ci/check_native_libs.py` and the packaging `tools/native/package_native_libs.py` (docs/tools/native-libs.md).
"""

import json
import re
from dataclasses import dataclass

from native.build_native_libs import MANIFEST, rpmalloc_libraries
from toolkit.repo import ROOT

KINDS = ("embedded", "fetched", "runtime", "packaged", "developer", "external")
# A binding: the directive, then (up to the `;`) the quoted library name.
LIBRARY = re.compile(r'#(?:foreign_)?(?:system_)?library\b[^"\n;]*"([^"\n\\]+)"')
NOTE = re.compile(r'@native\(([^)\s]*)\)')

# The script a `fetched`, `runtime` or `packaged` library is produced by while packaging. `{platform}`, `{cpu}`,
# `{stdlib}`, `{out}` and `{llvm_prefix}` in `args` are replaced.
RECIPES = {
    "wgpu_native": {
        "script": "tools/bindings/fetch_wgpu_native.py",
        "args": ["--platform", "{platform}", "--out", "{out}"],  # the pin is in tools/bindings/webgpu.json
    },
    "platform_io_windows_crash": {
        "script": "tools/stdlib/build_platform_io_runtime.py",
        "args": ["--stdlib", "{stdlib}", "--cpu", "{cpu}", "--static-library"],  # C built into stdlib/internal/runtime/crash-native
    },
    "generator_platform": {
        "script": "tools/ci/package_bindings_helpers.py",
        "args": ["--platform", "{platform}", "--llvm-prefix", "{llvm_prefix}", "--out", "{out}"],  # shipped static and shared
    },
}


@dataclass(frozen=True)
class Provider:
    kind: str       # "built" (native-libs.json `libraries`) or one of KINDS
    detail: dict    # `platforms` (OS names) and the recipe's `script` and `args`; {} for built and external


@dataclass(frozen=True)
class Binding:
    name: str
    path: object                 # the file
    line: int
    kind: str | None             # the note's kind, None without a note
    platforms: tuple             # the note's OS names
    problem: str | None = None   # a note that does not parse


def load_manifest():
    """native-libs.json: the libraries that are built."""
    return json.loads(MANIFEST.read_text())


def library_name(raw):
    """The library a bound string names: without a local folder and without `lib`."""
    return raw.rsplit("/", 1)[-1].removeprefix("lib")


def parse_note(text):
    """(kind, platforms, problem) of the first `@native(...)` note in `text`, or (None, (), None)."""
    found = NOTE.search(text)
    if found is None:
        return None, (), "@native note must look like `@native(kind)`" if "@native" in text else None
    kind, *platforms = found.group(1).split(",")
    if kind not in KINDS:
        return None, (), f"`@native({found.group(1)})`: the kind must be one of {', '.join(KINDS)}"
    if any(platform not in ("macos", "linux", "windows") for platform in platforms):
        return None, (), f"`@native({found.group(1)})`: the OS names are macos, linux and windows"
    return kind, tuple(platforms), None


def bindings(stdlib):
    """Each library binding of the shipped sources (not `tests/`), by file and line."""
    for path in sorted(stdlib.rglob("*.jai")):
        if "tests" in path.relative_to(stdlib).parts:
            continue
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if line.lstrip().startswith("//"):
                continue
            for found in LIBRARY.finditer(line):
                kind, platforms, problem = parse_note(line[found.end():])
                yield Binding(library_name(found.group(1)), path, number, kind, platforms, problem)


def bound_names(stdlib):
    """Each library name the shipped sources bind, with a file that does."""
    found = {}
    for binding in bindings(stdlib):
        found.setdefault(binding.name, binding.path)
    return found


def providers(stdlib):
    """The notes' declarations by library name, and the problems with them (a note that does not parse, notes of one name
    that disagree, a `fetched`, `runtime` or `packaged` library without a recipe)."""
    found = {}
    problems = []
    for binding in bindings(stdlib):
        where = f"{binding.path}:{binding.line}"
        if binding.problem:
            problems.append(f"{where}: {binding.problem}")
        if binding.kind is None:
            continue
        provider = Provider(binding.kind, {"platforms": list(binding.platforms)} if binding.platforms else {})
        if binding.kind in ("fetched", "runtime", "packaged"):
            if binding.name not in RECIPES:
                problems.append(f"{where}: `{binding.name}` is {binding.kind} but tools/native/provided_libs.py has no recipe for it")
                continue
            provider = Provider(binding.kind, {**provider.detail, **RECIPES[binding.name]})
        if found.setdefault(binding.name, provider) != provider:
            problems.append(f"{where}: `{binding.name}` is declared {binding.kind} here and "
                            f"{found[binding.name].kind} elsewhere (or for other OSes)")
    return found, problems


def provider_of(name, manifest, declared, profiles=None):
    """The Provider of a bound name: built from `libraries` (the rpmalloc profile libraries are one
    entry, `rpmalloc`; a note on one is a mistake `check_native_libs.py` reports), else the one that declares it, else None."""
    profiles = set(rpmalloc_libraries()) if profiles is None else profiles
    if name in manifest["libraries"] or name in profiles:
        return Provider("built", {"library": "rpmalloc" if name in profiles else name})
    return declared.get(name)


def on_platform(provider, platform):
    """Whether a declaration applies to the archive of `platform` (`platforms` lists OS names)."""
    return platform.split("-")[0] in provider.detail.get("platforms", [platform.split("-")[0]])
