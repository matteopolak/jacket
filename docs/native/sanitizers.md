# Sanitizers for native builds

## What it is

`jaic -sanitize address,undefined` instruments a native build with LLVM's AddressSanitizer (ASan) and the UBSan checks that exist at the LLVM IR level, and links the matching runtime. It catches native-codegen bugs that happen to run fine (wrong struct layout, ABI marshalling that writes past a temporary, stack slots that are too small) and ordinary memory errors in the stdlib and tests. `tools/jaic-sweep.py --sanitize` runs the test sets this way.

```
$ jaic uaf.jai -sanitize address && ./uaf
==58208==ERROR: AddressSanitizer: heap-use-after-free on address 0x603000001010 ...
    #5 0x000102bff350 in main.7 uaf.jai:7
```

## How it works

Compile side (`Sanitize` in `crates/jaic-llvm/src/lib.rs`, applied in `emit_module`):

- `address`: every defined function gets the `sanitize_address` attribute, then the `asan` module pass runs after the optimization pipeline. It instruments loads and stores, adds redzones to every `Slot` alloca and global, and registers globals. A sanitized build is always one codegen unit (on Intel macOS only the first of several sanitized objects reached the debug map, so reports showed `main.8+0xc18` instead of `file.jai:9`) and at `-O0` writes a `.dSYM` ([debug info](debug-info.md)).
- `undefined`: the `bounds-checking<rt-abort>` pass (Clang's `-fsanitize=local-bounds`): an access outside an object whose size LLVM can see calls `__ubsan_handle_local_out_of_bounds_abort`. At `-O0` `sroa` runs first so the pass can trace pointers to their objects.
- Bounds checks run before `asan`, so ASan does not instrument the checks.

Other UBSan checks are deliberately absent: signed overflow wraps in Jai ([arithmetic overflow checks](../language/numbers.md)), shifts and float-to-int conversions are defined, division by zero already traps, and an alignment check would flag deliberate unaligned reads like `<< cast(*u32) byte_pointer`.

Link side (`link`): a Clang driver from the LLVM install jaic was built against links with `-fsanitize=address,undefined`, which adds the runtime. Instrumentation and runtime must come from the same LLVM (Apple's `cc` ships an older runtime; GCC's `libasan` is different). Driver search: `JAIC_SANITIZER_CC`; `$LLVM_SYS_231_PREFIX/bin/clang`; `llvm-config-23 --bindir` / `llvm-config --bindir`; `clang-23` or `clang` on `PATH`.

`Default_Allocator` calls C `malloc`/`free`, so ASan sees every heap block. Allocators that carve blocks out of a larger region (temporary allocator, `Flat_Pool`, `Pool`, `Bucket_Array`) are invisible: an overrun inside such a region is not reported.

### The sweep

`tools/jaic-sweep.py --sanitize address,undefined corpus modules regressions` builds every `run` case with `-sanitize`, runs it from the source's directory and applies the interpreter run's expectation. A case also fails when stderr holds a sanitizer report. `--native` does the same without sanitizers; `--opt O2` picks the level. `examples` and `upstream` are not expected to pass in full (some entry points do not fit a native build; two The_Way_to_Jai examples read freed memory on purpose). The sweep sets `ASAN_OPTIONS=detect_leaks=0:halt_on_error=1:detect_stack_use_after_return=1:strict_string_checks=1:check_initialization_order=1` (leaks off because Jai programs leave memory to the OS), `UBSAN_OPTIONS=print_stacktrace=1:halt_on_error=1`, `ASAN_SYMBOLIZER_PATH` (same LLVM's `llvm-symbolizer`), and on macOS `DYLD_FALLBACK_LIBRARY_PATH` with Homebrew's `lib` (ASan intercepts `dlopen`, so libraries loading others through `@loader_path` break otherwise).

## How to change it

- **Another IR-level sanitizer pass** (e.g. `tysan`): add a field to `Sanitize`, its name to `Sanitize::parse` and `driver_flag`, its pass to `Sanitize::passes`, and a function attribute in `emit_module` if the pass needs one.
- A report from the sweep is a jaic codegen/ABI bug, a stdlib bug or a test bug: fix the cause and add a regression test (`tests/compiler/regressions/*.jai`, or `crates/jaic-cli/tests/native.rs` for code that only misbehaves compiled). Do not silence a report unless it is a proven false positive, and then document why here.
- An executable cannot call into the compiler (libclang, `compiler_*`): such a test does that work in `#run` and checks the result there. Programs without `main` are listed as not run natively.
- Unsupported targets fail in `check_sanitizer_target`: cross builds, wasm (no ASan runtime) and Windows hosts (would need `clang-cl /fsanitize=address` and its runtime DLLs). Static libraries and object files come out instrumented but unlinked; whoever links them must pass `-fsanitize=...` to a Clang of the same version.

## Configuration

- CLI: `-sanitize address`, `undefined` or `address,undefined` (also `--sanitize`, repeatable); combines with `-O0..-O3`, debug info stays on.
- `JAIC_SANITIZER_CC`, `LLVM_SYS_231_PREFIX` (also finds `clang` and `llvm-symbolizer`); runtime `ASAN_OPTIONS`, `UBSAN_OPTIONS`.
- Sweep: `--sanitize LIST`, `--native`, `--opt`, `--jobs` (ASan roughly doubles memory and reserves terabytes of virtual shadow space; keep jobs low on small machines).

## Dependencies

LLVM 23's `asan` and `bounds-checking` passes (through `inkwell`) and the compiler-rt sanitizer runtimes of the same LLVM (in Homebrew's `llvm`, `libclang-rt-23-dev` on apt.llvm.org). CI runs the sweep under ASan and UBSan on Linux and macOS at `-O0` and `-O2` with `--headless` (see `.github/workflows/ci.yml`).
