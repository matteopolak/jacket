# Windows probe

`.github/workflows/windows-probe.yml` compiles and runs one Jai program on a real Windows runner (x64 or arm64) and prints the compile output, the program output and both exit codes into the job summary. It reproduces a Windows report without a Windows machine.

## Usage

Actions, "windows probe", Run workflow:

| Input | Meaning |
| --- | --- |
| `source` | `release`: install a published `jacket-windows-<cpu>` archive (about a minute). `branch`: build `jaic` from the dispatched ref (a few minutes) |
| `tag` | the release for `source=release` (empty: latest) |
| `arch` | `x64` (`windows-2025`) or `arm64` (`windows-11-arm`) |
| `snippet` | the Jai source, inline |
| `path` | a `.jai` file in the ref; wins over `snippet` |
| `args` | extra `jaic` arguments such as `-O2` |
| `bench` | also run the benchmark harness smoke test (reports into the summary, never fails the job) |

The program is built with `jaic probe.jai -exe probe.exe` and run. The step fails when the compile or run exits non-zero.

A workflow can be dispatched only once it is on the default branch. To probe a change that isn't on `main`, push a throwaway branch: a push to `tmp/win-probe-*` builds the branch and runs `probe.jai` from the repository root on both architectures. Delete the branch afterwards.

```sh
git push origin HEAD:refs/heads/tmp/win-probe-foo
git push origin --delete tmp/win-probe-foo
```

## How to change it

Add inputs under `on.workflow_dispatch.inputs` and read them through `env:` (never interpolate them into a script). Keep setup in the shared composite actions (`setup-windows`, `native-libs-cache`) so the probe tracks what CI builds. The LLVM version comes from `.github/pins.json`.
