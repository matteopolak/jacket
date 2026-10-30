# Windows probe

## What it is

`.github/workflows/windows-probe.yml` compiles and runs one Jai program on a real Windows runner (x64 or arm64) and prints the compile output, the program output and both exit codes into the job summary. It reproduces a Windows report without a Windows machine.

## How it works

Actions, "windows probe", Run workflow. Inputs:

| Input | Meaning |
| --- | --- |
| `source` | `release`: install a published `jacket-windows-<cpu>` archive (about a minute). `branch`: build `jaic` from the ref the workflow was dispatched on (a few minutes; reuses the `setup-windows` action and the native-libs cache of `ci.yml`'s Windows jobs) |
| `tag` | the release for `source=release` (empty: the latest) |
| `arch` | `x64` (`windows-2025`) or `arm64` (`windows-11-arm`) |
| `snippet` | the Jai source, inline |
| `path` | a `.jai` file in the checked-out ref; wins over `snippet` |
| `args` | extra `jaic` arguments such as `-O2` |

The program is built with `jaic probe.jai -exe probe.exe` and run. `jaic` finds the MSVC linker and the Windows SDK itself, as in the other Windows jobs, so no developer prompt is entered. The step fails when the compile or the run exits non-zero.

A workflow can be dispatched only once it is on the default branch. To probe a change that is not on `main` (the workflow itself, or a compiler fix), push a throwaway `tmp/win-probe-*` branch instead: a push to such a branch builds the branch and runs `probe.jai` from the repository root on both architectures. Delete the branch afterwards.

```sh
git push origin HEAD:refs/heads/tmp/win-probe-foo
git push origin --delete tmp/win-probe-foo
```

## How to change it

Add inputs under `on.workflow_dispatch.inputs` and read them through `env:` (never interpolate them into a script). Setup stays in the shared composite actions so the probe tracks what CI builds.

## Configuration

The LLVM version comes from `.github/pins.json` through the `setup-windows` action.

## Dependencies

`.github/actions/setup-windows`, `.github/actions/native-libs-cache`, `tools/ci/release_smoke.py fetch`, the `gh` CLI on the runner.
