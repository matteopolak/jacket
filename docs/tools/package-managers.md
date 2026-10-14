# Package managers and installers

## What it is

Ways to install a released Jai toolchain (`jaic`, `jailsp`, `jailint`, `jaifmt` and the standard library) without unpacking an archive by hand. Every one installs the archives the [release workflow](releases.md) publishes, and every version and checksum comes from the release itself (its tag and its `SHA256SUMS`); nothing in this repository holds a hand-maintained version or hash.

| Channel | Platforms | Users run | Published by |
| --- | --- | --- | --- |
| Homebrew tap | macOS arm64 and x86-64, Linux x86-64 and arm64 | `brew install matteopolak/tap/jai` | `homebrew` job in `release.yml`, secret `HOMEBREW_TAP_TOKEN` |
| `install.sh` | macOS arm64 and x86-64, Linux x86-64 and arm64 | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jai/main/install.sh \| sh` | nothing to publish: it reads the latest release |
| `install.ps1` | Windows x64, arm64 | `irm https://raw.githubusercontent.com/matteopolak/jai/main/install.ps1 \| iex` | same |
| Nix flake | Linux, macOS (builds from source) | `nix profile install github:matteopolak/jai` | [Nix flake](nix.md) |

The package is named `jai` (formula `jai`, matching the VS Code extension `matteopolak.jai`) because it installs all four tools. The release archives are named after it too: `jai-<platform>.tar.gz` (`.zip` on Windows), unpacking to `jai-<platform>/`, since 0.4.1. Releases up to 0.4.0 shipped `jaic-<platform>` and keep those names, so everything that downloads an archive picks the name from the version (see [archive names](#archive-names)). Releases after 0.4.1 have Intel macOS (`macos-x64`) and arm64 Linux (`linux-arm64`) archives too; 0.4.1 and earlier do not, so for those versions Homebrew refuses the two platforms with `depends_on arch:` and the installer script says the release has no build for them (build from source, use Nix, or install a later version).

### Other ways to install

- **Archives:** every release on the [releases page](https://github.com/matteopolak/jai/releases) has one per platform. Unpack it anywhere and put `jaic` on your `PATH`, directly or through a symlink.
- **GitHub Actions:** [setup-jai](https://github.com/matteopolak/setup-jai) (`- uses: matteopolak/setup-jai@v1`).
- **From source:** you need [Rustup](https://rustup.rs/) (it picks up the pinned toolchain) and, for native builds, LLVM 23 with Clang ([LLVM setup](llvm-setup.md)):

  ```sh
  git clone https://github.com/matteopolak/jai.git && cd jai
  cargo build -p jaic-cli -p jai-language-server -p jailint --release --locked
  ```

  With [just](https://github.com/casey/just) 1.58 or newer, `just build --release` does the same and also builds `target/jaifmt` ([Justfile](justfile.md)).

## How it works

### Archive names

| Release | Archive | Top folder |
| --- | --- | --- |
| 0.4.0 and earlier | `jaic-<platform>.tar.gz` / `.zip` | `jaic-<platform>/` |
| 0.4.1 and later | `jai-<platform>.tar.gz` / `.zip` | `jai-<platform>/` |

Published assets are never renamed. The rule (`jaic` up to 0.4.0, else `jai`) lives in four places, kept in step by hand: `archive_prefix` in `install.sh`, `$prefix` in `install.ps1`, `archivePrefix` in the extension's `src/toolchain.ts`, and `expected_prefix` in `tools/render_packages.py`. `release.yml` always packages `jai-<platform>`, even on a dry run while `Cargo.toml` still says 0.4.0, so the renderer takes whichever prefix `SHA256SUMS` (or the archive folder) has for every platform, trying the version's own first. The installed layout does not depend on the name: both install scripts move the archive's top folder to `<install dir>/<version>`, so upgrading a 0.4.0 install to a `jai-` release (or back) is an ordinary upgrade.

### Rendering

`tools/render_packages.py` fills the templates in `packaging/` (`@VERSION@`, `@BASE_URL@`, `@ARCHIVE_PREFIX@` (`jai` or `jaic`), `@SHA256_<PLATFORM>@`, and for the formula's Intel macOS and arm64 Linux blocks `@MACOS_INTEL_ASSET@`/`@SHA256_MACOS_INTEL@`, `@LINUX_ARM_ASSET@`/`@SHA256_LINUX_ARM@` and `@MACOS_ARCH@`/`@LINUX_ARCH@`):

```sh
python3 tools/render_packages.py --version v0.4.0 --sums SHA256SUMS --out out
# out/homebrew/jai.rb
```

`--sums` reads a release's `SHA256SUMS`; `--archives DIR` hashes the archives in a directory instead (dry runs, before a release exists). It fails if an archive is missing from the checksums or a placeholder is unknown. The `macos-x64` and `linux-arm64` archives (`NEWER_PLATFORMS`) are required only for a version after 0.4.1; when a release lacks one, its block gets the nearest build's URL and checksum (`macos-arm64`, `linux-x64`) and a `depends_on arch:` line (`@MACOS_ARCH@` is `:arm64`, `@LINUX_ARCH@` `:x86_64`). With both present those placeholders are `None`, and `render` leaves out every line holding a `None` placeholder, plus the blank line after it. Tests: `tools/test_render_packages.py`.

### Release jobs

After `publish` creates the GitHub release, the `homebrew` job in `release.yml` runs. It depends on `publish`, so it cannot hold up the release or the extension stores; a failure only marks its own job red.

- `homebrew` (ubuntu): downloads the release's `SHA256SUMS`, renders `jai.rb`, puts it in a throwaway local tap, runs `brew style`, `brew audit --strict --online`, `brew install` and `brew test` (Linux x86-64), then clones `matteopolak/homebrew-tap` with `HOMEBREW_TAP_TOKEN`, writes `Formula/jai.rb` (creating `Formula/` on the first run) and pushes a `jai <version>` commit to its default branch. An unchanged formula is not committed.

Without its secret the job prints that it skipped and succeeds. A run without a tag (Actions, release, Run workflow, empty tag) renders from that run's archives and the `Cargo.toml` version, validates (`brew style` and offline `brew audit`) and publishes nothing.

### Homebrew formula

`packaging/homebrew/jai.rb.tmpl` downloads `jai-macos-arm64.tar.gz`, `jai-macos-x64.tar.gz`, `jai-linux-x64.tar.gz` or `jai-linux-arm64.tar.gz` (`on_macos`/`on_linux` with `on_arm`/`on_intel`; `jaic-*` when rendered for 0.4.0 or earlier), installs the four binaries, `stdlib/` (including its prelude) and `artifacts/` (the prebuilt native libraries: wgpu-native, `stb_*`, `rpmalloc`; see [native libraries](native-libs.md)) into `libexec`, and links the binaries into `bin`. Every channel installs the whole archive tree, so `artifacts/native-libs/<platform>/` stays beside `stdlib/`, where `jaic` looks for it: `install.sh` and `install.ps1` move the unpacked folder, and the formula installs `artifacts` when the archive has it (0.4.1 and later). `jaic` finds `libexec/stdlib` by resolving the `bin` symlinks to the real file (`jaic::stdlib_dir`, see [releases](releases.md#how-it-works)). Rendered for 0.4.1 or earlier, Intel macOS and arm64 Linux get the other architecture's URL anyway so the formula loads everywhere (`brew info`, tap JSON), and `depends_on arch:` refuses the install with Homebrew's own message. The `test do` block runs a hello world with `jaic -interp`, formats a line with `jaifmt --stdin` and lints with `jailint`. The license is `AGPL-3.0-or-later`; the runtime library exception has no SPDX id, so `brew audit` would reject it in the `license` line and it is noted in a comment instead.

### install.sh

POSIX `sh` (checked with `shellcheck -s sh`). It maps `uname` to an archive (`macos-arm64`, `macos-x64`, `linux-x64`, `linux-arm64`; on macOS it also detects Apple silicon under Rosetta, so a translated shell still gets the arm64 build), finds the latest version from the `releases/latest` redirect (no API rate limit), downloads `SHA256SUMS` and then the archive with `curl` or `wget` (a release whose `SHA256SUMS` lists no archive for the platform, such as 0.4.1 on an Intel Mac, stops with a message before the download), checks the hash with `sha256sum` or `shasum`, and unpacks into `~/.local/share/jai/<version>`. It then points `~/.local/bin/{jaic,jailsp,jailint,jaifmt}` at it (only the tools the archive has: 0.2.0 shipped just `jaic` and `jailsp`), deletes the versions it installed earlier, and prints how to add the bin folder to `PATH` when it is missing. It refuses to replace a bin entry that is not a symlink, so it never overwrites another install. The whole script is one function called on the last line, so a download cut short by `curl | sh` runs nothing.

```sh
JAI_VERSION=0.3.0 sh install.sh                      # a specific release (v prefix optional)
JAI_INSTALL_DIR=/opt/jai JAI_BIN_DIR=/usr/local/bin sh install.sh
```

`JAI_VERSION`, `JAI_INSTALL_DIR` and `JAI_BIN_DIR` are also read as `JAIC_VERSION`, `JAIC_INSTALL_DIR` and `JAIC_BIN_DIR` (the names before 0.4.1, still used by `packaging.yml`); the `JAI_` name wins when both are set. A version up to 0.4.0 downloads the `jaic-<platform>` archive, a later one `jai-<platform>`.

To try a change against a release that does not exist yet, repackage an archive under the new name with its own `SHA256SUMS` in a folder `<dir>/v<version>/`, and run a copy of the script with the `base=` URL replaced by `file://<dir>/v$version` and curl's `--proto '=https' --tlsv1.2` removed (curl reads `file://` URLs); the script itself has no URL override.

Uninstall: `rm -rf ~/.local/share/jai ~/.local/bin/{jaic,jailsp,jailint,jaifmt}`.

### install.ps1

Runs in Windows PowerShell 5.1 and PowerShell 7 (`irm ... | iex`, so it never calls `exit`). It picks `windows-x64` or `windows-arm64` from the OS architecture (not the process's, which may be emulated), finds the latest version from the `releases/latest` redirect (no API rate limit, as in `install.sh`), verifies the zip against `SHA256SUMS`, unpacks it into `%LOCALAPPDATA%\Programs\jai\<version>`, points the directory junction `%LOCALAPPDATA%\Programs\jai\current` at it, adds `current` to the user `PATH` once, and deletes older versions. A junction needs no administrator rights or developer mode, unlike a symbolic link, and `jaic.exe` started through it finds `current\stdlib`. `JAI_VERSION` and `JAI_INSTALL_DIR` (or `JAIC_VERSION` and `JAIC_INSTALL_DIR`) work as in `install.sh`, as does the archive name.

### Validation in CI

`.github/workflows/packaging.yml` runs when these files change (and by hand), always against the latest published release:

- `shellcheck`: `install.sh` as POSIX sh.
- `install.sh` on ubuntu-24.04, ubuntu-24.04-arm, macos-15 and macos-15-intel: where the latest release has no archive for the runner's platform (0.4.1 on the arm64 Linux and Intel macOS runners), only checks that `install.sh` fails naming that archive; otherwise installs the previous release, upgrades to the latest through `sh < install.sh` (checking that the old version is removed and the `PATH` advice is printed), runs `jaic -interp`, `jailint` and `jaifmt` from another directory, re-runs with custom directories, and checks the failures for a missing version and an occupied bin entry.
- `install-ps1` on windows-2025: installs twice under Windows PowerShell 5.1, then runs the tools from `PATH` under PowerShell 7.
- `homebrew` on macos-15 and ubuntu-24.04: renders a formula from synthetic checksums for all four archives and checks it has the Intel macOS and arm64 Linux URLs and no `depends_on arch:` (`brew style`, offline `brew audit --strict`); then renders the latest release's formula and runs `brew style`, `brew audit --strict --online`, `brew install --build-from-source`, `brew test` and `brew uninstall` from a local tap.

Nothing in `packaging.yml` publishes.

## How to change it

- Formula: edit the template in `packaging/`, then render against the latest release and check locally (`brew tap-new --no-git you/local`, copy `out/homebrew/jai.rb` into its `Formula/`, `brew style`, `brew audit --formula --strict --online you/local/jai`, `brew install`, `brew test`, `brew uninstall`, `brew untap`). A new placeholder needs a value in `substitutions()` in `tools/render_packages.py`; an unknown one is an error.
- A new platform archive: add it to `PLATFORM_ASSETS` in `tools/render_packages.py` (or, if older releases lack it and their formulas must still render, to `NEWER_PLATFORMS` with a fallback), to the formula (`on_macos`/`on_linux` with `on_arm`/`on_intel`) or the installer list, to the `platform` functions of both install scripts, to `RELEASE_ASSETS` in the extension's `src/toolchain.ts`, and to `packaging.yml`'s runners.

Gotchas:

- A release's assets must stay as they are: re-uploading an archive changes its hash and breaks that version's formula.
- The formula is not bottled: `brew install` unpacks the release archive, so installs need no build tools.

## Configuration

One-time setup. `matteopolak/homebrew-tap` (public, with a README) already exists; only the repository secret remains (Settings, Secrets and variables, Actions):

| Secret | What | Scope |
| --- | --- | --- |
| `HOMEBREW_TAP_TOKEN` | fine-grained personal access token | repository `matteopolak/homebrew-tap` only, Contents: read and write |

Other knobs: `JAI_VERSION`, `JAI_INSTALL_DIR`, `JAI_BIN_DIR` (install scripts; also as `JAIC_*`), the identifier `REPOSITORY` in `tools/render_packages.py`.

### No winget

winget support was dropped before any release was accepted into winget-pkgs: the release binaries are unsigned and new every release, so every submission to `microsoft/winget-pkgs` failed Microsoft Defender validation (`Validation-Defender-Error`). Do not re-add it without Windows code signing. Windows users install with `install.ps1` or the release zip.

## Dependencies

The GitHub release assets (`jai-*.tar.gz`/`.zip`, `jaic-*` up to 0.4.0, and `SHA256SUMS`), Homebrew on the runners, Python 3 for the renderer, and `curl`/`wget`, `tar` and `sha256sum`/`shasum` on users' machines for `install.sh`.
