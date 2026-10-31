# `#asm` blocks

jaic never emits machine code for `#asm { ... }`. Each x64 instruction is parsed into an AST and lowered to ordinary IR operations on the Jai variables used as operands, so a block behaves the same in the interpreter, the browser and the LLVM backend on any host CPU. The user-facing summary is [SIMD and `#asm`](../language/simd-asm.md).

Supported: general-purpose, BMI/ADX, string, division, SSE through AVX2, FMA, AES/PCLMUL, SHA, F16C, GFNI and common AVX-512 with op-mask registers. Anything else is `` unsupported #asm instruction `x` ``.

## How it works

- **Parsing** (`parser/asm.rs`) is mnemonic-agnostic and produces `ast::AsmBlock`: feature modifiers (`#asm AVX2 {`), then `;`-separated instructions (`mnemonic[.size|?size] operands`) or declarations and pins (`x: gpr;`, `t: gpr === a;`).
- **Lowering** is split by family; `asm_inst` (`sema/asm.rs`) tries `lookup_op` (integer core), `asm_scalar_inst` (division, BMI/ADX, string ops, CRC32), `asm_mask_inst`, then `asm_vec_inst` (falling back to `simd::lookup_simd`).
- **Registers** are not modelled. Variables are read and written in place through their stack slots. Declared registers are zero-initialised locals that only mean something inside their block. Classes: `gpr`, `vec` (64 bytes), `str`, `omr`/`kmask`.
- **Operation size**: suffix (`.b/.w/.d/.q`, `?T`), else the first Jai variable operand's size, else 64. 32-bit writes zero-extend; 8/16-bit writes merge.
- **Implicit registers are explicit operands**: `mul hi, lo, src` is `hi:lo = lo * src`; `div hi, lo, d` is `lo = hi:lo / d`, `hi = remainder`; `rep_movs.q di, si, c` copies `c` elements; `cmpxchg16b d, a, [mem], c, b` compares `d:a` and stores `c:b`.
- **Flags** CF/ZF/SF/OF/PF are IR values tracked per block (unset reads as 0). Division traps on zero or overflow like hardware.
- **Atomics**: `lock_` read-modify-write and `xchg` on memory use a compare-and-swap loop. `lock_ cmpxchg16b` is atomic only against other `lock_ cmpxchg16b`, since the IR has no 128-bit CAS.
- **Vectors**: each instruction runs lane by lane into a scratch buffer, so aliasing works. Width: `.x/.y/.z` suffix, else 32 in an AVX block, 64 with AVX-512, otherwise 16.
- **Masks**: `dst: &k` (merge) / `&*k` (zero) apply after the instruction.
- **`syscall rcx_out:, r11_out:, rax_inout`** is the Linux x86-64 system call; argument registers are pinned to `di`, `si`, `d`, `10`, `8`, `9`.

### Not supported

x87, `push`/`pop`, `call`/`jmp`, `int n` other than 3 (no stack pointer or labels), I/O and privileged instructions, AVX-512 `getexp/getmant/scalef/fpclass/range/reduce`, BF16/FP16, SHA-512/SM3/SM4, AMX. Add them as `VOp`s when a program needs one.

Known differences from hardware: `rcp*/rsqrt*` are exact; single-precision FMA rounds twice in rare cases; MXCSR is not modelled (round to nearest, no exceptions); `cpuid`/`xgetbv` report no features, so runtime dispatch picks the baseline path.

## How to change it

- Integer instruction: an `Op` in `lookup_op` plus an arm in `asm_inst`, or an `XOp` in `scalar.rs` for implicit operands or loops. Reuse `asm_read`/`asm_write`, `rmw_begin`/`rmw_end`, `asm_alu`.
- Op-mask instruction: a `KOp` in `mask.rs`. Vector instruction: a `VOp` in `vec.rs`, or a row in `lookup_simd`'s tables. Condition code: `Cond`, `lookup_cond`, `eval_cond`.
- Editor catalog (`sema/asm/catalog.rs`): each new variant needs arms in its feature, forms and description matches; catalog tests fail if a spelling the lookups accept is missing.
- Tests: add a case to the matching `tests/compiler/regressions/asm-*.jai`. Record expected values from real hardware (an x86-64 C program, or Rosetta with `ROSETTA_ADVERTISE_AVX=1`), not from the implementation.
- Gotchas:
  - A value defined inside a loop or CAS retry is valid only after the loop exit dominates it; keep loop-carried values in stack slots.
  - IR `select` evaluates both arms, so clamp before a float-to-int conversion (out-of-range is poison in LLVM).
  - x86 masks shift counts first (`& 31`/`& 63`); IR shifts are total, so only the masking is needed.

## Configuration

Feature modifiers after `#asm` are checked against `FEATURES` in `sema/asm.rs` and only pick the default vector width (`AVX*` 256, `AVX512*` 512). `#if CPU == .X64` selects fallbacks as usual.

## Dependencies

`ir` intrinsics (`CompareAndSwap`, `Popcount`, `Fma`, ...), `sema::calls`, `interp` and `jaic-llvm`. Tests: `tests/compiler/regressions/lang-asm.jai` and `asm-*.jai`.
