# Sanitizers for native builds

`jaic -sanitize address,undefined` instruments a native build with AddressSanitizer and the UBSan checks that exist at the LLVM IR level, and links the matching runtime. It catches native-codegen bugs that happen to run fine (wrong layout, ABI marshalling that writes past a temporary, slots that are too small) and ordinary memory errors.

```
$ jacket run uaf.jai -sanitize address
==58208==ERROR: AddressSanitizer: heap-use-after-free on address 0x603000001010 ...
    #5 0x000102bff350 in main.7 uaf.jai:7
```

## How it works

Compile side (`Sanitize` in `crates/jaic-llvm/src/lib.rs`, applied in `emit_module`):

- `address`: every function gets `sanitize_address`, then the `asan` pass runs after optimisation, with redzones on every `Slot` alloca and global. A sanitized build is one codegen unit and writes a `.dSYM` at `-O0` ([debug info](debug-info.md)).
- `undefined`: the `bounds-checking<rt-abort>` pass (Clang's `local-bounds`). At `-O0` `sroa` runs first. It runs before `asan`, so ASan does not instrument the checks.
- `fuzzer`: the `sancov-module` pass runs last, adding edge coverage counters, a PC table and traced comparisons. The link adds libFuzzer without its `main`; the program starts the engine itself (`Fuzz.run_engine`, [stdlib fuzzing](../tools/stdlib-fuzzing.md)). Combine as `address,fuzzer`.

Other UBSan checks are absent on purpose: signed overflow wraps in Jai, shifts and float-to-int are defined, division by zero traps, and an alignment check would flag deliberate unaligned reads.

Link side: a Clang from the LLVM install jaic was built against links with `-fsanitize=...`. Instrumentation and runtime must come from the same LLVM. Search: `JAIC_SANITIZER_CC`, then `bin/clang` of the LLVM install ([LLVM setup](../tools/llvm-setup.md#where-jaic-finds-llvm)), then `clang-<major>` or `clang` on `PATH`.

`Default_Allocator` calls `malloc`/`free`, so ASan sees every heap block. Allocators that carve a larger region (temporary allocator, `Flat_Pool`, `Pool`, `Bucket_Array`) are invisible to it.

### The sweep

```
tools/differential/jaic_sweep.py --sanitize address,undefined corpus modules regressions
```

Builds every `run` case sanitized, runs it from its source folder and applies the interpreter run's expectation; a sanitizer report in stderr fails the case. `--native` skips sanitizers and `--opt O2` picks the level. Leak detection is off because Jai programs leave memory to the OS. `jaic -coverage` reuses this plumbing ([coverage](../tools/coverage.md)).

## How to change it

- Another IR-level pass (say `tysan`): a field on `Sanitize`, its name in `Sanitize::parse` and `driver_flag`, its pass in `Sanitize::passes`, and a function attribute in `emit_module` if needed.
- A sweep report is a codegen or ABI bug, a stdlib bug or a test bug. Fix the cause and add a regression test (`tests/compiler/regressions/*.jai`, or `crates/jaic-cli/tests/native.rs`). Silence only a proven false positive, and say why here.
- A native executable cannot call into the compiler (libclang, `compiler_*`); such a test does the work in `#run`.
- Unsupported targets fail in `check_sanitizer_target`: cross builds, wasm, and Windows hosts. Static libraries and objects come out instrumented but unlinked; whoever links them needs a Clang of the same version.

## Configuration

`-sanitize address|undefined|fuzzer` or a comma list. `JAIC_SANITIZER_CC`, `LLVM_PREFIX`, `ASAN_OPTIONS`, `UBSAN_OPTIONS`. Sweep: `--sanitize`, `--native`, `--opt`, `--jobs` (ASan roughly doubles memory; keep jobs low on small machines).

## Dependencies

LLVM's `asan` and `bounds-checking` passes through `inkwell`, and compiler-rt runtimes of the same LLVM. CI runs the sweep under ASan and UBSan on Linux and macOS at `-O0` and `-O2` (`.github/workflows/ci.yml`).
