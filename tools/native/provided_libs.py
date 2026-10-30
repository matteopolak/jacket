"""Who provides each native library the standard library binds.

    bound_names(stdlib)             every `#library` / `#system_library` name the shipped sources bind, with a file that does
    providers(manifest)             name -> Provider for everything but the libraries built from native-libs.json
    provider_of(name, ...)          the Provider of one bound name, or None when nothing declares it
    on_platform(provider, platform) whether a declaration applies to a release archive

The declarations are tools/native/provided-libs.json, beside the `libraries` of native-libs.json that are built
(docs/tools/native-libs.md); `load_manifest()` returns native-libs.json with them under `provided`. `tools/ci/check_native_libs.py` checks the stdlib against them and
`tools/native/package_native_libs.py` packages from them.
"""

import json
import re
from dataclasses import dataclass

from native.build_native_libs import MANIFEST, rpmalloc_libraries
from toolkit.repo import ROOT

PROVIDED = ROOT / "tools" / "native" / "provided-libs.json"

KINDS = ("embedded", "fetched", "runtime", "packaged", "external")
LIBRARY = re.compile(r'#(?:system_)?library[^"\n]*"([^"\n\\]+)"')
# Path prefixes a name carries inside the stdlib's own native-library folders.
LOCAL_PREFIXES = ("native/", "crash-native/", "../crash-native/")


@dataclass(frozen=True)
class Provider:
    kind: str       # "built" (native-libs.json `libraries`) or one of KINDS
    detail: dict    # the declaration: `script`, `platforms`, `note`; {} for built and external


def load_manifest():
    """native-libs.json with the declarations of provided-libs.json as its `provided` section."""
    return {**json.loads(MANIFEST.read_text()), "provided": json.loads(PROVIDED.read_text())}


def library_name(raw):
    """The library a bound string names: without a local folder prefix and without `lib`."""
    name = raw.rsplit("/", 1)[-1] if raw.startswith(LOCAL_PREFIXES) else raw
    return name.removeprefix("lib")


def bound_names(stdlib):
    """Each library name the shipped sources (not `tests/`) bind, with a file that does."""
    found = {}
    for path in sorted(stdlib.rglob("*.jai")):
        if "tests" in path.relative_to(stdlib).parts:
            continue
        for raw in LIBRARY.findall(path.read_text(encoding="utf-8")):
            found.setdefault(library_name(raw), path)
    return found


def providers(manifest):
    """Everything the `provided` section declares, by library name."""
    found = {}
    provided = manifest["provided"]
    for kind in KINDS:
        entries = provided.get(kind, {})
        if kind == "external":
            names = {name: {"group": group} for group, group_names in entries.items() for name in group_names}
        else:
            names = {name: detail if isinstance(detail, dict) else {"note": detail} for name, detail in entries.items()}
        for name, detail in names.items():
            if name in found:
                raise SystemExit(f"{PROVIDED}: `{name}` is declared as both {found[name].kind} and {kind}")
            found[name] = Provider(kind, detail)
    return found


def provider_of(name, manifest, declared, profiles=None):
    """The Provider of a bound name: built from `libraries` (the rpmalloc profile libraries are one
    entry, `rpmalloc`), else the one that declares it, else None."""
    profiles = set(rpmalloc_libraries()) if profiles is None else profiles
    if name in manifest["libraries"] or name in profiles:
        if name in declared:
            raise SystemExit(f"{PROVIDED}: `{name}` is built and also declared as {declared[name].kind}")
        return Provider("built", {"library": "rpmalloc" if name in profiles else name})
    return declared.get(name)


def on_platform(provider, platform):
    """Whether a declaration applies to the archive of `platform` (`platforms` lists OS names)."""
    return platform.split("-")[0] in provider.detail.get("platforms", [platform.split("-")[0]])
