# Sanitizers and Miri on the compiler

## What it is

The compiler's own Rust code under AddressSanitizer, ThreadSanitizer and Miri. This is separate from [`jaic -sanitize`](../native/sanitizers.md), which instruments the programs jaic compiles. The compiler is safe Rust except the interpreter (it dereferences program addresses), the fiber and native-call glue, the memory limit and the language server's allocator hook, so these find what the type system cannot.

## How it works

`.github/workflows/sanitizers.yml` runs weekly and by dispatch:

| Job | Runs |
| --- | --- |
| `address`, `thread` | `cargo test -Zbuild-std --target x86_64-unknown-linux-gnu` for `jaic`, `jai-language-server`, `jailint` and `jacket-config` with `RUSTFLAGS=-Zsanitizer=...` on the pinned nightly. `-Zbuild-std` instruments the standard library too; without it the sanitizer misses allocations and locks made there. |
| `miri` | `tools/ci/run_miri.py`: `cargo miri test -p jaic --lib` on the modules in its `RUN` list |

ThreadSanitizer matters for the parallel parse (`jaic::pool`), the interpreter's threads (`interp::threads`) and the language server's worker. The interpreter dereferences the interpreted program's pointers, and a test of a wild pointer faults on purpose for jaic's own handler to catch; `handle_segv=0` keeps AddressSanitizer from taking that signal, and the one test that stores through a wild address (`invalid_pointer_access_still_traps...`) is skipped, since AddressSanitizer's own shadow check of the store faults first. The job runs with `--no-fail-fast` so one crashing test binary does not hide the rest.

Miri cannot call foreign functions, run inline assembly or switch stacks, so `fiber`, `interp::native`, `interp::probe`, `memory_limit` and the thread tests are not in its list. What it does cover is the sandboxed libc (`interp/sandbox`), the error cell and the record layouts. `-Zmiri-disable-isolation` is on because the sandbox tests read the clock and the environment; integer-to-pointer casts are warnings, not errors, since the interpreter hands programs real addresses by design.

## How to change it

- A module whose tests pass under Miri goes in `RUN` in `tools/ci/run_miri.py`; one that is too slow gets a `SKIP` entry with the reason.
- A sanitizer report is a bug: fix it and add a test that fails without the fix (a misaligned store is found by a debug build, an unsynchronised access by the test that runs it twice). Suppressions are a last resort and need a comment saying why the report is wrong.
- Run Miri locally with `rustup component add miri` for the pinned nightly, then `python3 tools/ci/run_miri.py`. A local ASan run: `RUSTFLAGS=-Zsanitizer=address cargo test -Zbuild-std --target <host triple> -p jaic --lib` with its own `CARGO_TARGET_DIR`.

## Configuration

`ASAN_OPTIONS` (`detect_leaks=0`: the compiler leaves memory to the OS; `handle_segv=0`), `TSAN_OPTIONS` (`halt_on_error=1`) and `MIRIFLAGS` are set in the workflow.

## Dependencies

The pinned nightly in `rust-toolchain.toml` with the `rust-src` and `miri` components (`tools/ci/install_pinned.py rust --component`).
