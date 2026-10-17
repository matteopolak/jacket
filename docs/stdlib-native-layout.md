# Stdlib native layout

## What it is
How the platform-specific parts of `Socket`, `rpmalloc` and the shared C-library declarations are organised under `stdlib/`.

## How it works
- `Socket/module.jai` loads `internal/net/socket.jai`, which `#load`s a generated ABI file per OS (`socket-abi-{macos,linux,android,windows}.jai`), shared `socket-posix.jai` and `socket-cmsg.jai`, and per-OS `socket-fields-*.jai`.
- `internal/bindings/rpmalloc-{heaps,sdk,callbacks}` have one `#if` block per (OS, CPU, profile); library and symbol names must be string literals, so they cannot be computed. Unsupported combinations hit `#assert false`.
- `internal/libc` declares `c_malloc`, `c_realloc`, `c_free`, `c_abort` once. `internal/files/windows-text.jai` holds the UTF-8/UTF-16 conversions. `internal/numerics/ui_types.jai` serves `GetRect` and `GetRect_LeftHanded` (`#if LEFT_HANDED`).

## How to change it
- Socket ABI: edit `docs/specs/stdlib/modules/socket.md` and rerun `tools/generate_socket_abi.py`; do not hand-edit `socket-abi-*.jai`.
- rpmalloc profiles: edit `SUPPORTED` in `tools/native_bindings/rpmalloc_profiles.py` and regenerate.
- Modules cannot re-export names, so public module folders `#load` internal files into their own scope.

`rpmalloc` options are module parameters (`ENABLE_ASSERTS`, `ENABLE_STATISTICS`, ...). Depends on `tools/generate_socket_abi.py` and `tools/native_bindings/*`.
