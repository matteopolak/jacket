# Project structure (`jaic::project`)

## What it is

The one place that knows how a Jai project is laid out for tools that do not compile all of it: the base folder (the `jacket.toml` itself is read by `crates/jacket-config`, [jacket](../tools/jacket.md)), the entry files, the `#load`/`#import` directives of a file, the files a program's `#load`s reach, and the root file a source file is compiled from. `jailsp` and `jailint` build on it; neither carries a parser or a load walk of its own.

## How it works

- **Settings.** Not here: `jacket-config` parses `jacket.toml` (serde and `toml`) and finds the nearest one through a `Files` trait (the editor counts unsaved buffers). `find_config_dir` remains for a caller that wants any one file name.
- **Base and entries.** `project_base`: the folder of the nearest config, else the innermost workspace folder holding the file. `jacket_config::entry_files`: `[project] entry`, else the `INFERRED_ENTRIES` that exist (`build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai`; `declared` says which); `first_entry` is the one `jacket` hands to jaic.
- **Import paths.** `import_paths(main_dir, configured, extra, stdlib)` is the one place the module search order is decided, and what `jaic` does for a real compile: `<main_dir>/modules`, then `configured` (`[project] import_path`, joined onto the `jacket.toml` folder by `jacket_config::Project::import_dirs`; `jacket` hands them to `jaic` as `-import_dir`), then `extra` (`-import_dir`/`-I` folders and whatever else a tool adds), then the stdlib. A folder listed twice keeps its first place; folders that do not exist stay (the compiler skips them; `existing_dirs(fs, dirs)` filters for a tool that needs only real ones). Parent folders are not searched. `jaic-cli`, `jailint` (`driver::program_of`), the playground (`jai-wasm`) and the language server's environments all call it; `add_import_dirs` is the same insertion for an environment that owns the base list. The compiler itself also looks in `modules/` next to the importing file first (`sema::modules::find_module_in`).
- **Programs.** `resolve_program(fs, file, &RootOptions, &ProgramInputs, cache)` is `resolve_root` plus the import folders that root is compiled with (`program_for`); a module root also searches the folder holding it. `ProjectFs` is the only file system it sees, so the editor's unsaved text is a parameter. `module_import` is how a program imports a module file (the language server's semantic compile builds its stub with it).
- **Directives.** `scan_directives(tokens)` finds `#load` and `#import` (with `,file`/`,dir` flags; `,string` is skipped) from lexer tokens, so it works while the text does not parse. `directive_links(file, text)` also resolves `#load`/`,file`/`,dir` targets against the file's folder. A module import needs the import paths, so its `target` stays `None`.
- **Files.** `ProjectFs` is what a walk sees: `DiskFs` resolves links in its keys; `jailsp` puts open buffers and its scan cache over the disk (`auto_import.rs` `IndexFs`) and adds the files a build metaprogram compiles to a file's `edges`. `load_closure` and `reachable_loads` follow `edges`.
- **Roots.** `resolve_root(fs, file, &RootOptions, &mut LoadCache)` returns a `Root`: a module (the `module.jai` whose loads reach the file, or the entry of an import folder) or a program (the file that loads it, else the file itself). `RootOptions` carries the choices that differ between tools:
  - `RootOptions::linter(import_dirs)`: module detection on, loaders looked for in the file's folder and two above, `TieBreak::Outermost` (the loader no other loader loads; first path wins).
  - `RootOptions::language_server(folder)`: no module detection, loaders anywhere under `folder`, `TieBreak::ModuleFirstShallowest` (climb: `module.jai`, then shallowest path, then name).
  
  The tools agree on trees with a single loader chain and differ only when several unrelated files load one file; that difference is the tie-break option, not a silent divergence. `jailint` does not use the entry files; `jailsp` tries the project's entry files before calling the loader search.

## How to change it

`ProjectFs::loader_graph` lets a file system that keeps the loaders of a tree between calls (the language server's `IndexFs`) answer the `ModuleFirstShallowest` search without reading every `#load` again; the default returns `None` and `build_loader_graph` is used.

Add a setting by extending the schema in `crates/jacket-config/src/schema.rs`. Change root rules by adding a variant to `TieBreak` or `LoaderSearch` rather than editing the existing ones, and extend `linter_and_language_server_agree_on_the_same_tree` and `tie_breaks_differ_when_loaders_disagree`. `jai-language-server` `project.rs` has a test that the real `Session::project_root` equals the linter preset on a fixture tree.

## Configuration

`jacket.toml`; see [jacket](../tools/jacket.md), the [language server](language-server.md) and [jailint](../tools/jailint.md) docs for their keys.

## Dependencies

`jaic::lexer` (tokens), `jaic::paths` (normalisation, canonical paths), `jaic::ast::ImportSource`.
