# GitHub Action (setup-jai)

## What it is

`setup-jai` is a composite GitHub Action, kept in its own repository (`matteopolak/setup-jai`), that installs the released toolchain on a CI runner so other projects can run `jaic`, `jaifmt` and `jailint` in their workflows.

## How it works

Four steps, each a script under `scripts/` in the action repository:

1. `resolve.sh` picks the platform from `RUNNER_OS`/`RUNNER_ARCH` (`linux-x64`, `linux-arm64`, `macos-arm64`, `macos-x64`, `windows-x64`, `windows-arm64`), turns `latest` into a version (GitHub API with the token, then the `releases/latest` redirect), and chooses the [archive name](package-managers.md#archive-names) (`jaic-` up to 0.4.0, else `jai-`).
2. `actions/cache` restores the unpacked folder, keyed by version and platform.
3. `install.sh`, on a cache miss, downloads `SHA256SUMS` and the archive from the release, checks the hash, and unpacks it. A release whose `SHA256SUMS` lists no archive for the platform fails with a message.
4. `expose.sh` adds the tools to `PATH` and sets `JAIC_STDLIB` to the unpacked `stdlib/`, so `jaic::stdlib_dir` and the native-library lookup (`<stdlib>/../artifacts/native-libs`) work even through links.

Published as `matteopolak/setup-jai`; use `matteopolak/setup-jai@v1`. Usage:

```yaml
- uses: matteopolak/setup-jai@v1
  with:
    version: 0.4.3
- run: jaifmt --check . && jailint -D warnings . && jaic build main.jai
```

## How to change it

The archive naming and layout must match [releases](releases.md) and [package managers](package-managers.md): if the archive name rule, the platform list or the top-folder layout changes, update `resolve.sh` too (it is a fifth copy of the `jaic`/`jai` prefix rule). Its own `test.yml` runs the action on every supported runner, then compiles, formats and lints a hello world.

## Configuration

Inputs `version`, `tools`, `cache`, `token`; outputs `version`, `path`. See the action's README.

## Dependencies

The release assets (`jai-<platform>` archives and `SHA256SUMS`), `actions/cache`, `curl`, and `7z` or PowerShell on Windows.
