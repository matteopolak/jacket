# Sanitizers for native builds

## What it is

`jaic build -sanitize address,undefined` instruments a native build with LLVM's AddressSanitizer (ASan) and the UBSan checks that exist at the LLVM IR level, and links the matching runtime. The point is to catch native-codegen bugs that happen to run fine today (wrong struct layout, ABI marshalling that writes past a temporary, stack slots that are too small) along with ordinary memory errors in the stdlib and tests. `tools/jaic-sweep.py --sanitize` runs the test sets this way.

```
$ jaic build uaf.jai -sanitize address && ./uaf
==58208==ERROR: AddressSanitizer: heap-use-after-free on address 0x603000001010 ...
    #0 0x000102c27fa4 in get_s64_from_void_pointer.82 Print.jai:150
    ...
    #5 0x000102bff350 in main.7 uaf.jai:7
```

## How it works

Compile side (`Sanitize` in `crates/jaic-llvm/src/lib.rs`, applied in `emit_module`):

- `address`: every function *defined* in the module gets the `sanitize_address` attribute (ASan instruments only functions carrying it, as Clang does per definition), then the `asan` module pass runs after the optimization pipeline. It instruments loads and stores, gives every `Slot` alloca and every global a redzone, and registers globals with the runtime. Codegen units each get their own ASan constructor, which is fine.
- `undefined`: the `bounds-checking<rt-abort>` function pass (Clang's `-fsanitize=local-bounds`): an access outside an object whose size LLVM can see (an alloca, a global, a `malloc` result) calls `__ubsan_handle_local_out_of_bounds_abort`. At `-O0` every value lives in a stack slot and the pass cannot trace a pointer to its object, so `sroa` runs first.
- Both: the bounds checks run before `asan`, so ASan does not instrument the checks themselves.

Other UBSan checks are deliberately absent. Clang implements most of them in its front end, not as IR passes, and most do not map to Jai: signed overflow wraps in Jai (overflow checks are the language's own `#no_aoc` machinery, see [arithmetic overflow checks](../language/arithmetic-overflow-checks.md)), shifts and float-to-int conversions are defined (see [LLVM backend](llvm-backend.md#lowering-rules-lowerrs)), and division by zero already traps. An alignment check would flag every `<< cast(*u32) byte_pointer`, which Jai programs use freely, so it would only produce noise.

Link side (`link`): a sanitized build links with a Clang driver from the LLVM install jaic was built against and passes `-fsanitize=address,undefined`, which makes the driver add the runtime (`libclang_rt.asan_osx_dynamic.dylib` on macOS, the static `libclang_rt.asan.a` plus its dynamic-list flags on Linux). The instrumentation and the runtime must come from the same LLVM: Apple's `cc` ships an older runtime and GCC's `libasan` is a different implementation. The driver is found in this order: `JAIC_SANITIZER_CC`; `$LLVM_SYS_221_PREFIX/bin/clang` (at run time, then the value jaic was compiled with); `llvm-config-22 --bindir` / `llvm-config --bindir`; `clang-22` or `clang` on `PATH`.

Jai's `Default_Allocator` calls C `malloc`/`free`, so ASan sees every heap block, including use-after-free and double free. Allocators that carve blocks out of a larger region (the temporary allocator, `Flat_Pool`, `Pool`, `Bucket_Array`) are invisible to it: an overrun inside such a region is not reported.

### The sweep

`tools/jaic-sweep.py --sanitize address,undefined corpus stdlib modules` builds every `run` case with `jaic build -sanitize ...` into a scratch directory, runs the executable from the source's directory and applies the same expectation as the interpreter run (exact stdout and exit code for `corpus`, exit 0 otherwise). A case also fails when stderr holds a sanitizer report, whatever its exit code; the report's first line is shown as the failure. `--native` does the same without sanitizers, and `--opt O2` picks the optimization level (default: what the program's metaprogram asks for, usually `-O0`). `check` cases (`negative`) are unchanged.

The sweep sets these runtime options unless they are already in the environment:

- `ASAN_OPTIONS=detect_leaks=0:halt_on_error=1:abort_on_error=0:detect_stack_use_after_return=1:strict_string_checks=1:check_initialization_order=1`. Leaks are off because Jai programs routinely leave their memory to the OS at exit (LeakSanitizer is on by default on Linux).
- `UBSAN_OPTIONS=print_stacktrace=1:halt_on_error=1`.
- `ASAN_SYMBOLIZER_PATH`: `llvm-symbolizer` from the same LLVM, so reports show `file.jai:line`.

## How to change it

- Another IR-level sanitizer pass (for example `tysan`): add a field to `Sanitize`, its name to `Sanitize::parse` and `driver_flag`, and its pass to `Sanitize::passes`. Check the pass needs a function attribute (like `sanitize_address`) and add it in `emit_module`.
- A report from the sweep is a jaic codegen/ABI bug, a stdlib bug or a test bug: fix the cause and add a regression test (`tests/stdlib/*.jai`, or `crates/jaic-cli/tests/native.rs` for code that only misbehaves when compiled). Do not silence a report unless it is proven to be a false positive, and then document why here.
- Unsupported targets fail early in `check_sanitizer_target`: cross builds (`-os windows`, `-target ...`, wasm) and Windows hosts. Supporting Windows would mean driving `clang-cl /fsanitize=address` and its runtime DLLs; wasm has no ASan runtime.
- Static libraries and object files (`OutputType.STATIC_LIBRARY`, `.OBJECT_FILE`) come out instrumented but nothing is linked; whoever links them must pass `-fsanitize=...` to a Clang of the same LLVM version.

## Configuration

- CLI: `-sanitize address`, `-sanitize undefined`, `-sanitize address,undefined` (also `--sanitize`, repeatable). Combines with `-O0..-O3`; debug info stays on so reports carry source lines.
- `JAIC_SANITIZER_CC`: the Clang driver that links sanitized builds.
- `LLVM_SYS_221_PREFIX`: also used to find `clang` and `llvm-symbolizer`.
- Runtime: `ASAN_OPTIONS`, `UBSAN_OPTIONS`, `ASAN_SYMBOLIZER_PATH` (see the [ASan flags](https://github.com/google/sanitizers/wiki/AddressSanitizerFlags)).
- Sweep: `--sanitize LIST`, `--native`, `--opt O0|O1|O2|O3`, `--jobs` (ASan roughly doubles a program's memory and reserves terabytes of virtual shadow space; keep jobs low on small machines).

## Dependencies

LLVM 22's `asan` and `bounds-checking` passes (through `inkwell`), and the compiler-rt sanitizer runtimes of the same LLVM: included in Homebrew's `llvm`, `libclang-rt-22-dev` on apt.llvm.org. CI runs the sweep under ASan and UBSan on Linux; see [continuous integration](../tools/continuous-integration.md).
