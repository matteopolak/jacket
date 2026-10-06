# PGO and BOLT release builds

## What it is

The released `jaic`, `jailsp` and `jailint` are built with profile-guided optimisation (PGO): rustc compiles them once with instrumentation, they run a training workload from this repository, and rustc compiles them again using the recorded profile. On Linux the PGO binaries are then rewritten by [BOLT](https://github.com/llvm/llvm-project/tree/main/bolt), a post-link optimiser that reorders functions and basic blocks from a second profile. `tools/build_pgo.py` does all of it, locally and in [the release workflow](releases.md).

Only our Rust code is profiled. LLVM, which `jaic` links for native output, is the prebuilt library from the LLVM release; BOLT does reorder its code in the Linux binary along with ours.

## How it works

`tools/build_pgo.py` runs five steps, all under `<target-dir>/pgo/`:

1. **Instrumented build** (`pgo/instrumented`): `cargo build --release --target <triple>` of `jaic-cli`, then of `jai-language-server` and `jailint`, with `-Cprofile-generate`.
2. **Training** (`train()`), with `LLVM_PROFILE_FILE=pgo/profraw/%4m.profraw` so the hundreds of `jaic` runs merge into a few files per binary as they exit:
   - `tools/jaic-sweep.py` over `corpus negative stdlib modules examples` (check and interpreted run);
   - the sweep again with native builds: `--opt O0` over `corpus stdlib examples` and `--opt O2` over `corpus examples`;
   - `jaic run` of each `benchmarks/*.jai` (interpreter dispatch, compile-time execution);
   - `jaic build jaifmt/main.jai` at `-O0` and `-O2` (a real multi-file program), `jaic check examples/tour/main.jai`;
   - `jailint` over `stdlib`, `examples` and `tests/stdlib`;
   - a scripted `jailsp` session (`lsp_session()`): it opens a few files, asks for symbols, semantic tokens, folding, inlay hints, hovers, completions, definitions, highlights, signature help and references across each file, types a new procedure character by character with completions along the way, restores the file, closes it and shuts down.

   A failing training step only prints a warning (`--strict` makes it fatal): the profile needs representative work, and correctness is checked separately (see below).
3. **Merge** with the `llvm-profdata` of rustup's `llvm-tools` component for the pinned toolchain. Raw profiles must be read by the LLVM that rustc was built with (the nightly's LLVM, not the LLVM that `jaic` links).
4. **Optimised build** (`pgo/optimized`) with `-Cprofile-use=pgo/merged.profdata`.
5. **BOLT** (`--bolt`, Linux only): the optimised build is linked with `-Wl,--emit-relocs` so BOLT can move code. Each binary is instrumented (`llvm-bolt -instrument`, one `.fdata` per process), the whole training workload runs again with the instrumented binaries, the profiles are combined with `merge-fdata`, and `llvm-bolt -data=... -reorder-blocks=ext-tsp -reorder-functions=cdsort -split-functions -split-all-cold -split-eh -icf=all` writes the final binary. Instrumentation mode is used because GitHub's runners have no `perf` LBR sampling.

The results are copied to `--out` (default `<target-dir>/pgo/dist`).

### What runs where

| Release target | Optimisation | Notes |
|---|---|---|
| Linux x86-64 | PGO + BOLT | `llvm-bolt` and `merge-fdata` come from the official LLVM release tarball the job already unpacks (`$RUNNER_TEMP/llvm/bin`). |
| macOS arm64 | PGO | BOLT's Mach-O support is not production ready. |
| Windows x86-64, arm64 | PGO | BOLT only rewrites ELF. rustup ships the profiler runtime for both MSVC targets. |

### Gotchas

- **C code compiled by the `cc` crate.** cc copies `-Cprofile-generate`/`-Cprofile-use` into the flags of clang-compiled C code (llvm-sys's target wrappers). That clang is not rustc's LLVM, so its profile records have a different layout: the instrumented `jaic` crashed in `initializeValueProfRuntimeRecord` while writing its profile at exit, and the system clang could not read our `.profdata`. The script appends `-fno-profile-generate -fno-profile-use` to `CFLAGS_<triple>` (cc puts environment flags last). MSVC targets use `cl.exe`, which cc leaves alone.
- **RUSTFLAGS.** When `RUSTFLAGS` is set it overrides every `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`, so the script extends whichever one is in effect. The release workflow puts `+crt-static` (Windows) and `-fuse-ld=lld` (macOS) in the target variable, and the PGO flags are appended to it. The script always passes `--target`, which keeps the flags off build scripts and proc macros.
- **Paths with spaces** cannot be carried in RUSTFLAGS; the script refuses a target directory containing one.

## How to change it

- **Training workload:** edit `train()` (commands) and `LSP_FILES`/`lsp_session()` (the editor session). Keep it inside this repository so CI can run it; add work that the released binaries spend real time on. A heavier workload makes the release job slower, twice on Linux.
- **BOLT options:** the `llvm-bolt` call at the end of `main()`. Check `-dyno-stats` in the job log after a change.
- **A new release target:** if rustup ships `profiler_builtins` for it (`lib/rustlib/<triple>/lib/libprofiler_builtins-*.rlib`), PGO works unchanged; BOLT needs an ELF target.
- To reuse a profile instead of training (for example to iterate on BOLT), pass `--profile path/to/merged.profdata`.

Before shipping a change, check the optimised binaries as in the next section.

### Checking and measuring locally (macOS)

```sh
rustup component add llvm-tools          # once, for the pinned toolchain
python3 tools/build_pgo.py --jobs 4      # dynamic LLVM, like a development build
python3 tools/jaic-sweep.py --jaic $CARGO_TARGET_DIR/pgo/dist/jaic corpus negative stdlib modules upstream examples
python3 tools/jaic-diff.py --jaic $CARGO_TARGET_DIR/pgo/dist/jaic corpus
```

Compare against a plain `cargo build --release` by alternating the two binaries on the same workloads, as [benchmarks](benchmarks.md#comparing-two-builds) describes. Results on an Apple M5 are below.

### Measured gain

Apple M5, dynamic LLVM 22.1.8, plain `cargo build --release --target aarch64-apple-darwin` (A) against `build_pgo.py` (B). The two binaries alternated on each workload, 7 runs each, then the whole series again with 11 runs. The machine was shared, so medians are noisy and minimums are a cross-check. Starred workloads are corpus projects that are not part of the training.

| Workload | A median | B median | B/A (runs 1, 2) |
|---|---|---|---|
| `jaic check` Focus `first.jai`* | 1.90 s | 1.74 s | 0.91, noisy (min 1.86 → 1.46 s, 1.67 → 1.51 s) |
| `jaic check` Jails `build.jai`* | 0.171 s | 0.157 s | 0.92, 0.91 |
| `jaic check` sgpu examples* | 0.436 s | 0.406 s | 0.93, 0.93 |
| `jaic run` jaison tests* | 0.357 s | 0.342 s | 0.96, 0.94 |
| `jailint stdlib` | 1.83 s | 1.67 s | 0.91, 0.93 |
| scripted `jailsp` session | 3.04 s | 2.80 s | 0.92, 1.01 (min 2.96 → 2.57 s, 2.86 → 2.65 s) |
| `benchmarks/interp-strings.jai` | 1.24 s | 1.22 s | 0.98, 0.96 |
| `benchmarks/interp-loops.jai` | 0.386 s | 0.370 s | 0.96, 0.95 |
| `jaic build jaifmt/main.jai -O0` | 0.152 s | 0.148 s | 0.97, 0.98 |
| `jaic build jaifmt/main.jai -O2` | 0.825 s | 0.833 s | 1.01, 0.99 |

The front end (parsing, sema, compile-time execution) is 6 to 9% faster; the interpreter's hot loops gain 2 to 5%. `-O2` builds do not change, since their time is LLVM's optimiser, which we do not profile. The PGO binaries passed the full sweep (`corpus negative stdlib modules examples upstream`: 852 passed), the native sweep at `-O0` and `-O2` (`corpus stdlib modules`: 332 passed each), and `jaic-diff.py --backends interp,native,native-O2 corpus gen:1:40` (84 agree). The whole PGO build took 93 s on this machine, against about 20 s for a plain release build.

### Linux with BOLT

On an x86-64 or arm64 Linux machine with an LLVM that has BOLT (the official release tarball, or apt.llvm.org's `llvm-<N>-tools`/`bolt-<N>` packages):

```sh
rustup component add llvm-tools
python3 tools/build_pgo.py --bolt --llvm-bin /path/to/llvm/bin --jobs 4
```

Without `--llvm-bin` the script looks in `$LLVM_SYS_*_PREFIX/bin`, then on `PATH` (also as `llvm-bolt-<N>`).

## Configuration

- `tools/build_pgo.py` options: `--llvm dynamic|static|none` (how `jaic` links LLVM; releases use `static`, `none` also skips native training steps), `--bolt`, `--llvm-bin`, `--jobs`, `--target`, `--target-dir` (default `CARGO_TARGET_DIR`, else `target`), `--out`, `--profile`, `--step-timeout` (default 1800 s per training step), `--strict`, `--verbose`.
- Environment: `CARGO_BUILD_TARGET` (default target), `RUSTFLAGS`/`CARGO_TARGET_<TRIPLE>_RUSTFLAGS` (extended), `CFLAGS_<triple>` (extended), `LLVM_SYS_*_PREFIX` (where BOLT is looked for). Training runs with `JAIC_STDLIB` pointing at the repository's `stdlib/`.
- The release workflow passes `--llvm static --jobs 4`, plus `--bolt --llvm-bin $RUNNER_TEMP/llvm/bin` on Linux.

## Dependencies

- rustup's `llvm-tools` component for the pinned nightly (`llvm-profdata`), and the profiler runtime rustup ships with each target's standard library.
- Python 3.11+, and everything the [sweep](jaic-sweep.md) needs for its native sets (a C linker; on macOS and Linux the [third-party native libraries](native-libs.md), which the sweep builds when missing).
- For BOLT: `llvm-bolt` and `merge-fdata` (with its instrumentation runtime `libbolt_rt_instr.a`) from LLVM 22 or later.
