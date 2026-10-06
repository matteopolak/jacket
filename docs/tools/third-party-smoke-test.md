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

## How to change it

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
