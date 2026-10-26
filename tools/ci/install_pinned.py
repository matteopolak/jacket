#!/usr/bin/env python3
"""Install a tool at the version .github/pins.json pins, verifying what it downloads.

    install_pinned.py nextest | llvm-cov | cargo-mutants   prebuilt cargo subcommand into $CARGO_HOME/bin
    install_pinned.py rust [--component NAME]... [--target TRIPLE]...
    install_pinned.py llvm-release ARCHIVE

The prebuilt tools are downloaded, compared with the SHA-256 in pins.json for this platform and only then
unpacked into `${CARGO_HOME:-~/.cargo}/bin`, which is on PATH wherever rustup is. A platform without a
pinned checksum fails with the `cargo install --locked` command to use instead.

`rust` installs the channel rust-toolchain.toml names (minimal profile) from this checkout, so the
channel is written in one place.

`llvm-release` downloads the official LLVM release archive ARCHIVE (kept in $RUNNER_TEMP/llvm-download,
retried; nothing caches it between runs), unpacks it into $RUNNER_TEMP/llvm and appends LLVM_SYS_231_PREFIX to
$GITHUB_ENV. It serves ci.yml's Windows setup and release.yml's static builds.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path

from ci import pins
from toolkit.download import USER_AGENT, fetch
from toolkit.hashing import sha256_bytes
from toolkit.host import host_cpu, host_os
from toolkit.repo import ROOT


@dataclass(frozen=True)
class Prebuilt:
    """A release archive of one cargo subcommand: `url` takes {version} and the platform's {target}."""

    version_pin: str
    url: str
    # host ("linux-x64", "linux-arm64", "macos") -> (the {target} in the url, the pins.json key of its SHA-256)
    hosts: dict[str, tuple[str, str]]
    binary: str
    version_command: list[str]


PREBUILT = {
    "nextest": Prebuilt(
        version_pin="nextest",
        url="https://get.nexte.st/{version}/{target}",
        hosts={
            "linux-x64": ("linux", "nextest_sha256_linux_x64"),
            "linux-arm64": ("linux-arm", "nextest_sha256_linux_arm64"),
            "macos": ("mac", "nextest_sha256_macos"),
        },
        binary="cargo-nextest",
        version_command=["nextest", "--version"],
    ),
    "llvm-cov": Prebuilt(
        version_pin="llvm_cov",
        url="https://github.com/taiki-e/cargo-llvm-cov/releases/download/v{version}/cargo-llvm-cov-{target}.tar.gz",
        hosts={
            "linux-x64": ("x86_64-unknown-linux-gnu", "llvm_cov_sha256_linux_x64"),
            "linux-arm64": ("aarch64-unknown-linux-gnu", "llvm_cov_sha256_linux_arm64"),
            "macos": ("universal-apple-darwin", "llvm_cov_sha256_macos"),
        },
        binary="cargo-llvm-cov",
        version_command=["llvm-cov", "--version"],
    ),
    "cargo-mutants": Prebuilt(
        version_pin="cargo_mutants",
        url="https://github.com/sourcefrog/cargo-mutants/releases/download/v{version}/cargo-mutants-{target}.tar.gz",
        hosts={
            "linux-x64": ("x86_64-unknown-linux-gnu", "cargo_mutants_sha256_linux_x64"),
        },
        binary="cargo-mutants",
        version_command=["mutants", "--version"],
    ),
}


def host_key() -> str:
    system = host_os()
    return "macos" if system == "macos" else f"{system}-{host_cpu()}"


def cargo_bin() -> Path:
    return Path(os.environ.get("CARGO_HOME") or Path.home() / ".cargo") / "bin"


def install_prebuilt(name: str, manifest: dict, dest: Path) -> None:
    tool = PREBUILT[name]
    version = manifest[tool.version_pin]
    host = host_key()
    if host not in tool.hosts:
        sys.exit(
            f"no prebuilt cargo-{name} pinned for {host}; use cargo install --locked cargo-{name.removeprefix('cargo-')}@{version}"
        )
    target, key = tool.hosts[host]
    body = fetch(tool.url.format(version=version, target=target), attempts=4)
    actual = sha256_bytes(body)
    if actual != manifest[key]:
        sys.exit(
            f"{name} {version} ({target}): checksum {actual} does not match {key} in .github/pins.json"
        )
    dest.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(suffix=".tgz", delete=False) as archive:
        archive.write(body)
    try:
        with tarfile.open(archive.name, "r:gz") as tar:
            extract(tar, dest)
    finally:
        os.unlink(archive.name)
    subprocess.run([str(dest / tool.binary), *tool.version_command], check=True)


def extract(tar: tarfile.TarFile, dest: Path, strip: int = 0) -> None:
    """Unpack `tar` into `dest`, dropping `strip` leading path components of every member (symbolic links stay as they are)."""
    for member in tar:
        parts = Path(member.name).parts[strip:]
        if not parts:
            continue
        member.name = Path(*parts).as_posix()
        if member.islnk():  # a hard link names another member of the archive
            member.linkname = Path(*Path(member.linkname).parts[strip:]).as_posix()
        tar.extract(
            member,
            dest,
            **({"filter": "tar"} if hasattr(tarfile, "tar_filter") else {}),
        )


def rust_channel(root: Path) -> str:
    match = re.search(
        r'^channel *= *"(.*)"', (root / "rust-toolchain.toml").read_text(), re.M
    )
    if not match:
        sys.exit("no channel in rust-toolchain.toml")
    return match.group(1)


def install_rust(root: Path, components: list[str], targets: list[str]) -> None:
    channel = rust_channel(root)
    command = ["rustup", "toolchain", "install", channel, "--profile", "minimal"]
    for component in components:
        command += ["--component", component]
    subprocess.run(command, check=True)
    for target in targets:
        subprocess.run(
            ["rustup", "target", "add", target, "--toolchain", channel], check=True
        )


def download(url: str, path: Path, attempts: int = 5, pause: int = 5) -> None:
    """Stream `url` to `path` (the archives are too large to hold in memory), retrying transient failures."""
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_name(path.name + ".part")
    for attempt in range(1, attempts + 1):
        try:
            with (
                urllib.request.urlopen(
                    urllib.request.Request(url, headers={"User-Agent": USER_AGENT}),
                    timeout=120,
                ) as response,
                partial.open("wb") as out,
            ):
                shutil.copyfileobj(response, out, 1 << 20)
            partial.replace(path)
            return
        except (OSError, urllib.error.URLError):
            if attempt == attempts:
                raise
            print(f"  attempt {attempt} failed; retrying in {pause}s", file=sys.stderr)
            time.sleep(pause)


def install_llvm_release(manifest: dict, archive: str) -> None:
    version = manifest["llvm"]
    temp = Path(os.environ["RUNNER_TEMP"])
    download_dir = temp / "llvm-download"
    llvm = temp / "llvm"
    if not (download_dir / archive).is_file():
        download(
            f"https://github.com/llvm/llvm-project/releases/download/llvmorg-{version}/{archive}",
            download_dir / archive,
        )
    llvm.mkdir(parents=True, exist_ok=True)
    with tarfile.open(download_dir / archive, "r:xz") as tar:
        extract(tar, llvm, strip=1)
    with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as env:
        env.write(f"LLVM_SYS_231_PREFIX={llvm.as_posix()}\n")
    config = (
        llvm / "bin" / ("llvm-config.exe" if host_os() == "windows" else "llvm-config")
    )
    print(
        subprocess.run(
            [str(config), "--version"], check=True, capture_output=True, text=True
        ).stdout.strip()
    )
    libs = subprocess.run(
        [str(config), "--link-static", "--system-libs"],
        check=True,
        capture_output=True,
        text=True,
    )
    print("system libraries:", libs.stdout.strip())


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="tool", required=True)
    for name in PREBUILT:
        sub.add_parser(name)
    rust = sub.add_parser("rust")
    rust.add_argument("--component", action="append", default=[], metavar="NAME")
    rust.add_argument("--target", action="append", default=[], metavar="TRIPLE")
    sub.add_parser("llvm-release").add_argument("archive")
    args = parser.parse_args(argv)
    manifest = pins.load()
    if args.tool in PREBUILT:
        install_prebuilt(args.tool, manifest, cargo_bin())
    elif args.tool == "rust":
        install_rust(ROOT, args.component, args.target)
    else:
        install_llvm_release(manifest, args.archive)
    return 0


if __name__ == "__main__":
    sys.exit(main())
