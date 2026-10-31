# Project structure (`jaic::project`)

The one place that knows how a Jai project is laid out for tools that don't compile all of it: the base folder, the entry files, the `#load`/`#import` directives of a file, which files a program's loads reach, and the module search order. `jaic`, `jailint` and `jailsp` all use it.

`jacket.toml` itself is read by `crates/jacket-config` ([jacket](../tools/jacket.md)).

## How it works

- **Base and entries.** `project_base` is the folder of the nearest config, else the innermost workspace folder holding the file. Entries are `[project] entry`, else whichever of `build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai` exist.
- **Import paths.** `import_paths(main_dir, configured, extra, stdlib)` decides the module search order for every tool. Tools set up a compile the same way: `ProgramInputs::for_file` finds the enclosing standard library (a checkout's `stdlib/` wins), then `sema::Options::for_program` builds the paths.
- **Directives.** `scan_directives(tokens)` finds `#load` and `#import` from lexer tokens, so it works while the text does not parse.
- **Files.** `ProjectFs` is what a walk sees: `DiskFs`, or the editor's open buffers over the disk. Metaprogram-added files are recorded as `edges`, which `load_closure` follows.
- **Roots.** `resolve_root` returns a module (the `module.jai` whose loads reach the file, or the entry of an import folder) or a program (the file that loads it, else the file itself). `RootOptions` differs per tool:
  - `linter(import_dirs)`: module detection on, loaders looked for in the file's folder and two above, outermost loader wins.
  - `language_server(folder)`: no module detection, loaders anywhere under `folder`, `module.jai` first then shallowest.

The tools agree whenever one loader chain exists and differ only when several unrelated files load one file; that is the tie-break option, not a divergence.

## Module search order

`#import "Name"` finds `Name.jai`, else `Name/module.jai`, in these roots, first match wins (`sema::modules::search_roots`):

| # | Root | Set by |
| --- | --- | --- |
| 0 | the bootstrap entry, only for the name `Preload` | `Options::preload` |
| 1 | `modules/` next to the importing file | the file's folder |
| 2 | `<main_dir>/modules` | `import_paths` |
| 3 | `[project] import_path` folders | `jacket.toml` (passed as `-import_dir`) |
| 4 | `-import_dir` / `-I` folders and tool additions | the command line |
| 5 | the standard library | `set_stdlib` |

If no root has `Preload`, the prelude is looked up in the same roots (`JAIC_PRELUDE` overrides). There are no special cases: no `Basic` rule, and a file's own location doesn't count.

So a project folder holding a module named like a library one (`Windows`, `Thread`) replaces it for the whole program, library modules included. For that reason `stdlib/internal` reaches platform bindings through private modules (`internal/bindings/windows-api`, `posix-api`, ...). `X11` is the exception: its globals are public API, so the window code shares the program's module.

## How to change it

- Add a setting in `crates/jacket-config/src/schema.rs`.
- Change root rules by adding a variant to `TieBreak` or `LoaderSearch`, and extend `linter_and_language_server_agree_on_the_same_tree` and `tie_breaks_differ_when_loaders_disagree`.
- Change the search order in `search_roots`, extend `search_roots_in_order` (`sema/modules/filesystem.rs`), and `import_paths` for roots outside the compiler. Fixtures: `crates/jaic-cli/tests/module_r*`.
- A file system that keeps the loader graph between calls (the language server's `IndexFs`) overrides `ProjectFs::loader_graph`.

## Dependencies

`jaic::lexer`, `jaic::paths`, `jaic::ast::ImportSource`, `jacket-config`.
