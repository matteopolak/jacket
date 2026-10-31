# Sanitizers and Miri on the compiler

The compiler's own Rust code under AddressSanitizer, ThreadSanitizer and Miri. This is separate from [`jaic -sanitize`](../native/sanitizers.md), which instruments the programs jaic compiles. The compiler is safe Rust except the interpreter (it dereferences program addresses), the fiber and native-call glue, the memory limit and the language server's allocator hook; these find what the type system can't.

## How it works

`.github/workflows/sanitizers.yml` runs weekly and by dispatch:

| Job | Runs |
| --- | --- |
| `address`, `thread` | `cargo test -Zbuild-std --target x86_64-unknown-linux-gnu` for `jaic`, `jai-language-server`, `jailint` and `jacket-config` with `RUSTFLAGS=-Zsanitizer=...`. `-Zbuild-std` instruments the standard library too. |
| `address` (second step) | the same for `jaic-cli --test native`; the built programs themselves are not sanitized |
| `miri` | `tools/ci/run_miri.py`: `cargo miri test -p jaic --lib` on the modules in its `RUN` list |

- ThreadSanitizer covers the parallel parse (`jaic::pool`), the interpreter's threads and the language server worker.
- `handle_segv=0` keeps ASan from taking the fault jaic's own handler catches on purpose; the one test that stores through a wild address is skipped.
- Miri can't call foreign functions, run inline assembly or switch stacks, so `fiber`, `interp::native`, `interp::probe`, `memory_limit` and the thread tests aren't in its list. It covers the sandboxed libc, the error cell and record layouts. `-Zmiri-disable-isolation` is on because sandbox tests read the clock and environment.

## How to change it

- A module whose tests pass under Miri goes in `RUN` in `run_miri.py`; one too slow gets a `SKIP` entry with the reason.
- A sanitizer report is a bug: fix it and add a test that fails without the fix. Suppressions are a last resort and need a comment.
- Locally: `rustup component add miri` for the pinned nightly, then `PYTHONPATH=tools python3 tools/ci/run_miri.py`. For ASan: `RUSTFLAGS=-Zsanitizer=address cargo test -Zbuild-std --target <host triple> -p jaic --lib` with its own `CARGO_TARGET_DIR`.

## Configuration

`ASAN_OPTIONS` (`detect_leaks=0`, `handle_segv=0`), `TSAN_OPTIONS` (`halt_on_error=1`) and `MIRIFLAGS` are set in the workflow. The pinned nightly in `rust-toolchain.toml` needs the `rust-src` and `miri` components.
