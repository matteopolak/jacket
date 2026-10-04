# jaic `#asm` blocks

## What it is

Support for Jai's `#asm { ... }` inline assembly in the new compiler core (`crates/jaic`). jaic never emits machine
code for it: each x64 instruction is parsed into an AST and lowered to ordinary IR operations on the Jai variables used
as operands. A block therefore behaves identically in the interpreter (`#run`, compile-time), the browser (wasm) and
the LLVM backend, on any host CPU. Only scalar general-purpose instructions are supported; SIMD is rejected with an
error naming the instruction.

## How it works

- **Parsing** (`parser/asm.rs`) is mnemonic-agnostic and produces `ast::AsmBlock` (`ExprKind::Asm`): feature
  modifiers (`#asm AVX2 {`), then `;`-separated items. An item is an instruction (`mnemonic[.size|?size] operands`) or a
  declaration/pin statement (`x: gpr;`, `t: gpr === a;`, `x === a;`). Operands are a variable/constant expression, a
  memory operand `[base + index*scale +/- disp]` (each term a unary expression, so `*x` and `(size_of(u64))` work) or
  an inline declaration (`tmp:`, `tmp: gpr === 15`). `===` lexes as `==` `=` and is recognised by adjacency.
- **Lowering** (`sema/asm.rs`, entered from `check_expr` via `check_asm`):
  - Variables are read and written in place through their stack slots. Declared registers (`mov a:, 10`,
    `x: gpr`) are 64-bit locals added to the *enclosing* scope (blocks are not scopes; macros see them, later blocks
    reuse them). They start at zero. `Compiler::asm_regs` records which locals came from declarations so their size is
    not "natural" (see below). `vec`/`str`/`omr` declarations are accepted but any use is an error.
  - Operation size: explicit suffix (`.b/.w/.d/.q`, `.8/.16/.32/.64`, or `?T` / `?BITS` for a type or bit count), else
    the size of the first Jai variable operand, else 64 bits. As on hardware, 32-bit writes zero-extend into a 64-bit
    destination and 8/16-bit writes merge into its low bits.
  - Registers are not modeled: `===` pins are parsed and ignored. Instructions with implicit registers take them
    explicitly as Jai does (`mul hi, lo, src` computes `hi:lo = lo * src`).
  - Flags CF/ZF/SF/OF are IR values tracked per block (unset flags read as 0); `setcc`/`cmovcc` evaluate conditions
    from them. Parity (`setp`) is not modeled.
  - `lock_`-prefixed read-modify-write instructions on a memory operand, and `xchg` with memory, use a
    compare-and-swap retry loop (`Intrinsic::CompareAndSwap`), so they are atomic on native targets. Register bit
    offsets on memory (`bts [p], idx`) address a bit string like hardware.
  - `__reg` (alias of `Code`) macro parameters bind to the caller's operand by name (`calls.rs::arg_cost` admits a
    variable for a `Code` parameter; `asm_value` resolves the code expression in the caller's scope).
- **IR**: only four intrinsics were added, `Popcount`, `Ctlz`, `Cttz`, `Bswap` (args: value, width in bits).
  They are implemented in `interp/mod.rs` and in `jaic-llvm` (`llvm.ctpop/ctlz/cttz/bswap`).

### Supported instructions

mov, movzx/movsx (`movzxbw`, `movsxbd`, `movsxd`), movbe, lea, xchg, xadd, cmpxchg, add, sub, adc, sbb, and, or, xor,
cmp, test, inc, dec, neg, not, shl/sal/shr/sar/rol/ror (count 1, immediate or variable), bt/bts/btr/btc, bsf, bsr,
popcnt, lzcnt, tzcnt, bswap, blsr/blsi/blsmsk, imul (2- and 3-operand, and `imul hi, lo, src`), mul, setcc and cmovcc
(e/z, ne/nz, b/c, ae/nc, be, a, s, ns, o, no, l, ge, le, g and aliases), clc/stc/cmc, nop/pause/fences/cld/std,
int3, rdtsc/rdtscp (`rdtsc hi:, lo`; a monotonic counter), rdrand, cpuid (all four outputs 0), with an optional
`lock_` prefix. `cmpxchg` takes the accumulator explicitly: `cmpxchg dest, src, acc` (or `acc, dest, src`).

Not supported (compile error `unsupported #asm instruction 'x'`): everything else, notably SIMD (`movdqu`, `pxor`,
`vpaddd`, gathers...), `div`/`idiv`, string instructions (`rep_movs`), `syscall`, 128-bit `cmpxchg16b`, shld/shrd.
`rcl/rcr` are unsupported. `imul`/`mul` set CF/OF from the full 128-bit product (`wide_mul`).

## How to change it

- New instruction: add an `Op` variant and its mnemonic to `lookup_op`, then a match arm in `asm_inst`. Reuse
  `asm_read`/`asm_write` (size-aware operand access), `rmw_begin`/`rmw_end` (atomic when `lock_`), and
  `asm_alu` for flag-setting arithmetic. Add a case to `tests/stdlib/lang-asm.jai`.
- New condition code: `Cond` + `lookup_cond` + `eval_cond`.
- New operand syntax: `parser/asm.rs` (AST in `ast.rs`), then `asm_operand`/`asm_value`.
- Gotchas: a `Val` defined inside a CAS retry loop is only valid after `rmw_end` (it dominates the exit block), so
  derive flags from `old`/`new` between `rmw_begin` and `rmw_end`. IR shifts are total (counts >= width give 0 / sign
  fill) so no guards are needed, but x86 masks the count first (`& 31`/`& 63`), which `asm_shift` does.
  The prelude no longer declares `__reg` as a distinct type; it is a root builtin alias of `Code`.

## Configuration

None. Feature modifiers after `#asm` are validated against a fixed list (`FEATURES` in `sema/asm.rs`) and otherwise
ignored; `#if CPU == .X64` still selects Jai-level fallbacks as usual (on arm64 and wasm hosts those branches are
simply not compiled).

## Dependencies

`ir` (builder, `Intrinsic::{CompareAndSwap, CycleCounter, Pause, DebugBreak, Popcount, Ctlz, Cttz, Bswap}`),
`sema::calls` (macro expansion, `Code` parameters), `interp` and `jaic-llvm` for the intrinsics. Tests:
`tests/stdlib/lang-asm.jai` (swept by `tools/jaic-sweep.py stdlib`) and the `asm_*` parser tests.
