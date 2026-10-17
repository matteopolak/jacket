# `#asm` blocks

## What it is

Jai's `#asm { ... }` inline assembly. jaic never emits machine code for it: each x64 instruction is parsed into an AST and lowered to ordinary IR operations on the Jai variables used as operands. A block behaves the same in the interpreter, the browser and the LLVM backend on any host CPU; an arm64 Mac runs x64 `#asm` correctly.

Supported: general-purpose, BMI/ADX, string, division, SSE through AVX2, FMA, AES/PCLMUL, SHA, F16C, GFNI and common AVX-512 including op-mask (`k`) registers. Anything else is the error `` unsupported #asm instruction `x` ``. The user-facing summary is [SIMD and `#asm`](../language/simd-asm.md). The authoritative instruction list is the catalog (`jaic::sema::asm_catalog`), which the [language server](language-server.md) also uses.

## How it works

- **Parsing** (`parser/asm.rs`) is mnemonic-agnostic and produces `ast::AsmBlock`: feature modifiers (`#asm AVX2 {`), then `;`-separated items. An item is an instruction (`mnemonic[.size|?size] operands`) or a declaration/pin (`x: gpr;`, `t: gpr === a;`). Operands are an expression, a memory operand `[base + index*scale + disp]`, or an inline declaration (`tmp:`).
- **Lowering** is split by family; `asm_inst` (`sema/asm.rs`) tries in order: `lookup_op` (integer core), `asm_scalar_inst` (`asm/scalar.rs`: division, BMI/ADX, string ops, CRC32), `asm_mask_inst` (`asm/mask.rs`), `asm_vec_inst` (`asm/vec.rs`, falling back to `simd::lookup_simd` in `asm/simd.rs`; F16C, SHA, GFNI in `asm/simd/ext.rs`).
- **Registers** are not modelled. Variables are read and written in place through their stack slots. Declared registers are locals added to the enclosing scope, start at zero, and only mean something inside the declaring block. Classes: `gpr` (8 bytes), `vec` (64 bytes), `str` (MMX), `omr`/`kmask` (op-mask). `===` pins are parsed and ignored, except `syscall` reads `gpr`s pinned to its argument registers.
- **Operation size**: suffix (`.b/.w/.d/.q`, `.8`..`.64`, `?T`), else the first Jai variable operand's size, else 64. 32-bit writes zero-extend; 8/16-bit writes merge.
- **Implicit registers are explicit operands**, in Jai's order:

  | Instruction | Form | Meaning |
  | --- | --- | --- |
  | `mul`/`imul` | `mul hi, lo, src` | `hi:lo = lo * src` |
  | `div`/`idiv` | `div hi, lo, d` | `lo = hi:lo / d`, `hi = remainder` |
  | `rep_movs.q` | `rep_movs.q di, si, c` | copy `c` elements |
  | `cmpxchg16b` | `cmpxchg16b d, a, [mem], c, b` | compare `d:a`, store `c:b` |

- **Flags** CF/ZF/SF/OF/PF are IR values tracked per block (unset reads as 0). The direction flag is tracked statically (`std`/`cld`; each block starts clear). Division traps on zero or overflow like hardware.
- **Atomics**: `lock_` read-modify-write and `xchg` on memory use a compare-and-swap loop. `lock_ cmpxchg16b` is atomic only against other `lock_ cmpxchg16b` (global spin lock), since the IR has no 128-bit CAS.
- **Vectors**: a `vec` register is a 64-byte local; each instruction runs lane by lane into a scratch buffer, so aliasing works. Width: `.x/.y/.z` suffix, else 32 in an AVX block, 64 with AVX-512, otherwise 16. With an AVX feature a write clears the bytes above it. `float32`/`float64` variables act as vector registers (zero-extended copy, low bytes stored back). `pmovmskb` and `pshufb` lower to `Intrinsic::SignMask`/`ByteLookup` (indices masked with `0x8F` to match hardware).
- **Masks**: `dst: &k` (merge) / `&*k` (zero) apply after the instruction; masked stores write only selected elements; compares into an `omr` write one bit per lane.
- **`syscall rcx_out:, r11_out:, rax_inout`** is the Linux x86-64 system call. Native x86-64 builds emit the real instruction; the interpreter calls the host `syscall(2)` on Linux x86-64 and traps elsewhere; hosts without native linking (sandbox, `-os wasm`, browser) return `-EPERM`.
- **Not supported**: x87, `push`/`pop`, `call`/`jmp`, `int n` other than 3 (no stack pointer or labels), I/O and privileged instructions, AVX-512 `getexp/getmant/scalef/fpclass/range/reduce`, BF16/FP16, SHA-512/SM3/SM4, AMX. Add them as `SOp`s when a program needs one.
- **Known differences from hardware**: `rcp*/rsqrt*` are exact; single-precision FMA rounds twice in rare cases; MXCSR is not modelled (round to nearest, no exceptions); `cpuid`/`xgetbv` report no features so runtime dispatch picks the baseline path; `rdpid` is 0, `stmxcsr` 0x1f80.

## How to change it

- **Integer instruction**: an `Op` in `lookup_op` plus an arm in `asm_inst`, or an `XOp` in `scalar.rs` if it has implicit operands or a loop. Reuse `asm_read`/`asm_write`, `rmw_begin`/`rmw_end` (atomic when `lock_`), `asm_alu`.
- **Small family with its own operand rules**: an `SOp` forwarding to `simd/ext.rs`.
- **Op-mask instruction**: a `KOp` and `lookup_kop` entry in `mask.rs`.
- **Vector instruction**: a `VOp`/`Lane` in `vec.rs` for lane-wise forms; otherwise an `SOp` in `simd.rs` (mnemonic in `lookup_simd`, `elem_size`, `dst_class` if it writes a mask or gpr, an arm in `asm_simd`).
- **Condition code**: `Cond` + `lookup_cond` + `eval_cond`.
- **Editor catalog** (`sema/asm/catalog.rs`): each new variant needs an arm in its feature, forms and description matches (exhaustive, the build says where). Catalog tests fail if a spelling the lookups accept is not listed. Register classes live in `REG_CLASSES`.
- **Tests**: add a case to the matching `tests/stdlib/asm-*.jai`. Record expected values on real hardware (an x86-64 C program, or under Rosetta with `ROSETTA_ADVERTISE_AVX=1`) rather than deriving them from the implementation. AVX-512 has no hardware here, so `asm-avx512-masks.jai` computes expectations in plain Jai.
- **Gotchas**:
  - A `Val` defined inside a loop or CAS retry is valid only after the loop exit dominates it; keep loop-carried values in stack slots.
  - IR `select` evaluates both arms, so clamp before a float-to-int conversion (out-of-range `FToS` is poison in LLVM).
  - x86 masks shift counts first (`& 31`/`& 63`); IR shifts are total, so only the masking is needed.

## Configuration

None. Feature modifiers after `#asm` are validated against `FEATURES` in `sema/asm.rs` and only choose the default vector width (`AVX*` = 256, `AVX512*` = 512). `#if CPU == .X64` selects Jai-level fallbacks as usual.

## Dependencies

`ir` (`Intrinsic::{CompareAndSwap, Popcount, Ctlz, Cttz, Bswap, Fma, SignMask, ByteLookup, ...}`), `sema::calls` (macros, `Code` parameters), `interp` and `jaic-llvm` for the intrinsics. Tests: `tests/stdlib/lang-asm.jai` and `asm-*.jai` (swept by `tools/jaic-sweep.py stdlib`; some also run natively in `crates/jaic-cli/tests/native.rs`) and the `asm_*` parser tests.
