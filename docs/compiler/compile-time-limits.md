# Compile-time limits and costs

## What it is

How the language server and `jailint` bound the compile-time code of the program they check, and how any tool finds the compile-time code that is slow. Every top-level compile-time evaluation has a block budget of its own, the whole check has a wall-clock limit, and each evaluation's blocks and time are recorded. `jaic` builds have no limits.

## How it works

**Budget per evaluation.** `Interp::block_budget` counts the basic blocks left (the interpreter's only per-block bookkeeping, in `enter_block`). `Interp::eval_blocks`, set from `Options::eval_block_limit`, is what each top-level evaluation starts with: `Compiler::call_thunk` (the one place `#run`, constants, `#if` conditions, `#modify`, computed `#insert` and build metaprograms start running) calls `Interp::begin_evaluation`, which refills the budget. A run inside another run, or inside a workspace the running metaprogram drives, draws on the budget of the run around it (`Interp::nested_eval`, set by `build::step` for a workspace stepped from a message loop and by `Interp::lend_budget` for the isolated `#run,host` and cross-target twin compilers). Without `eval_block_limit` the budget is shared by the whole compile, which is what the playground wants: its page sets one budget and refills it after each wait (`Host::refill_budget`).

When a budget runs out the trap is `TrapKind::BudgetExhausted` and `Compiler::budget_diagnostic` reports it at the directive that started the evaluation, with notes for the innermost three frames (`still running `spin` at main.jai:3`, `called from `outer` at main.jai:4`). Nothing else is affected: the next evaluation starts with a full budget, so a runaway `#run` cannot make `type_info(T)` or later runs fail. `Interp::budget_traps` counts exhaustions; `scope/expand.rs` uses it to tell an item that was cut off (not retried) from one that waits for another declaration.

**Wall-clock limit.** `SandboxHost::stop_after` starts a clock for the compile that uses the host. The interpreter asks `Host::stop_requested` every 1024 blocks and when a run starts; once the limit has passed every run on the host traps with `TrapKind::Stopped`, reported as an unknown-result diagnostic (`DiagnosticKind::Unknown`) that editors hide. The compile then goes through the rest of the program without running compile-time code. The language server adds one warning on the first line of the root file, `checking stopped after 10 s; results are partial`, marks the analysis incomplete and does not record the metaprogram's facts; `jailint` reports the root as incomplete with the same message. The clock reads zero in the browser, where the limit does nothing.

`jaic::ide::lint_compile::LintCompile` sets both for the language server and `jailint`, from one `CheckLimits`.

**Cost records.** Each `call_thunk` pushes a `perf::RunStat` (span, blocks, wall time, whether it was outermost) to `Compiler::meta::perf::runs`, the list the `PERFORMANCE_REPORT` message already uses. Blocks are the difference of the budget before and after. The compilers that run part of an evaluation for this one (the isolated `#run,host` compiler in `host_run.rs`, the host twin in `host_twin.rs`) hand their records over when the run ends, so the list and the lenses show those evaluations at the directive that started them; the compilers a metaprogram builds are listed through `Analysis::evaluation_costs`. Blocks exist wherever the interpreter has a budget: always in the editor and `jailint`, under `-profile_runs` in `jaic`, nowhere in a plain build.

- **`jaic -profile_runs`** (`jacket build -profile_runs` passes it through) gives the interpreter an unlimited budget so it counts, and `interp::profile::run_report` prints the top-level evaluations of the program's own files by blocks, with their share, wall time and `path:line:column`, then one line summing the runs the standard library started (`standard library (22 runs)`), so the program's costs come first. It composes with `-profile`, which then lists the hottest procedures too.
- **Language server.** `Analysis::evaluation_costs` sums the records by span. A code lens above each evaluation of at least `costLensBlocks` shows `1.2M blocks · 3 ms`; its command `jai.showCompileTimeCosts` (also run by the popup's button and Neovim's `:Jai costs`) answers with every evaluation of the program, costliest first, as `{uri, range, blocks, millis, title}`.

## How to change it

- A new kind of evaluation needs nothing if it goes through `call_thunk`. Code that runs the interpreter another way must call `Interp::begin_evaluation` itself, or it draws on whatever the last evaluation left.
- Another compiler that runs part of an evaluation (like `host_run.rs`) should `lend_budget` and `take_back_budget`, with `nested` true when the borrower runs inside an evaluation.
- Defaults are the constants on `jaic::ide::lint_compile::CheckLimits`; the VS Code defaults are in `package.json` and `limitPopups.ts`. Do not add a project-file setting for them.
- The lens threshold applies in `compile_costs.rs`; the cost text is `cost_text`.

## Configuration

The limits protect whoever opens a project, so a project cannot set them: there is nothing in `jacket.toml` for them. They come from the person running the tool.

| Where | Setting | Default | Meaning |
| --- | --- | --- | --- |
| VS Code | `jai.compileTime.evaluationBudget` | 200000000 | basic blocks one top-level evaluation may run; `0` for no limit |
| VS Code | `jai.compileTime.checkTimeLimit` | 10 | seconds a whole check may take; `0` for no limit |
| VS Code | `jai.compileTime.costLensBlocks` | 1000000 | evaluations of at least this many blocks get a cost lens |
| Neovim | `compile_time.evaluation_budget` | 200000000 | as `jai.compileTime.evaluationBudget` |
| Neovim | `compile_time.check_time_limit` | 10 | as `jai.compileTime.checkTimeLimit` |
| Neovim | `compile_time.cost_lens_blocks` | 1000000 | as `jai.compileTime.costLensBlocks` |
| `jailint` | `--compile-time-budget <blocks>` | 200000000 | as the budget above |
| `jailint` | `--check-time-limit <seconds>` | 10 | as the time limit above, for each program checked separately (every program gets the whole limit; the run as a whole has none) |
| `jaic` | `-profile_runs` | off | prints the cost table (the program's runs, then the standard library as one line); a `jaic` build has no limits |

The VS Code settings are `"scope": "machine"`: VS Code reads them from the user's settings only (or the remote machine's), and ignores a workspace's `.vscode/settings.json` for them.

`jailsp` reads `compileTime.evaluationBudget`, `compileTime.checkTimeLimit`, `compileTime.costLensBlocks` and `compileTime.limitNotices` from `initializationOptions` and the `jai` section of `workspace/didChangeConfiguration` (`protocol/notifications.rs`, `Session::set_evaluation_budget` and friends); one the client leaves out keeps its default. A change drops the compiles kept and checks again.

**The popup.** With `compileTime.limitNotices` the server sends `jai/compileTimeLimit` (`{kind: "budget" | "time", uri, range, message}`) for each fresh compile that hit a limit (`Cache::take_limit_hits`, drained with the other messages for the user). The VS Code extension (`limitPopups.ts`, the part without `vscode` imports; `limitUi.ts` is the glue) shows a warning naming the evaluation's place with three buttons: *Raise limit* doubles the setting in the user's settings, which makes the server check again; *Show compile-time costs* runs `jai.showCompileTimeCosts` at the evaluation; *Open settings* opens the setting. The same evaluation is not shown again until the user edits (a time limit, which is about the whole check, until any edit). Clients that do not ask get no notification. Neovim (`lua/jai/limits.lua`) shows a `vim.notify` warning with the place and the `compile_time.*` setting to raise, once per place until a buffer is written.

## Dependencies

`jaic::interp` (budget, host, traps), `jaic::sema::perf` (records), the `jai-language-server` session (config lookup, lens, command) and `jailint`'s driver.
