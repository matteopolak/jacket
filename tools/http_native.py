#!/usr/bin/env python3
"""Prepare HTTP/Io/Tls's native dependency closure using the pinned SDK builder.

mbedTLS includes the repository's complete TLS configuration and C adapter. SDL
and graphics profiles belong to their independent consumers and remain intact.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path

import build_native_libs as native


def native_profile():
    manifest = json.loads(native.MANIFEST.read_text())
    library = manifest["libraries"]["mbedtls"]
    source = library["source"]
    return {
        "libraries": {"mbedtls": library},
        "sources": {source: manifest["sources"][source]},
    }


def required_files(platform):
    return native.shipped_files(native_profile(), platform)


def profile_identity(profile, platform):
    local = native.ROOT / profile["libraries"]["mbedtls"]["local"]
    files = sorted(path for path in local.rglob("*") if path.is_file())
    inputs = {
        "platform": platform,
        "profile": profile,
        "local": {
            str(path.relative_to(native.ROOT)): hashlib.sha256(
                path.read_bytes()
            ).hexdigest()
            for path in files
        },
        "builder": hashlib.sha256(Path(native.__file__).read_bytes()).hexdigest(),
    }
    return hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()


def artifact_hashes(directory, required):
    return {
        name: hashlib.sha256((directory / name).read_bytes()).hexdigest()
        for name in required
    }


def complete_provider(directory, required, identity, default):
    if not all((directory / name).is_file() for name in required):
        return False
    receipt = directory / "http-native-profile.json"
    if not receipt.exists():
        return (
            directory != default
        )  # Explicit external providers retain the CLI trust contract.
    try:
        data = json.loads(receipt.read_text())
        if not isinstance(data, dict):
            return False
        return data.get("identity") == identity and data.get(
            "artifacts"
        ) == artifact_hashes(directory, required)
    except (OSError, ValueError):
        return False  # Invalid identity is rebuilt; SDK/build failures remain fatal.


def ensure_native_libraries(output=None, cache=None):
    platform = native.host_dir()
    profile = native_profile()
    library = profile["libraries"]["mbedtls"]
    if not native.wanted(library, platform):
        raise RuntimeError(f"HTTP native SDK profile is not available for {platform}")
    required = native.shipped_files(profile, platform)
    identity = profile_identity(profile, platform)
    existing = [
        Path(path).resolve()
        for path in os.environ.get("JAIC_NATIVE_LIBS", "").split(os.pathsep)
        if path
    ]
    target = Path(os.environ.get("CARGO_TARGET_DIR", native.ROOT / "target")).resolve()
    default = target / "http-native" / platform
    if output is None:
        # An explicit complete provider remains caller-owned. Prefer it as a
        # whole over a preceding partial directory to keep archive/DLL identity.
        output = next(
            (
                path
                for path in existing
                if complete_provider(path, required, identity, default)
            ),
            default,
        )
    output = Path(output).resolve()
    if not complete_provider(output, required, identity, default):
        output.mkdir(parents=True, exist_ok=True)
        source_name = library["source"]
        source_spec = profile["sources"][source_name]
        source_cache = (
            Path(cache).resolve() if cache else target / "http-native-sources"
        )
        source = native.fetch(source_spec, source_cache / source_name)
        native.build("mbedtls", library, source, output, platform)
        native.copy_licenses(source_name, source_spec, source, output)
        missing = [name for name in required if not (output / name).is_file()]
        if missing:
            raise RuntimeError(
                f"HTTP native SDK build missing required artifacts: {missing}"
            )
        (output / "http-native-profile.json").write_text(
            json.dumps(
                {"identity": identity, "artifacts": artifact_hashes(output, required)},
                indent=2,
            )
            + "\n"
        )
    missing = [name for name in required if not (output / name).is_file()]
    if missing:
        raise RuntimeError(
            f"HTTP native SDK build missing required artifacts: {missing}"
        )
    paths = [output, *(path for path in existing if path != output)]
    os.environ["JAIC_NATIVE_LIBS"] = os.pathsep.join(map(str, paths))
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, help="HTTP native library directory")
    parser.add_argument("--cache", type=Path, help="pinned SDK source cache directory")
    args = parser.parse_args()
    print("HTTP native SDK provider:", ensure_native_libraries(args.out, args.cache))


if __name__ == "__main__":
    main()
