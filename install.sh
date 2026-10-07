#!/bin/sh
# Installs the Jai toolchain (jaic, jailsp, jailint, jaifmt) from a GitHub release of
# https://github.com/matteopolak/jai, verified against the release's SHA256SUMS:
#
#   curl -fsSL https://raw.githubusercontent.com/matteopolak/jai/main/install.sh | sh
#
# Environment:
#   JAIC_VERSION      release to install, e.g. 0.4.0 or v0.4.0 (default: the latest release)
#   JAIC_INSTALL_DIR  where releases are unpacked, one folder per version (default: ~/.local/share/jai)
#   JAIC_BIN_DIR      where the command symlinks go (default: ~/.local/bin)
#
# Running it again installs the requested (or latest) version, points the symlinks at it and
# removes the versions it installed before. See docs/tools/package-managers.md.
set -eu

REPO="matteopolak/jai"
TOOLS="jaic jailsp jailint jaifmt"

say() { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

download() { # url dest
    if command -v curl > /dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$2" "$1"
    elif command -v wget > /dev/null 2>&1; then
        wget -q -O "$2" "$1"
    else
        fail "neither curl nor wget is installed"
    fi
}

latest_version() {
    # The /releases/latest page redirects to /releases/tag/v<version>; no API rate limit applies.
    if command -v curl > /dev/null 2>&1; then
        url="$(curl --proto '=https' --tlsv1.2 -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest")"
    else
        url="$(wget -q -S --max-redirect=0 -O /dev/null "https://github.com/$REPO/releases/latest" 2>&1 | sed -n 's/^ *[Ll]ocation: *//p' | tail -n 1)"
    fi
    url="$(printf '%s' "$url" | tr -d '\r')"
    case "$url" in
        */tag/v*) printf '%s\n' "${url##*/tag/v}" ;;
        *) fail "could not find the latest release of $REPO (got '$url'); set JAIC_VERSION" ;;
    esac
}

sha256() {
    if command -v sha256sum > /dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif command -v shasum > /dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        fail "neither sha256sum nor shasum is installed, so the download cannot be verified"
    fi
}

platform() {
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$os" in
        Darwin)
            # A shell running under Rosetta reports x86_64 on Apple silicon.
            if [ "$arch" = x86_64 ] && [ "$(sysctl -n sysctl.proc_translated 2> /dev/null || echo 0)" = 1 ]; then
                arch=arm64
            fi
            case "$arch" in
                arm64 | aarch64) echo macos-arm64 ;;
                *) fail "there is no prebuilt Jai toolchain for macOS on Intel ($arch): releases ship Apple silicon builds only.
Build from source instead: https://github.com/$REPO#install (or use the Nix flake)." ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64 | amd64) echo linux-x64 ;;
                *) fail "there is no prebuilt Jai toolchain for Linux on $arch: releases ship x86-64 builds only.
Build from source or use the Nix flake instead: https://github.com/$REPO#install" ;;
            esac
            ;;
        MINGW* | MSYS* | CYGWIN* | Windows_NT)
            fail "on Windows, use winget (winget install matteopolak.jai) or install.ps1:
  irm https://raw.githubusercontent.com/$REPO/main/install.ps1 | iex" ;;
        *) fail "there is no prebuilt Jai toolchain for $os ($arch); see https://github.com/$REPO#install" ;;
    esac
}

main() {
    plat="$(platform)"
    version="${JAIC_VERSION:-}"
    version="${version#v}"
    if [ -z "$version" ] || [ "$version" = latest ]; then
        version="$(latest_version)"
    fi
    case "$version" in
        *[!0-9A-Za-z.-]* | "") fail "JAIC_VERSION '$version' is not a version like 0.4.0" ;;
    esac
    install_dir="${JAIC_INSTALL_DIR:-$HOME/.local/share/jai}"
    bin_dir="${JAIC_BIN_DIR:-$HOME/.local/bin}"
    archive="jaic-$plat.tar.gz"
    base="https://github.com/$REPO/releases/download/v$version"

    tmp="$(mktemp -d 2> /dev/null || mktemp -d -t jai-install)"
    trap 'rm -rf "$tmp"' EXIT
    trap 'exit 130' INT TERM

    say "Downloading jai $version ($archive)"
    download "$base/SHA256SUMS" "$tmp/SHA256SUMS" || fail "could not download $base/SHA256SUMS (is $version a release?)"
    download "$base/$archive" "$tmp/$archive" || fail "could not download $base/$archive"
    expected="$(awk -v f="$archive" '$2 == f || $2 == "*" f { print $1 }' "$tmp/SHA256SUMS")"
    [ -n "$expected" ] || fail "SHA256SUMS of $version has no entry for $archive"
    actual="$(sha256 "$tmp/$archive")"
    [ "$actual" = "$expected" ] || fail "checksum mismatch for $archive: expected $expected, got $actual"
    say "Verified SHA-256 $actual"

    mkdir -p "$tmp/unpack"
    tar -xzf "$tmp/$archive" -C "$tmp/unpack"
    [ -x "$tmp/unpack/jaic-$plat/jaic" ] || fail "$archive does not contain jaic-$plat/jaic"

    mkdir -p "$install_dir" "$bin_dir"
    # Refuse to replace a command that is not one of our symlinks (another install of jaic).
    for tool in $TOOLS; do
        link="$bin_dir/$tool"
        if [ -e "$link" ] && [ ! -L "$link" ]; then
            fail "$link exists and is not a symlink; remove it or set JAIC_BIN_DIR"
        fi
    done
    target="$install_dir/$version"
    rm -rf "$target.new"
    mv "$tmp/unpack/jaic-$plat" "$target.new"
    rm -rf "$target"
    mv "$target.new" "$target"
    linked=""
    for tool in $TOOLS; do
        if [ -x "$target/$tool" ]; then # older releases lack jaifmt
            ln -sfn "$target/$tool" "$bin_dir/$tool"
            linked="$linked $tool"
        fi
    done
    # Versions installed by earlier runs; the symlinks no longer point into them.
    for old in "$install_dir"/[0-9]*; do
        if [ -d "$old" ] && [ "$old" != "$target" ] && [ -x "$old/jaic" ] && [ -d "$old/stdlib" ]; then
            rm -rf "$old"
            say "Removed the previous version in $old"
        fi
    done

    say "Installed jai $version into $target"
    say "Linked$linked into $bin_dir"
    case ":${PATH:-}:" in
        *":$bin_dir:"*) say "Try it: jaic run hello.jai" ;;
        *)
            say ""
            say "$bin_dir is not on your PATH. Add it, for example:"
            case "${SHELL:-}" in
                */fish) say "  fish_add_path $bin_dir" ;;
                */zsh) say "  echo 'export PATH=\"$bin_dir:\$PATH\"' >> ~/.zshrc" ;;
                *) say "  echo 'export PATH=\"$bin_dir:\$PATH\"' >> ~/.profile" ;;
            esac
            say "then open a new shell and run: jaic run hello.jai"
            ;;
    esac
}

main "$@"
