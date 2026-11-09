# WebAssembly target (wasm64)

`jaic` compiles a program to a WebAssembly module: LLVM generates wasm64 (Memory64) code and `wasm-ld` links it. With `-os wasm` the module is a WASI preview 1 command that runs under node, wasmtime or a browser with a WASI shim. With a bare `wasm64-unknown-unknown` triple the host supplies imports and calls exports directly.

This is separate from the [browser compiler](../browser/playground.md), which is the compiler itself built to wasm32 and interprets programs whose target is `OS == .WASM`.

```sh
jacket build hello.jai -os wasm -exe hello.wasm            # WASI command; Wasi_Runtime is added
node --no-warnings tools/wasi_run.ts hello.wasm a b       # stdio/env/args, exits with main's status
jacket build lib.jai -target wasm64-unknown-unknown -O2 -exe lib.wasm   # no runtime
```

From a metaprogram:

```jai
options.os_target = .WASM;
options.cpu_target = .CUSTOM;
options.llvm_options.target_system_triple = "wasm64-unknown-wasi";   // "wasi": add Wasi_Runtime
options.llvm_options.target_system_features = "+bulk-memory,+sign-ext";
options.additional_linker_arguments = .["-z", "stack-size=1048576"];  // passed to wasm-ld
```

`tests/compiler/native/wasm/build_wasi.jai` is a complete example.

## How it works

- **Target.** `-os wasm` is `wasm64-unknown-wasi`; `-target` is taken as given. wasm32 triples are refused: Jai assumes 8-byte pointers. `+bulk-memory` is always on. `link_wasm` (`crates/jaic-native/src/wasm.rs`) runs `wasm-ld -mwasm64` with `--stack-first`, an 8 MiB stack, `--no-entry` unless the program exports `_start`, and `--strip-debug` with `-no_debug_info`.
- **Machine level.** At `-O0` LLVM gives every IR value a local, and a function that inlines a lot passes the engines' limit of 50000 locals. `wasm_machine_level` (`machine.rs`) runs the target machine one level up unless the program set `machine_code_optimization_setting`. If a function still passes the limit, make its inlined code smaller.
- **Imports and exports** (`lower.rs`). A `#foreign` procedure or `#elsewhere` global becomes an import from the library's name, or `env` for none or `libc`/`c`/`m`/`msvcrt`; if the program `#program_export`s that symbol, the reference binds to it. `#program_export` procedures are exported, except Wasi_Runtime's: only `_start` is, so `wasm-ld` drops unused runtime and its WASI imports. `#intrinsic "llvm.<name>"` calls that LLVM intrinsic.
- Every function has `no-builtins`, or LLVM turns the runtime's own loops into recursive calls to `strlen`/`memset`. A weak `__multi3` covers 64-bit division by constants.
- **ABI** (`Arch::Wasm64`): a struct with a single `s64`, pointer or float field is passed directly; every other aggregate by pointer. A Clang-built C library taking a small-integer struct by value would disagree.

### Wasi_Runtime

`stdlib/Extensions/Wasi_Runtime/` (written in Jai) implements what stdlib `OS == .WASM` code calls, on WASI preview 1: `_start`, `write`/`read`/`getenv`/`clock_gettime`/`exit`, a size-class heap for `malloc`, memory and string loops, files (`files.jai`), sockets (`sockets.jai`), process stubs that fail with `ENOSYS`/`ECHILD`, green threads (`threads.jai`), libm and binary128 helpers. jaic adds it when the triple has a component starting with `wasi`; with `wasm64-unknown-unknown` nothing is added and the host provides `env` imports.

- **Files.** WASI has no global file system; the host pre-opens directories. `files.jai` makes each path absolute against its working directory, normalizes `.`/`..`, and hands it to the pre-open with the longest matching prefix, else `ENOENT`. Stat and dirent use Linux x86-64 layouts. Under a runtime that pre-opens nothing, every path call fails.
- **Sockets.** Only sockets the host pre-opened work: `accept`, `recv`, `send`, `shutdown`. Everything else returns -1 with `ENOSYS`, so a program that builds a socket compiles and fails at run time. In `jaic -interp -os wasm`, `JAIC_TCPLISTEN=host:port,...` listens before the program starts (descriptors 3, 4, ... and `LISTEN_FDS`), like wasmtime's `--tcplisten` (`interp/sandbox/sockets.rs`).
- **Math.** `fma`, `fmod` and binary128 helpers round once from a wide exact accumulator; transcendentals work in float64 pairs.
- The interpreter and browser answer the same C names through `SandboxHost`, so no stdlib code tells them apart.
- **Running.** `tools/wasi_run.ts` runs a command with node's WASI. It exits with the module's status (134 on a trap, 127 naming missing imports). `--unknown-imports-trap` lets a WebGPU program run and find no WebGPU.

### Green threads

WASI preview 1 has no threads and wasm cannot switch stacks, so Thread-module programs get green threads: Wasi_Runtime runs them one at a time, and the backend rewrites procedures that may block so a thread can unwind to the scheduler and resume later. This applies to native WASI builds only; the interpreter and browser schedule threads themselves.

- **Scheduling** (`threads.jai`): `pthread_create` adds a thread with a 1 MiB frame stack that first runs when the creator blocks. Mutexes are recursive; condition waits block on the condition's address; `nanosleep`, `usleep` and an empty `poll` are timed waits. `_start` runs `main` as the first thread, round robin. If every thread waits forever, the last to block wakes with `EDEADLK`.
- **Unwinding** (`crates/jaic-llvm/src/green.rs`, the Asyncify technique on jaic's IR). A blocking call ends in `block_running`, which calls `green_suspend` and sets the state to unwinding; each rewritten procedure on the way back stores which call it was in and returns. To resume, the scheduler sets rewinding and calls the entry again, and each procedure takes its frame back and jumps to that call. `green::instrument` runs before lowering, only when the program has the `jaic_wasi_*` exports and reaches `pthread_create`. Slots stop being allocas and live in a heap frame (`[resume point][slots][stored values]`) that traps on overflow.

Debug info keeps lines and scopes for rewritten procedures but drops their variables.

## How to change it

- New libc entry point used under `OS == .WASM`: add a `#program_export` with the C name to Wasi_Runtime and keep `SandboxHost` answering it. A symbol neither defines becomes an `env` import and the module fails to instantiate (`wasm link error`); so does a libcall Wasi_Runtime lacks (`sinh`, `cbrt`, ...). `fp128` arguments arrive as two `u64`s, low first, and results go through a pointer passed first.
- New WASI call: declare it `#foreign wasi "<name>"` with `u32` offsets (preview 1 pointers are 32-bit, fine while stack and heap stay under 4 GiB).
- Linker flags: `link_wasm`, or `additional_linker_arguments` per program. `wasm-ld` discovery: `find_llvm_tool`. ABI: `classify_arg`/`wasm_single_scalar`.
- New blocking call in Wasi_Runtime: block with `block_running(object, deadline)`, wake with `wake_one`/`wake_all`, and keep a path for `!green.enabled`. The scheduler and `_start` are not rewritten, so never block below an unrewritten frame.
- `Green_Control`'s layout is shared between `green.rs` constants (`TOP`, `LIMIT`, `ENABLED`) and `threads.jai`. Stack sizes: `MAIN_FRAME_BYTES` (8 MiB), `THREAD_FRAME_BYTES`.
- Gotchas: nothing preempts, so a busy wait never yields. A `#c_call` procedure that may block and takes a struct by value gets a pointer into a stack that is gone after a resume. Rewritten procedures keep locals in memory and are slower.
- Tests: `crates/jaic-cli/tests/wasm_target/` builds hello, `program.jai` at `-O0`/`-O2`, the metaprogram build, jaifmt.wasm and every corpus case and runs them under node. `PYTHONPATH=tools python3 tools/differential/jaic_diff.py --backends interp,wasm-native corpus gen:1:200` compares the interpreter with WASI builds ([testing](../tools/testing.md)).

## Configuration

| Setting | Effect |
| --- | --- |
| `-os wasm` | `wasm64-unknown-wasi`, Wasi_Runtime added |
| `-target wasm64-unknown-unknown` | no runtime |
| `-no_debug_info` | `--strip-debug` |
| `JAIC_WASM_LD` | path of `wasm-ld`; else the LLVM install's `bin/` ([LLVM setup](../tools/llvm-setup.md#where-jaic-finds-llvm)), Homebrew `lld`, then `PATH` |
| `JAIC_REQUIRE_TOOLCHAIN=node,wasm-ld` | wasm tests fail instead of skipping (CI sets it) |

Install LLD with `brew install lld` or Debian's `lld-23`. Runtimes need Memory64: Node.js 24, Chrome/Edge 133, Firefox 134, Deno 2.2, Wasmtime 30, Safari Technology Preview 251 or later.

## Dependencies

LLVM with the WebAssembly target, `wasm-ld`, node 24 for tests. Internal: `crates/jaic-llvm` (`lower.rs`, `wasm.rs`, `green.rs`), `crates/jaic/src/abi.rs`, `stdlib/Extensions/Wasi_Runtime`.
