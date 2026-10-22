# Prelude selection

The prelude supplies the built-in types, `Context` and allocators loaded before a program. Its internal module name remains `prelude`; `#import "Preload"` aliases that same module even when the file has another name.

## How it works

The stdlib folder comes from `JAIC_STDLIB`, then `stdlib/` beside the executable (including its symlink target), then the build-time repository fallback. `jaic::PRELUDE_ENTRIES` in `crates/jaic/src/lib.rs` lists candidate files relative to that folder, in priority order. It holds `prelude/module.jai`, then `Preload.jai` (a single file at the folder's root). Selection uses the compiler's `FileSystem`, so virtual browser files follow the same detection rules.

`JAIC_PRELUDE` overrides detection. An absolute path can live outside the stdlib; a relative path resolves from the stdlib folder. An empty value enables detection. A missing explicit file is an error and does not fall back to a detected entry. Missing-library diagnostics list the explicit path and all detection candidates.

```sh
JAIC_STDLIB=/path/to/stdlib JAIC_PRELUDE=Base.jai jaic hello.jai
```

The configured stdlib stays the module root regardless of the prelude file's location. Library-file analysis, private imports and runtime loading must use that root rather than the custom file's parent. Relative `#load` paths still resolve from the file containing each directive.

## How to change it

Add a relative file path to `PRELUDE_ENTRIES` and add a detection-order and root-recovery test beside it. Both root files such as `Base.jai` and nested paths such as `prelude/module.jai` are supported. `find_preload` returns an existing selection; `preload_path` retains a missing requested path for diagnostics. Tool front ends call `Options::set_stdlib`, which sets both `preload` and `preload_stdlib`. Keep those together when changing configuration.

`stdlib_root_from_preload` recovers roots for detected layouts and older direct `Options::preload` callers. Explicit settings must use `Options::set_stdlib` instead. The selected file is excluded from compiler import suggestions and language-server auto-import modules by path, independently of its filename.

## Configuration

VS Code's `jai.stdlib.prelude` and Neovim's `stdlib = { prelude = 'Base.jai' }` pass `JAIC_PRELUDE` to both `jailsp` and `jaic`. Both editors expand `~`; empty settings inherit the process environment. Change `jai.stdlib.path` / Neovim `stdlib.path` to configure the module folder separately. The browser has no host environment settings and detects entries on its virtual filesystem.

## Dependencies and verification

Selection depends only on `jaic::sema::FileSystem` and the standard path/environment APIs. The CLI, language server, linter and browser share the resolver; `jaifmt` formats syntax without loading a prelude. Tests cover ordered detection, both entry shapes, explicit absolute/relative/missing paths, diagnostics, and library root recovery. Environment tests run in child processes to avoid changing process-wide variables during parallel tests.
