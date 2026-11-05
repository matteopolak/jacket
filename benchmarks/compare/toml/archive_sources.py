"""Archive the owned sources authenticated by a benchmark build, without executing its clients.

python3 benchmarks/compare/toml/archive_sources.py --build /tmp/toml-bench/build.json
"""

import argparse
import hashlib
import json
from pathlib import Path
import tarfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def archive_sources(root, build):
    """Keep a build's exact source receipt beside its binary; an existing archive is preserved.

    Example: archive_sources(Path.cwd(), Path('/tmp/toml-bench/build.json')).
    """
    root = root.resolve()
    build_hash = digest(build)
    receipt = json.loads(build.read_text())
    sources = receipt["sources"]
    if not sources:
        raise RuntimeError("build has no owned sources")
    binary = build.parent / "jai-batch"
    compiler = Path(receipt["command"][0])
    if digest(binary) != receipt["binary_sha256"]:
        raise RuntimeError("benchmark binary differs from its build receipt")
    if digest(compiler) != receipt["compiler_sha256"]:
        raise RuntimeError("compiler differs from its build receipt")
    paths = {}
    for name, expected in sources.items():
        relative = Path(name)
        if not name or relative.is_absolute() or ".." in relative.parts:
            raise RuntimeError(
                "source receipt paths must be relative to the repository"
            )
        path = root / relative
        if not path.resolve().is_relative_to(root) or not path.is_file():
            raise RuntimeError(
                "source receipt path escapes the repository or is not a file"
            )
        if digest(path) != expected:
            raise RuntimeError("owned source differs from build receipt: " + name)
        paths[name] = path
    archive = build.parent / "frozen-owned-sources.tar.gz"
    with tarfile.open(archive, "x:gz") as output:
        for name, path in paths.items():
            output.add(path, arcname=name, recursive=False)
    # Verify archived bytes too, rather than assuming files stayed unchanged during copying.
    with tarfile.open(archive) as output:
        actual = {}
        for member in output:
            if not member.isfile() or member.name in actual:
                raise RuntimeError(
                    "source archive contains an invalid or duplicate entry"
                )
            actual[member.name] = hashlib.sha256(
                output.extractfile(member).read()
            ).hexdigest()
    if actual != sources or any(
        digest(path) != sources[name] for name, path in paths.items()
    ):
        raise RuntimeError(
            "owned source changed while archiving; failed archive retained"
        )
    if (
        digest(build) != build_hash
        or digest(binary) != receipt["binary_sha256"]
        or digest(compiler) != receipt["compiler_sha256"]
    ):
        raise RuntimeError("build receipt or executable changed while archiving")
    return archive


def main():
    """Archive one fresh build receipt; for example, --build /tmp/toml-bench/build.json."""
    if not __debug__:
        raise RuntimeError(
            "Run without Python -O to retain the benchmark receipt guards"
        )
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[3]
    )
    parser.add_argument("--build", type=Path, required=True)
    args = parser.parse_args()
    print(archive_sources(args.root, args.build.resolve()))


if __name__ == "__main__":
    main()
