# WebAssembly target (wasm64)

## What it is

`jaic build` can compile a program to a WebAssembly module: LLVM generates wasm64 (Memory64) object code and `wasm-ld` links it. With `-os wasm`, the module is a WASI preview 1 command that runs under node, wasmtime, or a browser with a WASI shim. With a bare `wasm64-unknown-unknown` triple, the host supplies the imports and calls the exports directly.

This is a separate thing from the [browser compiler](../browser/playground.md), which is the compiler itself built to wasm32 by Cargo; it *interprets* programs whose target is `OS == .WASM`. `jaifmt.wasm` ([jaifmt](../tools/jaifmt.md#webassembly-build-jaifmtwasm)) is the main user of this target.

```sh
jaic build hello.jai -os wasm -o hello.wasm            # WASI command; Wasi_Runtime is added
node --no-warnings tools/wasi_run.mjs hello.wasm a b  # stdin/stdout/env/args, exits with main's status
jaic build lib.jai -target wasm64-unknown-unknown -O2 -o lib.wasm   # no runtime: your host's imports
```

From a metaprogram, the same options as Jai:

```jai
options.os_target = .WASM;
options.cpu_target = .CUSTOM;
options.backend = .LLVM;
options.llvm_options.target_system_triple = "wasm64-unknown-wasi";   // "wasi": add Wasi_Runtime
options.llvm_options.target_system_features = "+bulk-memory,+sign-ext";
options.additional_linker_arguments = .["-z", "stack-size=1048576"];  // passed to wasm-ld
```

`tests/native/wasm/build_wasi.jai` is a complete example. The output is `<output_executable_name>.wasm`.

## How it works

**Target selection.** `-os wasm` maps to the triple `wasm64-unknown-wasi`; `-target <triple>` is taken as given. A workspace with `os_target = .WASM` uses `llvm_options.target_system_triple`, or `wasm64-unknown-unknown` when it is empty, and `target_system_cpu` / `target_system_features` (the `llvm_target_system_*` options forwarded by `stdlib/Compiler/options.jai`). `cpu_target = .CUSTOM` is accepted for wasm. wasm32 triples are refused: Jai's `*void`, `s64` counts and `Type_Info` layouts assume 8-byte pointers. `+bulk-memory` is always added (so `memcpy`/`memset` lower to `memory.copy`/`memory.fill`), the relocation model is static, and the object files go through `wasm-ld` (`crates/jaic-llvm/src/wasm.rs`, `link_wasm`):

```
wasm-ld -mwasm64 <objects> -o out.wasm [--no-entry] --stack-first -z stack-size=8388608 [--strip-debug] <libraries> <additional_linker_arguments>
```

`--no-entry` is passed unless the program defines an exported `_start`. Libraries named by `#library` resolve to a `.a`/`.o` next to the source; system libraries are not linked (they are import modules, below).

**Imports and exports** (`crates/jaic-llvm/src/lower.rs`):

- A `#foreign` procedure or `#elsewhere` global becomes a wasm import. The import module is the library's name (`#system_library "host_graphics"` imports from `host_graphics`); no library, or `libc`/`c`/`crt`/`msvcrt`/`m`/`libm`, imports from `env`. The import name is the foreign symbol.
- If the program itself `#program_export`s a procedure with the foreign symbol's name, the reference binds to that definition instead of importing. This is how Wasi_Runtime supplies `malloc` or `write` to the rest of the stdlib.
- `#program_export` procedures get a `wasm-export-name` and are exported, along with `memory`. Wasi_Runtime's are the exception (`is_wasi_library`): only its `_start` is exported, so `wasm-ld` drops the runtime functions a program never calls, along with the WASI imports only they use. A hello world imports four WASI calls.
- `#intrinsic "llvm.<name>"` on a bodiless procedure calls the LLVM intrinsic of that name (for example `llvm.wasm.memory.grow.i64`, `llvm.ctpop.i64`, `llvm.trap`); this works on every LLVM target ([intrinsics](../language/intrinsics.md)).
- Every function has `no-builtins`: otherwise LLVM turns the runtime's own loops into calls to `strlen`/`memset`, which recurse into themselves.
- A weak `__multi3` (128-bit multiply) is emitted, because LLVM uses it for 64-bit division by constants and no compiler-rt is linked. Return addresses are null, the cycle counter reads 0 and `pause` is a no-op.

**ABI** (`crates/jaic/src/abi.rs`, `Arch::Wasm64`): a struct whose single scalar field is an `s64`, pointer, `float64` or `float32` is passed directly; every other aggregate goes by pointer (byval arguments, sret results). Clang passes single small-integer structs as `i32`, so a C library compiled with Clang and taking such a struct by value would disagree.

**Wasi_Runtime** (`stdlib/Extensions/Wasi_Runtime/`), written in Jai, implements what the stdlib's `OS == .WASM` code calls, on top of WASI preview 1:

| Export | From |
| --- | --- |
| `_start` | reads `args_sizes_get`/`args_get` into a `**u8` table, calls Runtime_Support's `main`, then `proc_exit` |
| `write`, `read`, `isatty`, `wasm_write_string`, `wasm_debug_break` | `fd_write`, `fd_read` |
| `getenv` | `environ_sizes_get`/`environ_get`, read once |
| `clock_gettime` | `clock_time_get` |
| `exit`, `_exit`, `abort` | `proc_exit`, `llvm.trap` |
| `malloc`, `calloc`, `realloc`, `free` | a power-of-two size-class heap above `__heap_base`, growing memory with `memory.grow` |
| `memmove`, `memcpy`, `memset`, `memcmp`, `strlen` | plain loops (LLVM lowers the copies to `memory.copy`) |
| `open`, `close`, `lseek`, `stat`/`fstat`/`lstat`, `access`, `mkdir`, `rmdir`, `unlink`, `remove`, `rename`, `link`, `realpath`, `getcwd`, `chdir`, `opendir`/`readdir`/`closedir`, `fopen` and the `f*` stream calls, `chmod`, `sysconf`, `strerror` | `files.jai`: `path_open`, `fd_readdir`, `path_filestat_get` and the other descriptor and path calls, below |
| `fork`, `execvp`, `pipe`, `dup2`, `socketpair` (`ENOSYS`), `waitpid` (`ECHILD`), `kill` (`ESRCH`), `fcntl`, `poll` (no descriptors: a sleep), `usleep` | `module.jai`: a sandbox with no processes, so Process and friends fail cleanly |
| libm: `round`, `fma`, `floor`, `ceil`, `trunc`, `sqrt`, `fabs`, `copysign`, `fmin`, `fmax`, `fmod` (exact); `sin`, `cos`, `tan`, `exp`, `exp2`, `log`, `log2`, `log10`, `pow`, `atan`, `atan2`, `asin`, `acos` (within an ulp); each with its `f` form | `math.jai`, `elementary.jai` |
| `__addtf3`, `__subtf3`, `__multf3`, `__divtf3`, `__negtf2`, the `__*tf2` comparisons, `__extend{s,d}ftf2`, `__trunctf{s,d}f2`, `__fix*tf*i`, `__float*itf` | `quad.jai`: compiler-rt's binary128 helpers, for Long_Double, correctly rounded |

WASI preview 1 is defined for 32-bit memories, so its pointers are `u32` offsets into our 64-bit memory; that works while the stack and heap stay under 4 GiB. A WASI module's only imports are `wasi_snapshot_preview1` functions.

**Files.** WASI has no global file system: the host pre-opens directories (descriptors 3 and up, each with a name and the rights files opened under it may have), and every path call names one of them plus a relative path. `files.jai` lists the pre-opens once, makes each C path absolute against its working directory, removes `.` and `..` lexically, and hands it to the pre-open whose name is its longest prefix; a path no pre-open covers fails with `ENOENT`. The working directory starts as `$PWD` when a pre-open covers it, else `/`, and `chdir` only changes that string. `realpath` resolves symlinks one component at a time with `path_readlink`. Results use the Linux x86-64 `struct stat`/`struct dirent` layouts and errno numbers the stdlib's `OS == .WASM` code already shares with the sandbox host. WASI has no modes or owners: `stat` reports 0644 files and 0755 directories, and `chmod` only checks the path exists. `FILE *` streams are unbuffered descriptors.

**Math.** `round` and `fma` are what LLVM calls for `roundsd`/`vfmadd` emulation and the `#intrinsic` procedures of those names; the rest are for programs that declare them `#foreign` from libm. `fma`, `fmod` and the binary128 helpers round once from a wide exact accumulator. The transcendental functions work in float64 pairs (about 106 bits) and round once; `sin`, `cos` and `tan` reduce with 2/pi to 1344 bits, so huge arguments are right too. The float32 forms compute in float64.

jaic adds `#import "Extensions/Wasi_Runtime"` to the first workspace when the target triple has a component starting with `wasi` (`jaic::build::wants_wasi_runtime`). With a bare `wasm64-unknown-unknown` nothing is added, as in Jai: Runtime_Support's own needs (`malloc`, `free`, `memmove`, `memcmp`, `wasm_write_string`, `wasm_debug_break`) are `env` imports the host provides, and `tests/native/wasm/exports.mjs` shows a host that does.

**Why the sandbox's `OS == .WASM` code is unaffected.** The stdlib's `OS == .WASM` branches call ordinary C names (`malloc`, `write`, `clock_gettime`, `getenv`, ...). Under the interpreter, in the browser engine and in the compile-time code of a wasm build (`make_host` gives `.WASM` workspaces the `SandboxHost`), the sandbox host answers those `#foreign` calls. In a native wasm build the same calls bind to Wasi_Runtime's exports. No stdlib code distinguishes the two, so `tools/check_playground_stdlib.mjs` sees no change.

**Running.** `tools/wasi_run.mjs` runs a command with node's WASI, this process's stdio and environment (plus `PWD` set to its working directory), with the host's `/` pre-opened as `/`, and exits with the module's status (134 and `wasm trap: ...` on a trap, 127 and `wasm link error: missing imports a.b, c.d` naming every import the module lacks, checked before it starts).

## How to change it

- **A new libc entry point** used by stdlib code under `OS == .WASM`: add a `#program_export` with the C name to Wasi_Runtime and keep the `SandboxHost` (`crates/jaic/src/interp/`) answering it too. A symbol neither defines becomes an `env` import and the WASI module fails to instantiate (`wasm link error`).
- **A new WASI call**: declare it `#foreign wasi "<name>"` with `u32` offsets, as the existing ones are.
- **Linker flags**: `link_wasm` in `crates/jaic-llvm/src/wasm.rs`; per-program flags go in `additional_linker_arguments`.
- **ABI changes**: `classify_arg`/`wasm_single_scalar` in `crates/jaic/src/abi.rs` and the test next to them.
- **wasm-ld discovery**: `find_llvm_tool` in `wasm.rs`. When LLVM's major version changes, update the versioned names there with the rest ([LLVM setup](../tools/llvm-setup.md)).
- **A libm or compiler-rt function** LLVM starts calling: a `#program_export` under its name in `math.jai`/`elementary.jai`/`quad.jai`. `fp128` arguments arrive as two `u64`s (low first) and results go through a pointer passed first. Check accuracy against the host's libm with a probe that reads bit patterns from stdin.
- Gotchas: a libcall Wasi_Runtime does not define (`sinh`, `cbrt`, `__powitf2`...) becomes an `env` import and the module fails to instantiate. Threads, sockets and processes are not available: WASI preview 1 has none. Under a runtime that pre-opens nothing (wasmtime without `--dir`), every path call fails with `ENOENT`. Wasi_Runtime's `#program_export`s are not wasm exports (`is_wasi_library` in `lower.rs` checks the source directory's name), so a host cannot call `malloc` on a WASI command.
- **Tests**: `crates/jaic-cli/tests/wasm_target/` (part of the `native` test binary) builds hello, `tests/native/wasm/program.jai` at `-O0` and `-O2`, the metaprogram build, the bare exports module, jaifmt.wasm and every corpus case, and runs them under node. `python3 tools/jaic-diff.py --backends interp,wasm-native corpus gen:1:200` compares the interpreter with WASI builds ([differential testing](../tools/differential-testing.md)).

## Configuration

| Setting | Effect |
| --- | --- |
| `-os wasm` | triple `wasm64-unknown-wasi`, Wasi_Runtime added |
| `-target wasm64-unknown-unknown` | no runtime; the host supplies imports |
| `-O2`, `--no-debug-info` | as for native builds; without debug info the module is linked with `--strip-debug` |
| `JAIC_WASM_LD` | path of `wasm-ld`; otherwise `$LLVM_SYS_231_PREFIX/bin`, the prefix jaic was built with, `llvm-config --bindir`, Homebrew's `lld`/`lld@23` kegs, `/usr/lib/llvm-23/bin`, then `PATH` (`wasm-ld-23`, `wasm-ld`) |
| `JAIC_REQUIRE_WASM_TESTS=1` | the wasm tests fail instead of skipping when node or wasm-ld is missing (CI sets it) |

Install LLD with `brew install lld` (or `lld@23`) or Debian's `lld-23`. A missing linker gives `wasm-ld not found: install LLD (...) or set JAIC_WASM_LD`.

Runtimes need Memory64:

| Runtime | Memory64 |
| --- | --- |
| Node.js | 24 (no flag) |
| Chrome / Edge | 133 |
| Firefox | 134 |
| Safari | not in a release; Safari Technology Preview 251 and later |
| Deno | 2.2 |
| Wasmtime | 30 |

## Dependencies

LLVM 23 with the WebAssembly target (Homebrew and apt.llvm.org builds include it), LLD's `wasm-ld`, and node 24 to run the tests and `tools/wasi_run.mjs`. Internal: `crates/jaic-llvm` (`lower.rs`, `wasm.rs`, `lib.rs`), `crates/jaic/src/abi.rs`, `crates/jaic/src/build.rs` (options, `wants_wasi_runtime`), `crates/jaic-cli/src/main.rs` (CLI target, linking), `stdlib/Extensions/Wasi_Runtime` (`module.jai`, `files.jai`, `math.jai`, `elementary.jai`, `quad.jai`), and the `Runtime_Support` entry points it calls.
