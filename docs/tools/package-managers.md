# Package managers and installers

## What it is

Ways to install a released Jai toolchain (`jaic`, `jailsp`, `jailint`, `jaifmt` and the standard library) without unpacking an archive by hand. Every one installs the archives the [release workflow](releases.md) publishes, and every version and checksum comes from the release itself (its tag and its `SHA256SUMS`); nothing in this repository holds a hand-maintained version or hash.

| Channel | Platforms | Users run | Published by |
| --- | --- | --- | --- |
| Homebrew tap | macOS arm64, Linux x86-64 | `brew install matteopolak/tap/jai` | `homebrew` job in `release.yml`, secret `HOMEBREW_TAP_TOKEN` |
| winget | Windows x64, arm64 | `winget install matteopolak.jai` | `winget` job in `release.yml`, secret `WINGET_TOKEN` |
| `install.sh` | macOS arm64, Linux x86-64 | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jai/main/install.sh \| sh` | nothing to publish: it reads the latest release |
| `install.ps1` | Windows x64, arm64 | `irm https://raw.githubusercontent.com/matteopolak/jai/main/install.ps1 \| iex` | same |
| Nix flake | Linux, macOS (builds from source) | `nix profile install github:matteopolak/jai` | [Nix flake](nix.md) |

The package is named `jai` (formula `jai`, winget `matteopolak.jai`, matching the VS Code extension `matteopolak.jai`) because it installs all four tools. The release archives are named after it too: `jai-<platform>.tar.gz` (`.zip` on Windows), unpacking to `jai-<platform>/`, since 0.4.1. Releases up to 0.4.0 shipped `jaic-<platform>` and keep those names, so everything that downloads an archive picks the name from the version (see [archive names](#archive-names)). There is no Intel macOS or arm64 Linux build: Homebrew refuses those with `depends_on arch:` and the installer script says so (build from source, or use Nix).

## How it works

### Archive names

| Release | Archive | Top folder |
| --- | --- | --- |
| 0.4.0 and earlier | `jaic-<platform>.tar.gz` / `.zip` | `jaic-<platform>/` |
| 0.4.1 and later | `jai-<platform>.tar.gz` / `.zip` | `jai-<platform>/` |

Published assets are never renamed. The rule (`jaic` up to 0.4.0, else `jai`) lives in four places, kept in step by hand: `archive_prefix` in `install.sh`, `$prefix` in `install.ps1`, `archivePrefix` in the extension's `src/toolchain.ts`, and `expected_prefix` in `tools/render_packages.py`. `release.yml` always packages `jai-<platform>`, even on a dry run while `Cargo.toml` still says 0.4.0, so the renderer takes whichever prefix `SHA256SUMS` (or the archive folder) has for every platform, trying the version's own first. The installed layout does not depend on the name: both install scripts move the archive's top folder to `<install dir>/<version>`, so upgrading a 0.4.0 install to a `jai-` release (or back) is an ordinary upgrade.

### Rendering

`tools/render_packages.py` fills the templates in `packaging/` (`@VERSION@`, `@BASE_URL@`, `@ARCHIVE_PREFIX@` (`jai` or `jaic`), `@SHA256_<PLATFORM>@`, `@RELEASE_DATE@`):

```sh
python3 tools/render_packages.py --version v0.4.0 --sums SHA256SUMS --out out
# out/homebrew/jai.rb
# out/winget/manifests/m/matteopolak/jai/0.4.0/matteopolak.jai{,.installer,.locale.en-US}.yaml
```

`--sums` reads a release's `SHA256SUMS`; `--archives DIR` hashes the archives in a directory instead (dry runs, before a release exists). It fails if an archive is missing from the checksums or a placeholder is unknown. The Homebrew formula takes lower-case hex, winget upper-case. Tests: `tools/test_render_packages.py`.

### Release jobs

After `publish` creates the GitHub release, two jobs in `release.yml` run. They depend on `publish`, so neither can hold up the release or the extension stores; a failure only marks its own job red.

- `homebrew` (ubuntu): downloads the release's `SHA256SUMS`, renders `jai.rb`, puts it in a throwaway local tap, runs `brew style`, `brew audit --strict --online`, `brew install` and `brew test` (Linux x86-64), then clones `matteopolak/homebrew-tap` with `HOMEBREW_TAP_TOKEN`, writes `Formula/jai.rb` (creating `Formula/` on the first run) and pushes a `jai <version>` commit to its default branch. An unchanged formula is not committed.
- `winget` (windows-2025): renders the three manifests, runs `winget validate` through `packaging/winget/validate.ps1`, downloads the pinned `komac` and checks its SHA-256, then, with `WINGET_TOKEN` as `GITHUB_TOKEN`: `komac sync` fast-forwards the token owner's fork of `microsoft/winget-pkgs` (a failure only warns), and `komac submit --yes` commits the manifests to a branch of that fork and opens the pull request against `microsoft/winget-pkgs`.

Without its secret a job prints that it skipped and succeeds. A run without a tag (Actions, release, Run workflow, empty tag) renders from that run's archives and the `Cargo.toml` version, validates (`brew style` and offline `brew audit`; `winget validate`, and `komac submit --dry-run` when `WINGET_TOKEN` is set) and publishes nothing.

### Homebrew formula

`packaging/homebrew/jai.rb.tmpl` downloads `jai-macos-arm64.tar.gz` or `jai-linux-x64.tar.gz` (`jaic-*` when rendered for 0.4.0 or earlier), installs the four binaries, `stdlib/` and `prelude/` into `libexec`, and links the binaries into `bin`. `jaic` finds `libexec/stdlib` by resolving the `bin` symlinks to the real file (`jaic::stdlib_dir`, see [releases](releases.md#how-it-works)). Intel macOS and arm64 Linux get a URL anyway so the formula loads everywhere (`brew info`, tap JSON), and `depends_on arch:` refuses the install with Homebrew's own message. The `test do` block runs a hello world with `jaic run`, formats a line with `jaifmt --stdin` and lints with `jailint`. The license is `AGPL-3.0-or-later`; the runtime library exception has no SPDX id, so `brew audit` would reject it in the `license` line and it is noted in a comment instead.

### winget manifests

`packaging/winget/` holds the version, installer and default-locale manifests at schema 1.12.0, the version komac 2.16.0 writes. `komac submit` re-serializes the manifests before committing them (its own `# Created with komac` header, sorted lists, and the installer manifest in its own schema version), so the templates must use the same version as the pinned komac: with 1.10.0 templates its pull request would mix a 1.12.0 installer manifest with 1.10.0 locale and version manifests. The `winget` on GitHub's Windows runners (1.11) does not know 1.12.0 and warns "The schema header URL does not match the expected pattern" (exit code `-1978335192`, valid with warnings); `packaging/winget/validate.ps1` accepts that one warning and fails on any other warning or error.

The installer is `InstallerType: zip` with `NestedInstallerType: portable` and one `NestedInstallerFiles` entry per tool with a `PortableCommandAlias`. winget extracts the whole zip into `%LOCALAPPDATA%\Microsoft\WinGet\Packages\matteopolak.jai_<source>\`, so `jai-windows-x64\stdlib` stays next to `jaic.exe` (each `RelativeFilePath` names that folder, `@ARCHIVE_PREFIX@-windows-x64\jaic.exe` in the template), and creates symbolic links named `jaic.exe` and so on in `%LOCALAPPDATA%\Microsoft\WinGet\Links` (on `PATH`). Through such a link, `std::env::current_exe` gives the link's path, which has no `stdlib` beside it; `jaic::stdlib_dir` then canonicalizes it to the real file in the package folder and finds the standard library there. The `packaging` workflow checks this: it installs from the rendered manifests with `winget install --manifest` and runs `jaic run`, `jailint` and `jaifmt` through the links.

### install.sh

POSIX `sh` (checked with `shellcheck -s sh`). It maps `uname` to an archive (`macos-arm64`, `linux-x64`; on macOS it also detects Apple silicon under Rosetta), finds the latest version from the `releases/latest` redirect (no API rate limit), downloads the archive and `SHA256SUMS` with `curl` or `wget`, checks the hash with `sha256sum` or `shasum`, and unpacks into `~/.local/share/jai/<version>`. It then points `~/.local/bin/{jaic,jailsp,jailint,jaifmt}` at it (only the tools the archive has: 0.2.0 shipped just `jaic` and `jailsp`), deletes the versions it installed earlier, and prints how to add the bin folder to `PATH` when it is missing. It refuses to replace a bin entry that is not a symlink, so it never overwrites another install. The whole script is one function called on the last line, so a download cut short by `curl | sh` runs nothing.

```sh
JAI_VERSION=0.3.0 sh install.sh                      # a specific release (v prefix optional)
JAI_INSTALL_DIR=/opt/jai JAI_BIN_DIR=/usr/local/bin sh install.sh
```

`JAI_VERSION`, `JAI_INSTALL_DIR` and `JAI_BIN_DIR` are also read as `JAIC_VERSION`, `JAIC_INSTALL_DIR` and `JAIC_BIN_DIR` (the names before 0.4.1, still used by `packaging.yml`); the `JAI_` name wins when both are set. A version up to 0.4.0 downloads the `jaic-<platform>` archive, a later one `jai-<platform>`.

To try a change against a release that does not exist yet, repackage an archive under the new name with its own `SHA256SUMS` in a folder `<dir>/v<version>/`, and run a copy of the script with the `base=` URL replaced by `file://<dir>/v$version` and curl's `--proto '=https' --tlsv1.2` removed (curl reads `file://` URLs); the script itself has no URL override.

Uninstall: `rm -rf ~/.local/share/jai ~/.local/bin/{jaic,jailsp,jailint,jaifmt}`.

### install.ps1

Runs in Windows PowerShell 5.1 and PowerShell 7 (`irm ... | iex`, so it never calls `exit`). It picks `windows-x64` or `windows-arm64` from the OS architecture (not the process's, which may be emulated), gets the latest tag from the GitHub API, verifies the zip against `SHA256SUMS`, unpacks it into `%LOCALAPPDATA%\Programs\jai\<version>`, points the directory junction `%LOCALAPPDATA%\Programs\jai\current` at it, adds `current` to the user `PATH` once, and deletes older versions. A junction needs no administrator rights or developer mode, unlike a symbolic link, and `jaic.exe` started through it finds `current\stdlib`. `JAI_VERSION` and `JAI_INSTALL_DIR` (or `JAIC_VERSION` and `JAIC_INSTALL_DIR`) work as in `install.sh`, as does the archive name.

### Validation in CI

`.github/workflows/packaging.yml` runs when these files change (and by hand), always against the latest published release:

- `shellcheck`: `install.sh` as POSIX sh.
- `install.sh` on ubuntu-24.04 and macos-15: installs the previous release, upgrades to the latest through `sh < install.sh` (checking that the old version is removed and the `PATH` advice is printed), runs `jaic run`, `jailint` and `jaifmt` from another directory, re-runs with custom directories, and checks the failures for a missing version and an occupied bin entry.
- `install-ps1` on windows-2025: installs twice under Windows PowerShell 5.1, then runs the tools from `PATH` under PowerShell 7.
- `homebrew` on macos-15 and ubuntu-24.04: renders the formula, then `brew style`, `brew audit --strict --online`, `brew install --build-from-source`, `brew test` and `brew uninstall` from a local tap.
- `winget` on windows-2025: renders the manifests, `winget validate`, `winget install --manifest`, runs the tools through the WinGet links, and `winget uninstall --manifest`.

Nothing in `packaging.yml` publishes.

## How to change it

- Formula or manifests: edit the templates in `packaging/`, then render against the latest release and check locally (`brew tap-new --no-git you/local`, copy `out/homebrew/jai.rb` into its `Formula/`, `brew style`, `brew audit --formula --strict --online you/local/jai`, `brew install`, `brew test`, `brew uninstall`, `brew untap`). A new placeholder needs a value in `substitutions()` in `tools/render_packages.py`; an unknown one is an error.
- A new platform archive: add it to `PLATFORM_ASSETS` in `tools/render_packages.py` and to the formula (`on_macos`/`on_linux` with `on_arm`/`on_intel`) or the installer list, and to the `platform` functions of both install scripts.
- A newer winget schema: change the three `ManifestVersion` lines and `$schema` headers together, to the version the pinned komac writes (`komac submit --dry-run` prints it), and check `packaging.yml`'s `winget` job.
- komac: bump `KOMAC_VERSION` and `KOMAC_SHA256` in `release.yml` (the `x86_64-pc-windows-msvc.exe` asset's SHA-256, from the release's `SHA256SUMS` or the asset digest), after the release is 14 days old ([dependency policy](dependency-policy.md)).
- Re-running the `winget` job for a version that already has a pull request opens another one; close the duplicate.

Gotchas:

- A release's assets must stay as they are: re-uploading an archive changes its hash and breaks that version's formula and manifests.
- The formula is not bottled: `brew install` unpacks the release archive, so installs need no build tools.

## Configuration

One-time setup. `matteopolak/homebrew-tap` (public, with a README) and the fork `matteopolak/winget-pkgs` already exist; only the two repository secrets remain (Settings, Secrets and variables, Actions):

| Secret | What | Scope |
| --- | --- | --- |
| `HOMEBREW_TAP_TOKEN` | fine-grained personal access token | repository `matteopolak/homebrew-tap` only, Contents: read and write |
| `WINGET_TOKEN` | classic personal access token (komac cannot open the cross-repository pull request with a fine-grained one) | `public_repo`; it submits from its owner's fork of `microsoft/winget-pkgs` |

The first winget pull request:

- needs the Microsoft Contributor License Agreement signed once: the CLA bot comments on the pull request with instructions (`@microsoft-github-policy-service agree`).
- is validated by Microsoft's pipeline and then reviewed by a winget-pkgs moderator, as every new package is; later versions of an existing package are usually merged automatically once validation passes. `winget install matteopolak.jai` works only after the first one is merged.

Other knobs: `JAI_VERSION`, `JAI_INSTALL_DIR`, `JAI_BIN_DIR` (install scripts; also as `JAIC_*`), `KOMAC_VERSION`/`KOMAC_SHA256` (`release.yml`), the identifiers `REPOSITORY` and `WINGET_ID` in `tools/render_packages.py`.

## Dependencies

The GitHub release assets (`jai-*.tar.gz`/`.zip`, `jaic-*` up to 0.4.0, and `SHA256SUMS`), Homebrew on the runners, `winget` on the `windows-2025` runner, [komac](https://github.com/russellbanks/Komac) (pinned, checksum verified), Python 3 for the renderer, and `curl`/`wget`, `tar` and `sha256sum`/`shasum` on users' machines for `install.sh`.
