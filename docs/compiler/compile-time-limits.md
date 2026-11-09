# Compile-time limits and costs

How the language server and `jailint` bound the compile-time code of the program they check, and how any tool finds the compile-time code that is slow. Each top-level evaluation has its own block budget, compile-time execution shares a wall-clock deadline, and each evaluation's blocks and time are recorded.

```sh
jaic main.jai -profile_runs     # which #run costs the most
jaic main.jai -run_budget 0     # no limit
```

## How it works

- **Budget per evaluation.** `Interp::block_budget` counts steps: loop back-edges, calls and `switch` jumps. When it runs out the trap is `BudgetExhausted`, reported at the directive that started the evaluation with notes for the innermost frames. The next evaluation starts fresh.
- **Wall-clock limit.** `SandboxHost::stop_after` starts a clock per compile. The interpreter checks it every 1024 blocks; once it passes, every run traps with `Stopped` and shows as an unknown-result diagnostic. The type checker asks too (`Compiler::check_not_stopped`) before each body, entity, file, scope, macro expansion and polymorphic instantiation, so time spent outside the interpreter is bounded the same way; the work in progress fails with the same unknown-result "checking stopped".
- `ide::lint_compile::LintCompile` sets both for the language server and `jailint` from one `CheckLimits`. `jaic` has a budget of its own (default ten times the editor's).
- **Cost records.** Each `call_thunk` pushes a `perf::RunStat` (span, blocks, time) to `Compiler::meta::perf::runs`, also used by `PERFORMANCE_REPORT`.
- **`-profile_runs`** gives the interpreter an unlimited budget and prints the top-level evaluations of the program's own files by blocks, share, time and location, then one line for the standard library.
- **Language server.** `Analysis::evaluation_costs` sums records by span. A code lens above each evaluation of at least `costLensBlocks` shows e.g. `1.2M blocks · 3 ms`; `jai.showCompileTimeCosts` lists all, costliest first. With `compileTime.limitNotices` the server sends `jai/compileTimeLimit` when a fresh compile hit a limit, and the VS Code extension shows a popup.

## How to change it

### Build metaprograms in the editor

The checker executes build metaprograms such as `#run,stallable build()` and follows their child workspaces. These runs can supply imports, generated sources, constants and type-check message handlers, so skipping a run because it creates a workspace would lose program facts. `LintCompile` supplies no output backend: child workspaces are checked without emitting or linking executables. Their compile-time code uses the same sandbox host and check deadline as the parent.

The wall-clock deadline is shared across the workspace tree and enforced when interpreted code polls it and between units of checking work. It cannot interrupt a single unit already running: a foreign call from `#run`, or one very large expression, runs to its end, so the deadline is not a hard bound on request latency. A notice at the outer build directive does not mean the linker took that time. To investigate a slow check, reproduce it with `jailint --context first.jai first.jai --check-time-limit 20` and compare the checker path with the native-host build; `jaic -profile_runs` measures the build's own compile-time executions.

The sandbox answers POSIX `sysconf` queries using the checked program's ABI: macOS and Linux use different numbers for page size and processor counts. The sandbox reports one processor and the host page size used by its private anonymous mappings. On Unix hosts, private anonymous `mmap`/`mprotect`/`munmap` calls operate only on ranges owned by that sandbox; file-backed mappings, fixed mappings and executable protections are refused, and one host maps at most 1 GiB in total (`MAX_MAPPED_BYTES`); further `mmap` calls fail with `ENOMEM` until something is unmapped. Keep query numbers consistent with `stdlib/POSIX/declarations.jai`; a failed allocator page-size query can otherwise recurse through a program's allocating assertion handler until the check limit stops it.

### Implementation

- An evaluation that goes through `call_thunk` needs nothing. Code that runs the interpreter another way must call `Interp::begin_evaluation`.
- A compiler running part of an evaluation (like `host_run.rs`) should `lend_budget` and `take_back_budget`.
- Private anonymous mapping support lives in `interp/sandbox/virtual_memory.rs`; keep ownership checks and host-page alignment when adding operations or platforms.
- Defaults are on `CheckLimits`; VS Code defaults are in `package.json` and `limitPopups.ts`.
- Don't add project-file settings: the limits protect whoever opens a project, so a project cannot set them.

## Configuration

| Where | Setting | Default |
| --- | --- | --- |
| VS Code | `jai.compileTime.evaluationBudget` | 100000000 steps; `0` for none |
| VS Code | `jai.compileTime.checkTimeLimit` | 10 seconds; `0` for none |
| VS Code | `jai.compileTime.costLensBlocks` | 1000000 |
| Neovim | `compile_time.evaluation_budget`, `check_time_limit`, `cost_lens_blocks` | same |
| `jailint` | `--compile-time-budget <steps>`, `--check-time-limit <seconds>` | same; the time limit applies per program |
| environment | `JACKET_CHECK_TIME_LIMIT` | 10; `0` for none (CI sets `0`) |
| `jaic` | `-run_budget <steps>` | 1000000000; `0` for none |
| `jaic` | `-profile_runs` | off |

VS Code reads these from user settings only, not a workspace's `.vscode/settings.json`. `jailsp` reads them from `initializationOptions` and the `jai` section of `didChangeConfiguration`.

## Dependencies

`jaic::interp`, `jaic::sema::perf`, the language-server session and `jailint`'s driver.
