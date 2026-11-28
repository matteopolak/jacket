# Per-CPU Linux POSIX bindings

## What it is

`stdlib/POSIX/declarations.jai` is generated from `docs/specs/stdlib/modules/posix.md`, which records the x86-64 and arm64 Linux ABI. Every other Linux CPU (`i686`, `s390x`, `powerpc64`, `armv7`) takes the declarations that differ from that ABI from its own glibc headers.

## How it works

`tools/native_bindings/vendor_linux_posix.py <arch>` runs clang in the [cross container](../tools/linux-container.md) over the CPU's headers, with 64-bit `off_t` and `time_t` (`_FILE_OFFSET_BITS=64`, `_TIME_BITS=64`). For each Linux name the spec exports it renders the header's own declaration with `tools/native_bindings/linux_cabi.py` and writes the ones that differ to `stdlib/internal/bindings/vendor/linux-posix-<arch>.json`:

- **Functions.** Parameter and result widths, and the symbol the header links to (`clock_gettime` is `__clock_gettime64` on i686).
- **Records and typedefs.** Layouts, from the field types clang desugars for that target (`long` is 4 bytes on i686).
- **Constants.** Values from the headers, including the kernel numbers the spec only has for x64 and arm64.

`generate.py` then emits each entry as `#if OS == .LINUX && ARCHITECTURE == .X86` (the [`ARCHITECTURE`](../compiler/targets.md) constant), and adds `&& ARCHITECTURE != .X` to the 64-bit declaration it replaces, so a 32-bit target never sees a 64-bit layout. Names the headers can't express (bit-field records, opaque `FILE`) are listed under `uncovered` in the JSON and are excluded from that CPU instead of being guessed.

## How to change it

Regenerate after changing the spec or the renderer:

```bash
tools/dev/linux_x64.sh cross-up
PYTHONPATH=tools:tools/native_bindings python3 tools/native_bindings/vendor_linux_posix.py i686
PYTHONPATH=tools:tools/native_bindings python3 tools/native_bindings/generate.py --module POSIX
```

Add a CPU in the `ARCHES` table of `vendor_linux_posix.py`. `--check` fails when the committed JSON is stale. Run it for `i686`, `s390x` and `armv7`; a record with `__attribute__((aligned(N)))` becomes `struct #align N`, and a `typedef union { ... } name` the spec declares as a pointer or scalar becomes a record.

## Known gaps on non-x64 Linux targets

The stdlib modules that pass on each CPU are listed in `tools/dev/<cpu>-roots.txt` and run in CI. Not covered:

- `i686`, `armv7`: Socket, Http, Grpc, WebSocket and Tls, because `internal/net/socket-abi-linux.jai` has one 64-bit layout; the third-party native libraries (X11, mbedtls, freetype, lz4, stb), which have no 32-bit builds; the `Syscall` intrinsic, which is x86-64 only; musl.
- `s390x`: fibers, which have no switch routine.
- `powerpc64`: fibers and IBM double-double `long double`.
- Big-endian: the interpreter sandbox's hard-coded record layouts (`stat`, `dirent`) are little-endian.

## Configuration

`CROSS_SCRATCH` and the container image come from `tools/dev/linux_x64.sh`. The probe's header list is `PROBE_HEADERS` in `vendor_linux_posix.py`.

## Dependencies

The cross container with `libc6-dev-<arch>-cross`, clang, `ARCHITECTURE` in the compiler.
