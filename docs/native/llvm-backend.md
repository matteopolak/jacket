# jaic LLVM backend

## What it is

`crates/jaic-llvm` translates the `jaic::ir::Program` into LLVM IR, writes native object files, and links them with the system `cc`. The CLI exposes it as `jaic build <file.jai> [-o out]`; `jaic run` uses the interpreter and `jaic check` only type-checks.

## How it works

`emit_object(program, options, path)` in `lib.rs`:

1. Creates a target machine for the host or `Options::target` (for example `x86_64-pc-windows-gnu` for `-os windows`). The triple picks the `jaic::abi::Arch`.
2. `lower::lower_program` declares every lowered function, foreign symbol and global, fills in global initialisers, then defines function bodies.
3. Verifies the module, optionally runs the `default<On>` pipeline, and writes the object.

jaic sets no process-wide LLVM options (`LLVMParseCommandLineOptions`). Under LLVM 22 it passed `-unroll-add-parallel-reductions=false`: that release's runtime unroller, on by default for Apple CPUs, gave each unrolled copy of a reduction its own accumulator and, for a `sub` recurrence (`a -= b` in a loop of unknown length), combined them wrongly, so `-O2` printed different results from `-O0` and the interpreter ([llvm/llvm-project#201065](https://github.com/llvm/llvm-project/issues/201065), fixed in LLVM 23.1.0). The flag went with the move to LLVM 23. Corpus case `unrolled-sub-reduction` and the native test `optimized_sub_recurrence_matches_the_interpreter` still guard it. If an LLVM bug needs an option again, set it once before the first target machine is created (in `target_machine`, next to target registration), document the upstream issue, and remove it when jaic moves past the fixed release.

### Codegen units

`jaic build` calls `emit_objects`, which splits a large unoptimised program into codegen units, one LLVM context and module per thread. LLVM code generation is a large share of an `-O0` build, so this pays off on big programs like Focus.

- Units: `JAIC_CODEGEN_UNITS` if set, else one per 20,000 IR instructions (`INSTS_PER_UNIT`), capped at the core count. `-O1` and up, and `--emit-ir`, always use one unit so LLVM can inline across the whole program.
- Functions go largest first to the least loaded unit. Unit 0 also defines the globals.
- Each module defines its own functions and declares the rest (`lower::Shard`). Internal functions and globals become hidden external symbols (still named `name.index`), so the objects link together but a shared library exports nothing extra.
- Objects are `path`, `path.1.o`, `path.2.o`, .... The CLI links or archives them all, then deletes them. `-o x.o` stays a single module.

#### Splitting after the optimizer (`split.rs`)

An optimized build keeps one module through the optimizer, so inlining still sees the whole program. Machine code generation (instruction selection, register allocation) is about half of an `-O2` build and works one function at a time, so `split::emit` runs it in parallel once the passes are done:

1. `units_for`: one unit per 10,000 LLVM instructions (`INSTS_PER_UNIT`), capped at the core count and at 4 (`MAX_UNITS`). Below 2 units nothing changes.
2. Internal and private definitions become hidden external symbols (unnamed ones are named `jaic.local.N`), and function definitions are assigned to units largest first.
3. The module is written to bitcode once. Each extra unit's thread parses it into its own `Context`, turns other units' function bodies into declarations, and makes the data declarations. Unit 0 is the original module cut down in place; it defines the data and keeps the appending globals such as `llvm.used`.
4. Each thread writes its object with its own `TargetMachine`. The objects get the same names as the codegen units above.

`strip_body` deletes whole blocks after cutting every use into them; it must not erase instructions one at a time. In LLVM 22 an erased instruction's debug records move to the next instruction, and from a block's last instruction into a context-wide table of trailing records keyed by the block's address. Deleting the block leaves the entry behind. A block that codegen later allocates at the same address then picks up another function's variables, and `DwarfDebug::finalizeModuleInfo` crashes on a variable it never gave a DIE. That happened in about one build in four, depending on heap layout, and never in `llc` on the same bitcode. Under LLVM 23.1.2, 60 `-O2` builds of jaifmt (half with `JAIC_SPLIT_UNITS=4`) and the `-O2` asm tests ran clean; keep deleting whole blocks regardless.

Parsing and cutting down are serialized under a mutex. Every unit briefly holds a whole copy of the module, so running them all at once raised peak memory by about one module per unit. With the mutex and the cap, an `-O2` build of Jails or jaison takes about a fifth less wall time for about 12% more peak RSS. More than 4 units gave no further speedup, because the optimizer, which stays serial, then dominates.

It applies when `emit_objects` would use one unit at `-O1` and up, without `--emit-ir`, a sanitizer or `JAIC_CODEGEN_UNITS`. `-o x.o` (`emit_object`) never splits.

`Target::initialize_all` runs once per process behind a `Once`; calling it again from several threads at once crashed LLVM's target registry.

### Lowering rules (`lower.rs`)

- Every defined function has `"frame-pointer"="non-leaf"`, as clang has on Apple and AArch64 targets. Each function that calls another keeps a frame record, so frame-pointer stack walks see jaic frames. Those walks are macOS libc `backtrace`, which Debug's `backtrace` uses, and sampling profilers. Before this, `Debug.backtrace()` found no frames in a native build (`backtrace_sees_compiled_callers`).
- Each IR `Val` is an LLVM SSA value. Blocks are emitted in reverse post-order from the entry, so definitions precede uses; unreachable blocks are skipped. No phis are needed because mutable state lives in slots.
- `Slot`s become allocas in a dedicated first `allocas` block that branches to IR block 0. ABI marshalling temporaries live there too.
- Pointers are opaque; `PtrAdd` is an `i8` GEP. Loads and stores carry natural alignment.
- Integer semantics match the interpreter: division by zero traps (`llvm.trap`), `INT_MIN / -1` wraps, shifts of `>= bits` give 0 (`AShr` clamps), float-to-int conversions saturate, `FNe` is unordered-not-equal.
- Globals are packed structs of byte runs and pointer relocations, with the IR alignment and `read_only` flag. `#program_export` names stay external; everything else is internal and suffixed with its index.
- Foreign functions and variables are external declarations, de-duplicated by symbol. Every call is an indirect call with a function type built from the call's `Sig`, so one symbol can be called with several signatures.
- `Conv::Jai` functions map parameters and scalar results one to one; several results become a struct return.
- Intrinsics: `Memcpy` is `memmove`; `Memcmp` calls libc `memcmp` and normalises to -1/0/1; `CompilerWrite` calls `write(1|2, ...)`; `CompareAndSwap` is a seq_cst `cmpxchg` of the operand's width; `IsCompileTime` is 0; `CycleCounter` reads `cntvct_el0` on AArch64 and `rdtsc` on x86-64; `Fma` is `llvm.fma.f64`. `Loc` markers set the [debug location](debug-info.md).

### C ABI

`Conv::C` signatures with `Sig::c_abi` use the real calling convention for by-value aggregates; the rules are in [C ABI](c-abi.md). The IR passes aggregates by pointer, so a call site copies into a scratch temp, loads the pieces and passes them as separate LLVM arguments. Returned pieces are stored to a temp and copied through the IR out-pointer, which is the last IR parameter and is dropped from the LLVM signature.

Variadic calls use a vararg function type whose parameters are only the declared ones (`Sig::c_fixed`; the call site appends the rest), so LLVM applies the platform's variadic convention.

Definitions with C signatures (`#c_call` callbacks that C calls with structs) do the reverse in `bind_params`: register pieces are stored into an entry-block temp whose address stands in for the IR parameter, and `Ret` loads the return pieces from the out-pointer temp (`FnState::reg_ret`). `sret` and `byval` pointers are used directly.

## How to change it

- New IR instruction or intrinsic: `Backend::inst` or `Backend::intrinsic` in `lower.rs`, with semantics identical to `interp/mod.rs`.
- New target architecture: an `Arch` variant and classification in `abi.rs`, plus any inline-asm intrinsics in `lower.rs`.
- Debug info is in `debuginfo.rs`; `lower.rs` only calls its hooks (`begin_function`, `declare_vars`, `enter_block`/`leave_block`, `loc`, `finish_entry`, `Backend::set`, `describe_globals`).
- Anything module-level (a global, a constructor list) must be emitted once, in unit 0, and declared in the others. Use `Backend::internal_linkage` for new internal symbols so other units can reference them.
- Small test programs use one unit; check splitting with `JAIC_CODEGEN_UNITS=4 cargo test -p jaic-cli --test native`.
- Small optimized programs stay under `INSTS_PER_UNIT`; force the post-optimizer split with `JAIC_SPLIT_UNITS=4 cargo test -p jaic-cli --test native`. A declaration made from a definition must lose its body, personality and `!dbg` attachment (`strip_body`), or the verifier rejects the module.
- Windows (`Arch::Win64`): `#program_export` definitions are `dllexport` and `CompilerWrite` calls `_write`. See [Windows](windows.md).
- WebAssembly (`Arch::Wasm64`): foreign procedures become wasm imports, `#program_export`s get `wasm-export-name`, every function is `no-builtins`, and a weak `__multi3` is emitted. See [wasm target](wasm-target.md).

## Configuration

- `JAIC_CODEGEN_UNITS=N` forces the unit count; `1` turns splitting off. (before and after the optimizer).
- `JAIC_SPLIT_UNITS=N` forces the post-optimizer unit count of an optimized build (for tests); `1` turns that split off.
- `INSTS_PER_UNIT` and `MAX_UNITS` in `split.rs`.
- `jaic_llvm::Options { opt_level, target, emit_ir, debug_info, sanitize }`, set from the CLI flags `-O0..-O3`, `--emit-ir file.ll`, `--no-debug-info`, `-sanitize` ([sanitizers](sanitizers.md)), `-os`, `-target triple`.
- Building the crate needs `LLVM_SYS_231_PREFIX` pointing at LLVM 23 (for example `/opt/homebrew/opt/llvm`); see [LLVM setup](../tools/llvm-setup.md).

## Dependencies

`inkwell` (LLVM 23), the system `cc` for linking, and `jaic` for the IR. Linking rules: [native linking](native-linking.md).
