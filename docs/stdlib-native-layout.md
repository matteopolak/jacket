# Stdlib native layout

## What it is
How the platform-specific parts of `Socket`, `rpmalloc` and the shared C-library declarations are organised under `stdlib/`.

## How it works
- `Socket/module.jai` loads `internal/net/socket.jai`, which `#load`s one ABI file per OS (`socket-abi-{macos,linux,android,windows}.jai`, generated) plus the shared behavior files. Non-Windows targets share `socket-posix.jai` and `socket-cmsg.jai`; field accessors are per OS (`socket-fields-*.jai`).
- `internal/bindings/rpmalloc-{heaps,sdk,callbacks}` hold one `#if` block per supported (OS, CPU, profile) combination. Foreign library and symbol names must be string literals, so the blocks cannot be computed. An unsupported combination hits `#assert false`.
- `internal/libc` declares `c_malloc`, `c_realloc`, `c_free` and `c_abort` once; runtime, allocation and file backends import it instead of redeclaring.
- `internal/files/windows-text.jai` holds the single UTF-8/UTF-16 conversion set (`to_utf16`, `utf16_path`, `from_utf16`, `utf8_text`).
- `internal/numerics/ui_types.jai` serves both `GetRect` and `GetRect_LeftHanded`; the handedness differences are under `#if LEFT_HANDED`.

## How to change it
- Socket ABI: edit `docs/specs/stdlib/modules/socket.md` and rerun `tools/generate_socket_abi.py`; do not hand-edit `socket-abi-*.jai`.
- rpmalloc profiles: edit `SUPPORTED` in `tools/native_bindings/rpmalloc_profiles.py` and regenerate.
- Modules cannot re-export names, so public module folders `#load` internal files into their own scope.

## Configuration
`rpmalloc` options are module parameters (`ENABLE_ASSERTS`, `ENABLE_STATISTICS`, ...).

## Dependencies
`tools/generate_socket_abi.py`, `tools/native_bindings/*`, `jaifmt`.
