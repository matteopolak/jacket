# jaic LLVM backend

## What it is

`crates/jaic-llvm` translates the `jaic::ir::Program` produced by the new compiler core into LLVM IR, writes a native object file, and links it with the system `cc`. `crates/jaic-cli` (binary `jaic`) exposes it as `jaic build <file.jai> [-o out]`; `jaic run` still uses the interpreter and `jaic check` only type-checks.

## How it works

`emit_object(program, options, path)` (in `lib.rs`):

1. Creates a target machine for the host (or `Options::target`, e.g. `x86_64-pc-windows-gnu` for `-os windows`), sets triple and data layout. The triple picks the `jaic::abi::Arch`.
2. `lower::lower_program` declares every lowered function, foreign symbol and global, fills in global initializers, then defines function bodies.
3. Verifies the module, optionally runs the `default<On>` pipeline, writes the object.

`emit_objects(program, options, path)` is what `jaic build` uses for executables and libraries. A large unoptimized program is split into codegen units, one LLVM context and module per thread:

- The unit count is `JAIC_CODEGEN_UNITS` when set, otherwise one unit per 20,000 IR instructions (`INSTS_PER_UNIT`), capped at the core count. `-O1` and up, and `--emit-ir`, always use one unit, so LLVM can still inline across the whole program.
- Functions are assigned largest first to the least loaded unit. Unit 0 also defines the globals.
- Each module defines only its own functions and declares the rest (`lower::Shard`). Internal functions and globals become hidden external symbols there (still named `name.index`), so the objects link together but nothing is exported from a shared library.
- The objects are `path`, `path.1.o`, `path.2.o`... The CLI links them all (or archives them for a static library) and deletes them. `-o x.o` object output stays a single module.

Focus (`first.jai`, `-O0`) builds in 2.7s instead of 3.7s on a 10-core machine. LLVM's code generation was about a third of the build.

Lowering rules (`lower.rs`):

- Each IR `Val` is an LLVM SSA value. Blocks are emitted in reverse post-order from the entry, so a value's definition always precedes its uses; unreachable IR blocks are skipped. No phis are needed because the IR keeps mutable state in slots.
- `Slot`s become allocas in a dedicated first `allocas` block that branches to IR block 0. Temporaries used for ABI marshalling are allocated there too.
- Pointers are opaque; `PtrAdd` is an `i8` GEP. Loads and stores carry natural alignment.
- Integer semantics follow the interpreter: division by zero traps (`llvm.trap`), `INT_MIN / -1` wraps, shifts of `>= bits` give 0 (`AShr` clamps), float to int conversions saturate, `FNe` is unordered-not-equal.
- Globals are packed structs of byte runs and pointer relocations (`Reloc`), with the IR alignment and `read_only` flag. `#program_export` names stay external; everything else is internal and suffixed with its index.
- Foreign functions/variables are external declarations, de-duplicated by symbol. All calls are emitted as indirect calls with a function type computed from the call's `Sig`, so one symbol can be called with several signatures.
- `Conv::Jai` functions map parameters and scalar results 1:1 (several results become a struct return).
- Intrinsics: `Memcpy` is `memmove`, `Memcmp` calls libc `memcmp` and normalizes to -1/0/1 (`I16`), `CompilerWrite` calls `write(1|2, ...)`, `CompareAndSwap` is a seq_cst `cmpxchg` whose width comes from the operand type, `IsCompileTime` is the constant 0, `CycleCounter` reads `cntvct_el0` on AArch64 and `rdtsc` on x86-64, `Fma` is `llvm.fma.f64` (used by `#asm` FMA instructions). `Loc` markers set the debug location (see [debug info](debug-info.md)).

### C ABI (`jaic::abi`)

`Conv::C` signatures with `Sig::c_abi` set use the real calling convention for by-value aggregates:

| | AArch64 | x86-64 System V | Microsoft x64 (`Win64`) |
|---|---|---|---|
| in registers | <= 16 bytes: `i64` chunks in x registers; homogeneous float aggregates (<= 4 members) as separate `float`/`double` scalars | <= 16 bytes, per eightbyte: `i64`, `double`, `float` or `<2 x float>` | exactly 1, 2, 4 or 8 bytes: one `i64` |
| other args | caller copy, pointer passed | `byval` pointer | caller copy, pointer passed |
| return | chunk(s) in registers, else `sret` | same | same |

The IR passes aggregates by pointer, so the call site copies into a scratch temp, loads the chunks and passes them as separate LLVM arguments; returned chunks are stored to a temp and copied through the IR out-pointer (the last IR parameter, which is dropped from the LLVM signature). Variadic calls use a vararg function type whose parameters are only the declared ones (`Sig::c_fixed`; the call site appends the variadic arguments to `Sig::params`), so LLVM applies the platform's variadic convention (stack slots on Apple arm64). Test: `c_variadic_calls`.

Definitions with such signatures (`#c_call` callbacks C calls with structs) do the reverse in `bind_params`: register pieces are stored into an entry-block temp whose address stands in for the IR parameter, and `Ret` loads the return pieces from the IR out-pointer temp (`FnState::reg_ret`). `sret`/`byval` pointers are used directly. The classification lives in the `jaic` crate (`crates/jaic/src/abi.rs`) because the interpreter's native foreign calls use it too.

## How to change it

- New IR instruction/intrinsic: extend `Backend::inst` or `Backend::intrinsic` in `lower.rs`; keep semantics identical to `interp/mod.rs`.
- New target architecture: add an `Arch` variant and classification in `abi.rs`, plus the inline-asm intrinsics in `lower.rs`.
- Debug info lives in `debuginfo.rs` ([debug info](debug-info.md)); `lower.rs` only calls its hooks (`begin_function`, `declare_vars`, `enter_block`/`leave_block`, `loc`, `finish_entry`, `Backend::set` for watched addresses, `describe_globals`).
- Anything new at module level (a global, a constructor list) must be emitted once, in unit 0, and declared in the other units. `Backend::internal_linkage` gives the linkage of internal symbols; use it for new ones so they can be referenced across units.
- Check split codegen with `JAIC_CODEGEN_UNITS=4 cargo test -p jaic-cli --test native`. Small test programs otherwise use one unit.
- Gotchas: Small signed integers are not sign/zero-extended according to the C ABI because the IR does not carry signedness.
- Windows specifics (`Arch::Win64`): `#program_export` definitions are `dllexport`, `CompilerWrite` calls `_write`. See [Windows](windows.md).

## Configuration

- `JAIC_CODEGEN_UNITS=N` forces the number of codegen units (`1` turns splitting off).
- `jaic_llvm::Options { opt_level, target, emit_ir, debug_info }`; CLI flags `-O0..-O3`, `--emit-ir file.ll`, `-o output`, `-I dir`, `--no-debug-info`, `-os windows`, `-target triple`.
- `JAIC_STDLIB` overrides the standard library directory (as for `jaic run`).
- `LLVM_SYS_221_PREFIX` must point at an LLVM 22 install when building (for example `/opt/homebrew/opt/llvm`).

## Dependencies

- `inkwell` (workspace dependency, LLVM 22) and the system `cc` for linking.
- `jaic` for the IR. Library linking rules are in [native linking](native-linking.md).
- Tests: see [native linking](native-linking.md) and [C ABI](c-abi.md).
