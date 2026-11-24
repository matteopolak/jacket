# CI wall time

What decides how long a push waits for green in `ci.yml`, what was changed to cut it, and the measurements. Wall time is push to the last job finishing, queueing included.

## What sets it

- **Queueing.** GitHub runs 40 jobs at once, 5 of them on macOS. A run with 65 jobs always queues some. The macOS cap mattered most: three macOS `stdlib-runtime` jobs waited 9 to 12 minutes behind the macOS `test` and sanitizer jobs, and the longest of them (`suites`, 27 minutes) then ended the run.
- **Repeated setup.** Every `stdlib-runtime` leg built the same compiler (2 to 4 minutes each) and rebuilt the native helper libraries on a cache miss. The macOS catalogue shards spent 8 minutes setting up to run one minute of tests.
- **The longest leg.** After the queue is gone the run is as long as the slowest Windows shard or Linux tooling part (20 to 25 minutes).

## What changed

| Change | Effect |
| --- | --- |
| One macOS `stdlib-runtime` job on pushes, running only the macOS modules' `jacket test` (`tools/ci/platform_modules.py`) | Two macOS jobs fewer; `jacket test` on the Mac went from 12.9 to 4.9 minutes. The full tier is unchanged. |
| `stdlib-build`: the compiler built once per platform, downloaded by the legs | Compiler build time summed over the run: 35 to 5 minutes; the legs download 35 MB in seconds. |
| `stdlib-build` restores the Rust cache of `stdlib-runtime` (`rust-cache-key`) | The build job starts warm instead of compiling every dependency. |
| The helper-library cache is its own entry per OS, CPU and LLVM version, and restores the newest older one when its key changes | A change under `stdlib/internal/**` or `tools/` rebuilds the helpers it affects (each has an input stamp) instead of all of them. |

Not kept: `jacket test --partition time:K/N`, which dealt tests (then whole test roots) by recorded seconds from a table built from JUnit files. The spread between the shards did not shrink (Windows 8 to 18 minutes against 8 to 16 with `hash:`; Linux tooling 9 to 13 against 8 to 11), because a shard's time is mostly building its programs, which a per-test table does not see. Balancing needs per-root build times, which no run records yet; `jacket test --junit` plus the artifact upload used here is a way to collect them.

## Measurements

Both are full runs of the fast tier. Before: push of `chore(release): 0.39.0` to `main` (run 45374107592). After: the same fast tier with the changes above, run by hand on a branch (run 45484405817), so its caches were restore-only and the new helper-cache entry did not exist yet.

| | Before | After |
| --- | --- | --- |
| Wall time | 39.7 min | 29.1 min |
| Critical path | `stdlib runtime tests (macos-arm64, suites)`: queued 12.1, ran 27.3 | `compiler (ubuntu-24.04, targets 2/3)`: queued 2.7, ran 26.1; Windows `stdlib` shard 3/5 ends 1.3 min earlier |
| Jobs that queued over a minute | 24, 125 queued minutes in all, longest 12.1 | 12, 33 queued minutes, longest 5.7 |
| macOS jobs, their queue | 6 jobs, 12.1 / 11.7 / 9.3 / 0 / 0.1 / 0 min | 5 jobs, 3.0 / 4.2 / 5.7 / 0.1 / 0.5 min |
| Runner minutes | 556 (macOS 82) | 495 (macOS 41) |
| Native dependency builds, summed over jobs | 52 min | 15 min |
| Compiler builds in `stdlib-runtime`, summed | 35 min | 5 min (+ 1 min of downloads) |

The after run was not green: the `jai` format and generated-file checks (the stale file is `docs/reference/tools.md`), the docs-site and tools-scripts lint, a Windows `Bindings_Generator::target-triples` test and a Linux tooling check failed. None of them runs a file these changes touch, and the branch is based on a `main` commit whose own run also failed; they were not investigated further. The `targets` jobs took 26 minutes against 15 before; they are untouched by these changes and were slower for a reason this run does not show, so the critical path of a quiet run may be shorter than 29 minutes.

## How to change it

- Keep the macOS legs few and long: a job costs a cap slot for its setup as well as its tests. Add to `stdlib_matrix(False)` in `tools/ci/ci_changes.py` with that in mind (`test_ci_changes.py` checks the count).
- A program a `stdlib-runtime` leg needs from the compiler build goes in the `upload-artifact` `path` list of `stdlib-build`, not in the leg.
- What `platform_only` runs is whatever `platform_modules.py` finds; extend `PATTERNS` there for another OS's APIs.
- To measure a change: `gh workflow run ci.yml --ref BRANCH -f tier=fast`, then take each job's `created_at`, `started_at` and `completed_at` from `gh api repos/OWNER/REPO/actions/runs/ID/jobs`. Queue time is `started_at - created_at`.

## Configuration

`JACKET_TEST_PLATFORM_ONLY` (set from the leg's `platform_only`) limits `tools/ci/stdlib_jacket_test.sh` to the host's modules. `stdlib_platforms` (output of `ci_changes.py`) lists the platforms `stdlib-build` builds; it is `[]` when `stdlib-runtime` is not selected.

## Dependencies

`actions/upload-artifact` and `actions/download-artifact` (pinned), `Swatinem/rust-cache`, `actions/cache`. See [CI job selection](ci-job-selection.md) for how jobs are chosen and [development](development.md) for the caches.
