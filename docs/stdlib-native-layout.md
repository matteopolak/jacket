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
- Modules cannot re-export names, so public module folders `#load` internal files into their own scope.
