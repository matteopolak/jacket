#!/usr/bin/env python3
"""Check Cargo-managed authored Rust sources and present standalone packages."""
from fnmatch import fnmatch
from pathlib import Path
import subprocess
import sys
import tomllib


STANDALONE_MANIFESTS = ("fuzz/Cargo.toml",)


def standalone_manifests(root: Path) -> list[str]:
    return [manifest for manifest in STANDALONE_MANIFESTS if (root / manifest).is_file()]


def uncovered_packages(root: Path) -> list[str]:
    workspace = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]
    covered = {root / manifest for manifest in standalone_manifests(root)}
    for pattern in workspace["members"]:
        for directory in root.glob(pattern):
            relative = directory.relative_to(root).as_posix()
            if not any(fnmatch(relative, excluded) for excluded in workspace.get("exclude", [])):
                covered.add(directory / "Cargo.toml")

    authored = {path for path in (root / "crates").rglob("Cargo.toml")
                if "package" in tomllib.loads(path.read_text())}
    authored.update(root / manifest for manifest in standalone_manifests(root))
    return sorted(path.relative_to(root).as_posix() for path in authored - covered)


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    try:
        uncovered = uncovered_packages(root)
        if uncovered:
            print("Authored Rust packages missing from the declared formatter scope:", file=sys.stderr)
            for path in uncovered:
                print(f"  {path}", file=sys.stderr)
            print("Register them in the main workspace or add an explicit standalone format command.",
                  file=sys.stderr)
            return 1

        commands = [["cargo", "fmt", "--all", "--", "--check"]]
        commands.extend(["cargo", "fmt", "--manifest-path", manifest, "--", "--check"]
                        for manifest in standalone_manifests(root))
        results = [subprocess.run(command, cwd=root, check=False).returncode
                   for command in commands]
        return int(any(results))
    except (OSError, ValueError, KeyError) as error:
        print(f"Rust formatting check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
