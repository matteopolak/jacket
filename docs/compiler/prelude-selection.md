# Prelude selection

The prelude supplies the built-in types, `Context` and allocators loaded before a program. Its module name is always `prelude`; `#import "Preload"` aliases it whatever the file is called.

```sh
JAIC_STDLIB=/path/to/stdlib JAIC_PRELUDE=Base.jai jaic hello.jai
```

## How it works

- The stdlib folder comes from `JAIC_STDLIB`, then `stdlib/` beside the executable (following symlinks), then the build-time repository fallback.
- `jaic::PRELUDE_ENTRIES` (`crates/jaic/src/lib.rs`) lists candidates relative to that folder in priority order: `prelude/module.jai`, then `Preload.jai`. Detection uses the compiler's `FileSystem`, so browser files follow the same rules.
- `JAIC_PRELUDE` overrides detection. Absolute paths may be outside the stdlib; relative ones resolve from it; empty means detect. A missing explicit file is an error, never a fallback.
- The stdlib folder stays the module root wherever the prelude file lives. Relative `#load` paths still resolve from the file containing the directive.

## How to change it

- Add a relative path to `PRELUDE_ENTRIES` with a detection-order and root-recovery test.
- Front ends call `Options::set_stdlib`, which sets both `preload` and `preload_stdlib`; keep them together. `stdlib_root_from_preload` is only for detected layouts and older callers.
- The selected file is excluded from import suggestions and language-server auto-import by path.

Editors: VS Code `jai.stdlib.prelude` and Neovim `stdlib = { prelude = 'Base.jai' }` pass `JAIC_PRELUDE` to `jailsp` and `jaic`. The browser has no environment and detects on its virtual filesystem.
