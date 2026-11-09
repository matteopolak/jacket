# Stdlib native layout

How the platform-specific parts of `Socket`, `rpmalloc` and the shared C-library declarations are organised under `stdlib/`.

## How it works

- `Socket/module.jai` loads `internal/net/socket.jai`, which `#load`s a generated ABI file per OS (`socket-abi-{macos,linux,android,windows}.jai`), shared `socket-posix.jai` and `socket-cmsg.jai`, and per-OS `socket-fields-*.jai`.
- `internal/bindings/rpmalloc-{heaps,sdk,callbacks}` have one `#if` block per (OS, CPU, profile). Library and symbol names must be string literals, so they cannot be computed. Unsupported combinations hit `#assert false`.
- `internal/libc` declares `c_malloc`, `c_realloc`, `c_free`, `c_abort` once. `internal/files/windows-text.jai` holds UTF-8/UTF-16 conversions. `internal/numerics/ui_types.jai` serves `GetRect` and `GetRect_LeftHanded`.

## How to change it

- Socket ABI: edit `docs/specs/stdlib/modules/socket.md` and rerun `PYTHONPATH=tools python3 tools/generators/generate_socket_abi.py` (`--check` verifies). Never hand-edit `socket-abi-*.jai`.
- rpmalloc profiles: edit `SUPPORTED` in `tools/native_bindings/rpmalloc_profiles.py` and regenerate. Options are module parameters (`ENABLE_ASSERTS`, `ENABLE_STATISTICS`, ...).
- Native binding declarations (`stdlib/*/declarations.jai`): edit the module's spec in `docs/specs/stdlib/modules/` and run `python3 generate.py --formatter ../../target/jaifmt` in `tools/native_bindings/`. CI runs it with `--check`. Per-CPU layouts are separate `Variant N (OS, CPU):` fences with matching `[OS && CPU]` inventory lines.
- Notes on a bodiless `#foreign` (or `#elsewhere`) spec entry: write them after the signature in the spec (`f :: (p: *void) #foreign lib @FreesArgument`). `generate.py` splits trailing `@Name` notes off with `split_notes` and puts them back after the declaration's `;`, so `jailint` reads them from the generated `declarations.jai`. A procedure the generator wraps gets the same notes after its closing brace instead. Notes written anywhere but the end of the entry are not moved.
- Public modules expose internal code in one of two ways:
  - **`#load` into their own scope.** The usual way.
  - **Re-export a private module with `using Declarations :: #import "internal/bindings/..."`.** Used by `POSIX`, `Windows`, `Curl` and `Objective_C`, listed as `REEXPORTED` in `tools/native_bindings/config.py`. The library's own code imports the private module directly, so a project that ships its own `POSIX` never reaches the library's internals through it.
