# Native integration CI

## What it is

The temporary `codex/cleanroom-integration` branch exercises the combined implementation on native hosted runners. It publishes no release artifacts to GitHub Releases or external stores.

## How it works

Normal CI covers macOS and Linux x64/arm64. The Windows native workflow includes this branch and executes the File contracts in the interpreter and compiled O0/O2 programs on Windows x64 and arm64, alongside signed-wheel propagation and existing native checks. Known failures remain visible; a successful cross-check is not native runtime evidence.

## How to change it

Add behavior cases under `tests/platform-io` and keep the native runner step in both Windows jobs. Preserve the normal test gates. Review workflow triggers before pushing; release builds remain on hold.

## Configuration

`tools/check_platform_io.py --jaic <compiler> --stdlib stdlib --native` runs against production sources. The CI branch selector is separate from release tags and workflow dispatch.

The normal WebAssembly target job also runs the unchanged File contract with
`--wasm`, exercising the sandbox and actual WASI O0/O2. It does not use missing
import traps or mutate Extensions. Coordinator-filtered candidate 159ca65 results
confirm real Windows x64/arm64 File and signed-wheel checks; the overall workflow
failed on other runtime/platform/module gaps. The newer WASI fix still needs the
next coordinator-reviewed CI run. Release builds/publication remain held.

## Dependencies

The integrated compiler and stdlib, pinned Rust/LLVM toolchain, Visual Studio SDK, and GitHub native runners.

The editor archive-contract test locates the build matrix before searching for its following runner. Earlier metadata jobs can have their own `runs-on`, so searching from the start of the workflow produces an empty matrix and a false archive mismatch. Full editor unit checks still run in CI; local standalone matrix extraction verifies all six entries.
