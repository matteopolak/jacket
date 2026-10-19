#!/usr/bin/env python3
"""Does the repository have a published release the installer jobs can install, and does its archive ship `jacket`?

The CI jobs that install a published release (install.sh, install.ps1, the Homebrew formula) skip those
steps with a notice when there is none yet; any other `gh` failure still fails the job. Writes
`published=true|false` (and, with --formula, `installable=true|false`) to $GITHUB_OUTPUT, and the notices
to $GITHUB_STEP_SUMMARY.

    python3 tools/ci/published_release.py [--formula]   # needs GH_TOKEN, GITHUB_REPOSITORY, and for --formula RUNNER_OS and RUNNER_ARCH
"""
import argparse
import os
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

PLATFORMS = {
    "Linux-X64": "linux-x64",
    "Linux-ARM64": "linux-arm64",
    "macOS-X64": "macos-x64",
    "macOS-ARM64": "macos-arm64",
}


def gh(*args):
    return subprocess.run(["gh", *args], capture_output=True, text=True)


def notice(title, message):
    print(f"::notice title={title}::{message}")
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a") as out:
            out.write(message + "\n")


def check(repo, platform=None, run=gh, scratch=None):
    """(published, installable): `installable` is None unless `platform` names the archive to inspect."""
    latest = run("release", "view", "--repo", repo, "--json", "tagName", "-q", ".tagName")
    if latest.returncode != 0:
        if "release not found" in latest.stderr.lower():
            notice(
                "No published release",
                f"{repo} has no published release yet, so the steps that install a published release are skipped; "
                "the locally built archive steps still run.",
            )
            return False, None
        sys.stderr.write(latest.stderr)
        raise SystemExit(1)
    tag = latest.stdout.strip()
    print(f"Latest release: {tag}")
    assets = run("release", "view", tag, "--repo", repo, "--json", "assets", "-q", ".assets[].name")
    if assets.returncode != 0:
        sys.stderr.write(assets.stderr)
        raise SystemExit(1)
    # Releases cut before the archives were named jacket-<platform> cannot be installed by the current installers either.
    if not any(name.startswith("jacket-") for name in assets.stdout.split()):
        notice(
            "No installable release",
            f"{repo} release {tag} predates the jacket-<platform> archives, so the steps that install a published "
            "release are skipped; the locally built archive steps still run.",
        )
        return False, None
    if platform is None:
        return True, None
    # The formula is rendered from this checkout's template, so it can install a binary (such as `jacket`)
    # that the latest release's archive does not ship yet.
    archive = f"jacket-{platform}.tar.gz"
    directory = Path(scratch or tempfile.mkdtemp())
    download = run("release", "download", tag, "--repo", repo, "--pattern", archive, "--dir", str(directory))
    if download.returncode != 0:
        sys.stderr.write(download.stderr)
        raise SystemExit(1)
    with tarfile.open(directory / archive) as tar:
        names = tar.getnames()
    if f"jacket-{platform}/jacket" in names:
        return True, True
    notice(
        "Release older than the formula",
        f"{repo} release {tag} has no jacket binary, which the formula template installs, so the formula install "
        "and test are skipped until the next release.",
    )
    return True, False


def write_outputs(published, installable):
    lines = [f"published={str(published).lower()}"]
    if installable is not None:
        lines.append(f"installable={str(installable).lower()}")
    path = os.environ.get("GITHUB_OUTPUT")
    if path:
        with open(path, "a") as out:
            out.write("\n".join(lines) + "\n")
    print("\n".join(lines))


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--formula", action="store_true", help="also check that this runner's archive ships `jacket`")
    args = parser.parse_args()
    platform = None
    if args.formula:
        key = f"{os.environ['RUNNER_OS']}-{os.environ['RUNNER_ARCH']}"
        if key not in PLATFORMS:
            raise SystemExit(f"no release archive for {key}")
        platform = PLATFORMS[key]
    write_outputs(*check(os.environ["GITHUB_REPOSITORY"], platform, scratch=os.environ.get("RUNNER_TEMP")))
    return 0


if __name__ == "__main__":
    sys.exit(main())
