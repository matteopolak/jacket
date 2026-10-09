# Third-party smoke test

## What it is

A repeatable pass over public Jai repositories on GitHub: find active, popular projects, pin them in the
[upstream corpus](upstream-corpus.md), try `jaic check`/`build`/`run` on their entry points, and fix what is a
jaic or stdlib bug. Results live in the [project status](upstream-corpus.md#project-status) table.

## How it works

1. **Find repositories.** GitHub's search tags Jai as `language:Jai`. Two queries, because "updated" also
   counts stars and issues while "pushed" means code changed:

   ```sh
   gh search repos --language=Jai --stars=">=20" --updated=">=2026-05-05" --limit 100 \
       --json fullName,stargazersCount,pushedAt
   gh api -X GET search/repositories -f q='language:Jai stars:>=20 pushed:>=2026-05-05' \
       -f per_page=100 --jq '.items[] | [.full_name, .stargazers_count, .pushed_at] | @tsv'
   ```

   Drop repositories already in `REPOSITORIES`/`LIBRARIES` of `tools/fetch_upstreams.py`, and ones that are
   not Jai (jaithon is its own language in `.jai` files).
2. **Pin.** Add each repository to `REPOSITORIES` in `tools/fetch_upstreams.py`. Run `python3
   tools/fetch_upstreams.py` and commit `corpus/upstreams.json`. Only `.jai` files and a few others are
   downloaded. Resources a project reads at compile or run time go in `RESOURCE_PREFIXES`, and files over the
   default size cap go in `MAX_FILE_BYTES` (chess-jai's network).
3. **Find the entry point.** Usually `first.jai` or `build.jai` (a metaprogram) at the root. Its arguments after
   `-` select targets (`build.jai - ui ai release`). The README or a `build.sh`/`.bat` shows the real command.
4. **Try it**, cheapest first: `jaic check`, then `jaic build`, then run the result. A `--release` jaic keeps big
   metaprograms quick.
5. **Classify** the outcome:
   - passes;
   - jaic bug or missing stdlib API: reduce it to a test under `tests/stdlib/` and fix it if it is generic and of
     reasonable size, otherwise write the minimal repro into the project notes;
   - needs native libraries or another platform: note what (`-os linux`/`-os windows` still type-checks);
   - outdated against current Jai, or broken upstream: note the line.
6. **Record.** Add a row and a note to the project status table, and add passing entry points to
   `tools/upstream-cases.json` (`check` for heavy ones, with `setup` commands for prerequisites) so
   [jaic-sweep](jaic-sweep.md) keeps them working. The biggest new projects can also become
   [compile-time benchmark](compile-time-benchmark.md) workloads.

## Building and running the projects

Compiling is not enough. `tools/upstream_smoke.py` builds each project with its own entry point (`first.jai`,
`build.jai` and the arguments its README documents) and runs the result without a person at the keyboard. The
checks are in `tools/upstream-smoke.json`; CI runs them on linux-x64, linux-arm64, macos-arm64 (the
`upstream-smoke` job of `ci.yml`) and windows-x64 (the job of the same name in `windows-native.yml`).

```sh
python3 tools/fetch_upstreams.py --only $(python3 tools/upstream_smoke.py --jaic x --projects)
python3 tools/upstream_smoke.py --jaic target/release/jaic            # every check for this machine
python3 tools/upstream_smoke.py --jaic target/release/jaic --filter focus -v
```

Each check copies its project (and the `link`ed siblings) into a scratch directory, so a build that writes
into its tree leaves `corpus/upstream/` alone, then runs its steps in order:

| step `kind` | what it does |
| --- | --- |
| `jaic` | `jaic <argv>` in the project (`cwd` selects a subdirectory); `outputs` must exist afterwards, `contains`/`ordered`/`not_contains` check what it prints |
| `run` (default) | runs a program, feeds `stdin`, checks the exit code (`exit`) and the output |
| `gui` | starts a windowed program, requires it to be running after `seconds`, takes screenshots before and after it started, and requires the screen to have changed (see below), then stops it |
| `serve` | starts a server on a free port (`{port}` in `argv`), fetches `get`, checks status and body |
| `lsp` | sends `initialize` over stdio and reads frames until the reply |

The `gui`, `serve` and `lsp` programs start in their own process group (a new session on POSIX, `CREATE_NEW_PROCESS_GROUP` plus `taskkill /T` on Windows), and stopping one stops the whole group. Children they start, such as the chess UI's engine, would otherwise outlive the run.

A step with `needs` runs only if that program was built (some entry points do all their work at compile time),
and one with `only` only on those platforms. A check with `only` or `skip` is a documented skip and says why;
a check without steps is a skip on every platform.

**Screenshots.** Linux runs the checks under `xvfb-run` (the display the stdlib window tests use) and takes
screenshots with `xwd`; macOS uses `screencapture`; Windows copies the screen with .NET. Whatever the desktop
looks like, the program drew something if the sampled pixels differ from the screenshot taken just before it
started (at least 0.5% of 96x54 samples, and 4 or more colors). The images are uploaded as the
`smoke-screenshots-<platform>` artifacts. On Windows, Mesa's software `opengl32.dll` is copied next to each
window program (`--dlls`), as the stdlib window tests do.

**Deviations from the projects' own steps** (each is a step of the check):

- Focus builds with `first.jai - release`, but on x64 its `src/meow_hash.jai` is edited to use the portable
  fallback hash Focus already has for CPUs without AES: the stdlib's `meow_hash` has no x64 implementation.
- reflector's `build.jai` also builds a Windows-only benchmark, so the check builds `reflector-tests.jai`, a
  driver that makes the same test workspace calls (see [upstream corpus](upstream-corpus.md)).
- jai-tracy: `generate.jai` would rewrite the pinned bindings, so libtracy is compiled from the pinned Tracy client.
- toml-jai's `tests.jai` starts a `jai` process per example; the check builds and runs each example itself.

## How to change it

- A new check: add an object to `checks` in `tools/upstream-smoke.json` (`tools/test_upstream_smoke.py`
  validates the file). Prefer assertions the project documents (its own test summary, a protocol answer) over
  "exits 0". Programs that block (servers, editors, games) need `serve` or `gui`, never a bare `run`.
- Run it on every platform before trusting it: `-os linux`/`-os windows` only type-check, and most failures
  are runtime ones. Push a `tmp/...` branch, run `ci.yml` and dispatch `windows-native.yml` there, then delete it.

- Watch for projects that write into their own directory (generated bindings, copied DLLs). Run
  `python3 tools/verify_upstreams.py` after trying one. If it reports changed files, re-run
  `fetch_upstreams.py` to restore them, and give the sweep case a `setup` step that makes the write unnecessary
  (forbear) or a `check` mode.
- A repository that imports unpinned modules from the same author (KodaJai) can only be smoke-tested once those
  are pinned too. Add them to `LIBRARIES` and link them with `MODULE_LINKS`.

## Configuration

- The star threshold and the date are arguments to the search. The date is usually the previous pass.
- `RESOURCE_PREFIXES` and `MAX_FILE_BYTES` in `tools/fetch_upstreams.py`.
- `JAIC_NATIVE_LIBS` points native builds at the prebuilt stb and other libraries (see
  [native libraries](native-libs.md)).

## Dependencies

- `gh` (authenticated) for the search, and network access for `fetch_upstreams.py`.
- clang, for projects whose cases compile vendored C (forbear).
