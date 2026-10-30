#!/usr/bin/env python3
"""Every native library the shipped standard library binds must come with the release archive.

    python3 tools/ci/check_native_libs.py --stdlib stdlib --platform linux-x64 [--native DIR]

Scans `stdlib/` (not `tests/`) for `#library`, `#system_library`, `#foreign_library` and `#foreign_system_library`
names and looks each up in tools/native/native-libs.json or in the `@native(<kind>)` note on the line that binds it
(docs/tools/native-libs.md, tools/native/provided_libs.py):

- `libraries` (native-libs.json): built from pinned sources for the release (including the rpmalloc profile libraries).
  With `--native`, its static archive must be in that directory (the archive's `native-libs/<platform>`),
  when the library is built for the platform. Its binding carries no note.
- a note: `embedded` in `jaic`, `fetched` prebuilt (wgpu-native, whose archive must then be present), built into
  `stdlib/` by a `runtime` step, `packaged` by a script (its static and shared library must be present), built by the
  `developer` checks, or `external` (the operating system's, the SDK's or the user's).

A name that is neither built nor carries a note fails, and so does a note that does not parse, one that disagrees with
another note of the same name, or a name that is built and also carries one.
"""

import argparse
import sys
from pathlib import Path

from native.build_native_libs import static_name, wanted
from native.provided_libs import bound_names, load_manifest, on_platform, provider_of, providers

SHARED_SUFFIX = {"macos": "dylib", "linux": "so", "windows": "dll"}


def shared_name(name: str, platform: str) -> str:
    return f"lib{name}.{SHARED_SUFFIX[platform.split('-')[0]]}"


def check(stdlib: Path, platform: str, native: Path | None, manifest: dict | None = None) -> list[str]:
    manifest = manifest or load_manifest()
    libraries = manifest["libraries"]
    declared, problems = providers(stdlib)
    for name, path in bound_names(stdlib).items():
        provider = provider_of(name, manifest, declared)
        if provider is None:
            problems.append(f"{path}: `{name}` is bound but neither built by `libraries` nor declared by an `@native(<kind>)` "
                            "note on the line that binds it")
            continue
        if provider.kind == "built" and name in declared:
            problems.append(f"{path}: `{name}` is built by tools/native/native-libs.json and also declared "
                            f"`@native({declared[name].kind})`")
        if native is None:
            continue
        if expected_static(provider, libraries, platform) and not (native / static_name(name, platform)).is_file():
            problems.append(f"{path}: `{name}` is not in {native} ({static_name(name, platform)})")
        if provider.kind == "packaged" and on_platform(provider, platform):
            # Shipped as static and shared libraries by a script (Bindings_Generator's helper,
            # tools/ci/package_bindings_helpers.py).
            for file in (static_name(name, platform), shared_name(name, platform)):
                if not (native / file).is_file():
                    problems.append(f"{path}: `{name}` is not in {native} ({file})")
    return problems


def expected_static(provider, libraries, platform: str) -> bool:
    """Whether the archive of `platform` must hold the bound library's static archive."""
    if provider.kind == "built":
        return wanted(libraries[provider.detail["library"]], platform)
    return provider.kind == "fetched" and on_platform(provider, platform)


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
