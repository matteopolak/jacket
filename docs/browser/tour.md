# Language tour (playground default workspace)

## What it is

`examples/tour/` is a multi-file Jai program that walks through most of the language, one topic per file. The hosted playground (https://matteopolak.com/playground/jai) opens it as the default workspace. It ships in the browser bundle as `tour/` plus a `tour.json` index, so it is versioned and tested with the compiler that runs it.

| File | Shows |
| --- | --- |
| `main.jai` | Loads every file and runs the stops from a table of `{title, procedure}` with printed headings |
| `tour.md` | Plain-text guide: map of the files and things to try |
| `basics/basics.jai` | Declarations, constants, casts and `xx`, local procedures, named multiple returns, named/default arguments, overloading, procedure values |
| `types/structs.jai` | Field defaults, struct literals, `operator +`/`*` with `#symmetric`, `#as using`, `using` parameters, a union |
| `types/enums.jai` | Enums with explicit values, `enum_flags` operators and printing, `if #complete ... ==` with `#through` |
| `data/arrays.jai` | Fixed arrays, views into arrays, `[..]` with `array_add`, named `it`/`it_index`, `for <`, `for *`, `remove it`, `array_find`, a sieve |
| `data/strings.jai` | Slicing, `%1` positional arguments, formatFloat/formatInt, String_Builder FizzBuzz, String module helpers, a here-string banner |
| `memory/memory.jai` | `defer` order, `New`/`free`, temporary storage, a counting allocator via `push_allocator`, a logger via `push_context` |
| `generics/polymorphism.jai` | `$T` procedures, a polymorphic `Ring(Item, CAPACITY)` struct, `$R/Ring`, `$T/interface`, `#modify`, `$$` with `is_constant`, `#bake_arguments` |
| `meta/compile_time.jai` | `#run` tables, `#assert`, `#if OS`, an enum and array generated with `#insert #run` from a here-string, `#code`, `#compile_time` |
| `meta/macros.jai` | `#expand` with `Code`, backtick variables, `` `defer`` and `` `return``, `for_expansion` for a Collatz sequence and a linked list |
| `meta/reflection.jai` | Types as values, `type_info` members, offsets and notes, an `Any`-based JSON writer |
| `meta/metaprogram.jai` | A `#run` that compiles a second program in its own workspace and reports on its TYPECHECKED messages (a tiny style checker) |
| `finale/raymarch.jai` | An ASCII ray marcher (signed distance fields, smooth union, soft lighting, shadows, fog) built on `Math`'s `Vector3` |

## How it works

- **Compiler side.** `tests/examples.json` lists example cases. The tour's case has `bundle: "tour"`, so `tools/build_scripting_wasm.py` copies `examples/tour/**` (dotfiles skipped) to `<bundle>/tour/` and writes `<bundle>/tour.json`:

  ```json
  { "schema_version": 1, "main": "main.jai", "files": ["basics/basics.jai", "...", "tour.md"] }
  ```

  `package_browser_release.py` inventories them like every other asset (`tour.json` and `tour/main.jai` are required), and `check_browser_release.mjs` checks the index matches the folder exactly, then runs the **staged** tour in the staged engine.
- **Tests.** `tools/jaic-sweep.py examples` runs the tour natively (`jaic run main.jai -os wasm`) and checks the key lines in `stdout_contains`; `tools/check_scripting_wasm.mjs` and `check_browser_release.mjs` run it in the wasm engine under the case's `budget` (200,000,000 basic blocks, the playground's limit) via `tools/examples_wasm.mjs`.
- **Portfolio side.** `src/lib/jai/starter.ts::loadStarter` fetches `/jai/<rev>/tour.json`, then every listed file from `/jai/<rev>/tour/`, adds the default `jaifmt.toml`, and opens `main.jai` and `tour.md` in tabs. A release without a tour (or any fetch or validation failure) falls back to the built-in two-file starter. `scripts/sync-jai-web.ts` copies the tour for `JAI_WEB_LOCAL` builds; released bundles are extracted whole.

## How to change it

- Add a stop: write `examples/tour/<topic>/<file>.jai` with a `tour_<topic> :: ()` procedure, `#load` it in `main.jai` and add `.{ "Title", tour_<topic> }` to `stops`. Mention it in `tour.md`.
- All files share one global scope, so give helpers distinct names. A helper named `scale`, for example, collides with `Math`'s and makes `#bake_arguments scale(...)` ambiguous.
- Keep it browser-safe: no processes, sockets, windows or native `#foreign` libraries (see how `tests/stdlib` programs skip those parts under `OS == .WASM`, [playground](playground.md)). Keep runtime small: auto-run recompiles and reruns on every edit.
- Keep lines short (the editor is narrow on phones) and run jaifmt: `jaic run tools/jaifmt/main.jai -- --check "$PWD/examples/tour"`. The portfolio's `JAI_WASM_DIR=<bundle> pnpm test:jai` also checks the tour is formatted under the playground's default `jaifmt.toml`.
- When output changes, update `stdout_contains` in `tests/examples.json`.
- Gotchas: there is no `offset_of` (use `type_info(T).members`), and `builder_to_string`, `NewArray` and the `String` helpers take no allocator argument (wrap them in `push_allocator(temp)`).

## Configuration

- `tests/examples.json`: `directory`, `main`, `args` (native sweep), `budget` (wasm checks), `bundle` (staged name), `stdout_contains`, `stdout_excludes`.
- Portfolio: `MAX_TOUR_FILES` (64) and `MAX_TOUR_BYTES` (1 MiB) in `starter.ts`; the release inventory allows 64 files (`MAX_FILES`, schema `maxItems`).

## Dependencies

The `Basic`, `Math`, `String` and `Compiler` modules; the wasm engine's sandbox host (virtual clock, in-memory files). Tooling: `tools/build_scripting_wasm.py`, `tools/package_browser_release.py`, `tools/check_browser_release.mjs`, `tools/examples_wasm.mjs`, `tools/jaic-sweep.py`.
