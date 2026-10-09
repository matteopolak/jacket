# jaic LLVM backend

## What it is

`crates/jaic-llvm` translates the `jaic::ir::Program` into LLVM IR, writes native object files, and links them with the system `cc`. The CLI exposes it as `jaic build <file.jai> [-o out]`; `jaic run` uses the interpreter and `jaic check` only type-checks.

## How it works

`emit_object(program, options, path)` in `lib.rs`:

1. Creates a target machine for the host or `Options::target` (for example `x86_64-pc-windows-gnu` for `-os windows`). The triple picks the `jaic::abi::Arch`.
   The CPU is the oldest one the triple runs on (`baseline_cpu`: `x86-64`, `apple-m1` for arm64 macOS, else `generic`), never the build machine's: release archives (jaifmt) are built on one CI runner and run on others, and 0.4.1's jaifmt, built for a runner with AVX-512, died with SIGILL on runners without it. `Build_Options.llvm_options.target_system_cpu` / `target_system_features` override it; `target_system_cpu = "native"` targets the build machine's CPU and features (`cpu_and_features`). The jaifmt native test and the release smoke test fail if an x86-64 Linux jaifmt uses `ymm`/`zmm` registers or AVX-512 masks.
2. `lower::lower_program` declares every lowered function, foreign symbol and global, fills in global initialisers, then defines function bodies.
3. Verifies the module (debug builds of jaic, and any build with `JAIC_VERIFY_IR=1`; a release jaic skips the verifier, which costs about 3% of an `-O0` build), optionally runs the `default<On>` pipeline, and writes the object.

At `-O0` the target machine selects instructions with FastISel (`use_fast_isel` in `lib.rs`: `LLVMSetTargetMachineGlobalISel(false)`, `LLVMSetTargetMachineFastISel(true)`). LLVM turns GlobalISel on by default for unoptimised AArch64 code, and it was a third of codegen time there (`isWorthFoldingIntoExtendedReg` alone 11%); FastISel is the x86-64 default already, falls back to SelectionDAG per instruction, and made `-O0` codegen about 3x faster on a large program on Apple silicon. It is a per-machine setting, not a process option.

jaic sets no process-wide LLVM options (`LLVMParseCommandLineOptions`). Under LLVM 22 it passed `-unroll-add-parallel-reductions=false`: that release's runtime unroller, on by default for Apple CPUs, gave each unrolled copy of a reduction its own accumulator and, for a `sub` recurrence (`a -= b` in a loop of unknown length), combined them wrongly, so `-O2` printed different results from `-O0` and the interpreter ([llvm/llvm-project#201065](https://github.com/llvm/llvm-project/issues/201065), fixed in LLVM 23.1.0). The flag went with the move to LLVM 23. Corpus case `unrolled-sub-reduction` and the native test `optimized_sub_recurrence_matches_the_interpreter` still guard it. If an LLVM bug needs an option again, set it once before the first target machine is created (in `target_machine`, next to target registration), document the upstream issue, and remove it when jaic moves past the fixed release.

### Codegen units

`jaic build` calls `emit_objects`, which splits a large program into codegen units, one LLVM context and module per thread. LLVM code generation is a large share of an `-O0` build and the optimizer most of an `-O2` one, so this pays off on big programs like Focus.

- Units: one for a sanitized build, then `JAIC_CODEGEN_UNITS` if set. Otherwise an unoptimised build gets one per 5,000 IR instructions (`INSTS_PER_UNIT`, tuned on a 60k-line synthetic program, `jaifmt` and a hello world on 10 cores: more units kept winning down to roughly this size), capped at the core count, and an optimised one gets one per 20,000 (`OPT_INSTS_PER_UNIT`), capped at the core count and at 8 (`MAX_OPT_UNITS`). `--emit-ir`, sanitizers, the `output_*` IR/bitcode options and wasm targets always use one unit. `enable_split_modules = false` stops the split of unoptimised builds only; optimised builds are divided regardless (Focus sets it), and `JAIC_CODEGEN_UNITS=1` is the way to get a whole-program optimisation.
- Unoptimised builds: functions go largest first to the least loaded unit (`partition::flat`). Unit 0 also defines the globals. Optimised builds are divided by call graph, see [Dividing before the optimizer](#dividing-before-the-optimizer-partitionrs).
- Each module defines its own functions and declares only what it uses (`lower::Shard`): functions, globals and foreign symbols are declared on first reference (`Backend::func`, `global`, `foreign`), so a unit does not pay for the whole program's declarations. A lookup by name (`libc_call`, `__multi3`, `llvm.*` foreigns) first calls `declare_named`/declares the program's own symbol of that name, or LLVM would rename the later one (`write.1`) and the link would fail. Unit 0 defines the globals and so declares them all. Internal functions and globals that another unit refers to become hidden external symbols (still named `name.index`), so the objects link together but a shared library exports nothing extra; at `-O0` that is all of them.
- Objects are `path`, `path.1.o`, `path.2.o`, .... The CLI links or archives them all, then deletes them (an unoptimised macOS build with debug info keeps them, see [debug info](debug-info.md)). `-o x.o` stays a single module.

#### Dividing before the optimizer (`partition.rs`)

The optimizer is about 65% of an `-O2` build of Focus and runs on one thread when the program is one module, so an optimised program of at least two units' worth of instructions is divided first, each part is lowered into a module of its own and goes through the whole `default<On>` pipeline and machine code generation on its own thread. A previous attempt that only gave each unit its share of the functions, with every symbol external, saved little and cost 3.5% in speed of the generated code. What differs now:

- **Grouping by who refers to whom** (`partition::plan`). A function or global with exactly one referrer (a call, a function address, a global's relocation) hangs under it; the forest is cut where a group would exceed 1/(2 x units) of the program, and the groups are packed largest first. Most functions then have all their callers in their own module, and a function or global no other module refers to stays `internal` there (`Plan::func_shared`, `global_shared`), so single-caller inlining, dead code removal, argument promotion and constant propagation through arguments still work inside a module.
- **Copies for inlining** (`Plan::imports`). A function of another module that a module calls and that has at most 200 IR instructions (`IMPORT_MAX`; up to as many instructions again as the module owns, `IMPORT_BUDGET_PERCENT`) is also defined there as `available_externally`: the inliner can use it, the code generator drops it. What its body refers to becomes visible to the linker and is copied in turn when small. Exported (`#program_export`) functions are not copied.
- **Read-only data** is treated the same way: a read-only global of up to 16 KiB that another module reads is copied as `available_externally` (`Plan::global_imports`), so loads from small constant tables still fold. A read-only all-zero global without pointers, which is how the default value of a large struct is stored, gets a private copy in every module that uses it (`partition::replicated`): the optimizer turns a copy from it into a `memset` and the copy disappears, where one shared definition stayed in the executable (the Chess engine grew by 2.8 MB).
- The check handler (`check_failed`), exports and exported globals are always visible. Debug info for a global is emitted by the module that owns it.

Measured cost in generated code is about 1% to 3% more instructions on a search-heavy engine (Chess), none on a numeric and container benchmark. A hot function that is not copied because it is large and sits in another module is called instead of inlined; raise `IMPORT_MAX` to trade compile time for that.

#### Splitting after the optimizer (`split.rs`)

An optimized program too small to divide before the optimizer is optimized as one module, so inlining sees the whole program. Machine code generation (instruction selection, register allocation) is about half of an `-O2` build and works one function at a time, so `split::emit` runs it in parallel once the passes are done:

1. `units_for`: one unit per 10,000 LLVM instructions (`INSTS_PER_UNIT`), capped at the core count and at 4 (`MAX_UNITS`). Below 2 units nothing changes.
2. Internal and private definitions become hidden external symbols (unnamed ones are named `jaic.local.N`), and function definitions are assigned to units largest first.
3. The module is written to bitcode once. Each extra unit's thread parses it into its own `Context`, turns other units' function bodies into declarations, and makes the data declarations. Unit 0 is the original module cut down in place; it defines the data and keeps the appending globals such as `llvm.used`.
4. Each thread writes its object with its own `TargetMachine`. The objects get the same names as the codegen units above.

`strip_body` deletes whole blocks after cutting every use into them; it must not erase instructions one at a time. In LLVM 22 an erased instruction's debug records move to the next instruction, and from a block's last instruction into a context-wide table of trailing records keyed by the block's address. Deleting the block leaves the entry behind. A block that codegen later allocates at the same address then picks up another function's variables, and `DwarfDebug::finalizeModuleInfo` crashes on a variable it never gave a DIE. That happened in about one build in four, depending on heap layout, and never in `llc` on the same bitcode. Under LLVM 23.1.2, 60 `-O2` builds of jaifmt (half with `JAIC_SPLIT_UNITS=4`) and the `-O2` asm tests ran clean; keep deleting whole blocks regardless.

Parsing and cutting down are serialized under a mutex. Every unit briefly holds a whole copy of the module, so running them all at once raised peak memory by about one module per unit. With the mutex and the cap, an `-O2` build of Jails or jaison takes about a fifth less wall time for about 12% more peak RSS. More than 4 units gave no further speedup, because the optimizer, which stays serial, then dominates.

It applies (whatever `enable_split_modules` says) when `emit_objects` would use one unit at `-O1` and up (a program under two units of `OPT_INSTS_PER_UNIT` instructions, or a wasm target), without `--emit-ir`, a sanitizer or `JAIC_CODEGEN_UNITS`. `-o x.o` (`emit_object`) never splits.

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
- Anything module-level (a global, a constructor list) must be emitted once, in unit 0, and declared in the others. Use `Backend::internal_linkage` for new internal symbols so other units can reference them. A new way for lowered code to refer to another function or global (anything beyond `GlobalAddr`, `FuncAddr`, `ForeignAddr` and calls) must be added to `partition::references`, or the unit that needs it may find the symbol internal to another.
- Null checks (`null_check`) branch to a trap block that calls the `jaic.null_fail` helper (`null_fail_fn`). `FnState.facts` (`NullFacts`) remembers which pointers were checked in the current IR block, so a repeated load or a small constant offset is not checked again. Any new instruction that writes a slot, changes a pointer or lets a slot's address escape must update or reset the facts, or a stale fact hides a real null; `null_checks_skipped_after_a_check_still_catch_changes` covers the cases.
- Contexts discard value names, and `emit_objects` leaks each module, context and machine (the process is about to exit); `emit_object` and `jaic run` free them. Do not rely on names in emitted IR from `emit_objects`, except with `--emit-ir`.
- Small test programs use one unit; CI also runs `JAIC_CODEGEN_UNITS=4 cargo test -p jaic-cli --test native --test optimized_split --test debug_info --test embedded_data` (the `split-native` step), so a code generation bug that only shows across units fails there. With several units, `--emit-ir` and the `output_llvm_ir`/`output_bitcode` options write unit `n` to `x.n.ll` (`x.n.bc`) next to `x.ll`, instead of the units overwriting one file. A test that reads IR must read every unit (`read_ir_parts` in `native.rs`) or pin `JAIC_CODEGEN_UNITS=1` when it is about one module's objects (`custom_link_command`; `custom_link_command_with_several_codegen_units` is the split variant).
- Small optimized programs stay under `OPT_INSTS_PER_UNIT`; force the division before the optimizer with `JAIC_CODEGEN_UNITS=4` (`optimized_split.rs` compares 2, 3 and 5 units with a whole build), and the post-optimizer split with `JAIC_SPLIT_UNITS=4 cargo test -p jaic-cli --test native`. A declaration made from a definition must lose its body, personality and `!dbg` attachment (`strip_body`), or the verifier rejects the module.
- Windows (`Arch::Win64`): `#program_export` definitions are `dllexport` and `CompilerWrite` calls `_write`. See [Windows](windows.md).
- WebAssembly (`Arch::Wasm64`): foreign procedures become wasm imports, `#program_export`s get `wasm-export-name`, every function is `no-builtins`, and a weak `__multi3` is emitted. See [wasm target](wasm-target.md).

## Configuration

- `JAIC_VERIFY_IR=1` runs the LLVM verifier in a release jaic (`0` turns it off in a debug one).
- `JAIC_CODEGEN_UNITS=N` forces the unit count before the optimizer; `1` turns all splitting off, for a whole-program optimisation.
- `JAIC_SPLIT_UNITS=N` forces the post-optimizer unit count of an optimized build (for tests); `1` turns that split off.
- `INSTS_PER_UNIT`, `OPT_INSTS_PER_UNIT` and `MAX_OPT_UNITS` in `lib.rs`; `INSTS_PER_UNIT` and `MAX_UNITS` in `split.rs`; `IMPORT_MAX`, `IMPORT_BUDGET_PERCENT` and `GLOBAL_IMPORT_MAX` in `partition.rs`.
- `jaic_llvm::Options { opt_level, target, emit_ir, debug_info, sanitize, cpu, features }`, set from the CLI flags `-O0..-O3`, `--emit-ir file.ll`, `--no-debug-info`, `-sanitize` ([sanitizers](sanitizers.md)), `-os`, `-target triple`; `cpu` and `features` come from `llvm_options.target_system_cpu` / `target_system_features` (empty: the baseline CPU; `"native"`: the build machine's).
- Building the crate needs `LLVM_SYS_231_PREFIX` pointing at LLVM 23 (for example `/opt/homebrew/opt/llvm`); see [LLVM setup](../tools/llvm-setup.md).

## Dependencies

`inkwell` (LLVM 23), the system `cc` for linking, and `jaic` for the IR. Linking rules: [native linking](native-linking.md).
