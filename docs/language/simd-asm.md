# SIMD and `#asm`

## What it is

`#asm` blocks (x64 scalar, string, SSE through AVX2, FMA, AES and AVX-512 instructions with op-mask registers) and how `jaic` runs them on any CPU: each instruction is lowered to ordinary IR, never to machine code. The full instruction list and the internals are in [`docs/compiler/asm.md`](../compiler/asm.md).

## How it works

`crates/jaic/src/sema/asm.rs` lowers general-purpose instructions, `asm/scalar.rs` division, string and BMI instructions, `asm/mask.rs` the `k*` op-mask instructions, and `asm/vec.rs` plus `asm/simd.rs` vector (`vec` register) instructions lane by lane. Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory operands and register declarations. Registers are not modeled: pinning (`=== rax`) is accepted and ignored, and a declared `gpr` is a zero-initialized 64-bit local of the enclosing scope.

```jai
x: s64 = 10;
#asm { add x, 5; sub x, 3; }        // x == 12
```

Vector example (128-bit legacy form, `dst op= src`):

```jai
a := float32.[1, 2, 3, 4];
b := float32.[10, 20, 30, 40];
pa, pb := a.data, b.data;
#asm {
    movups.x v:, [pa];
    movups.x w:, [pb];
    mulps.x v, w;
    movups.x [pa], v;
}
```

Details:

- Operation size comes from the `.b/.w/.d/.q` suffix, else the first Jai variable operand, else 64 bits. 32-bit writes zero-extend; 8- and 16-bit writes merge into the low bits.
- Vector width comes from `.x/.y/.z` (16/32/64 bytes), else the block default (a block with an `AVX` feature modifier defaults to 256 bits). A leading `v` (`vaddps`) is accepted. Two operands mean `dst op= src`, three mean `dst = a op b`.
- Flags (CF, ZF, SF, OF, PF) are tracked as IR values within one block; `setcc`/`cmovcc` read them. `lock_`-prefixed memory operations use a compare-and-swap loop.
- Implicit registers are explicit operands, as in Jai: `div hi, lo, d` (quotient to `lo`, remainder to `hi`; division by zero or overflow traps), `rep_movs.q di, si, c`, `repe_cmps.b di, si, c`, `cqo d, a`, `cmpxchg16b d, a, [mem], c, b`.
- Feature modifiers after `#asm` (`AVX`, `AVX2`, `AVX512F`, `BMI2`, any CPUID feature name) are listed in `FEATURES` in `asm.rs`; they only choose the default vector width and VEX zeroing of upper register bytes.
- EVEX decorations (`AsmInst.evex`, `AsmMem.broadcast`): `[mem]!` broadcasts one element to every lane, `v !z` / `!n` / `!d` / `!u` pick the rounding of `cvtps2dq`, and `dst: &* mask` / `& mask` zero or merge the masked-off lanes after the instruction ran; a masked store writes only selected elements. Mask registers (`omr`, `kmask`) are 8-byte locals; the `k*` instructions operate on them, and compares, `ptestm`, `pmov*2m` write them. Tests: `tests/stdlib/asm-evex-decorations.jai`, `tests/stdlib/asm-avx512-masks.jai`.
- Unsupported instructions are a compile error naming the instruction. Jai's `#asm` has no x87 instructions, so `fld` and friends are rejected; also `syscall`, `push`/`pop`, privileged instructions, F16C, SHA, GFNI and a few AVX-512 extensions (see the compiler doc).
- `rcp*`/`rsqrt*` give exact results, and `cpuid`/`xgetbv` report no features.

The executable examples are `tests/stdlib/lang-asm.jai`, `asm-vector-instructions.jai`, `asm-scalar-extended.jai`, `asm-simd-extended.jai`, `asm-avx512-masks.jai` and `machine-x64-intrinsics.jai`; each prints `ok` under `jaic run`. The scalar and SIMD expectations were recorded on x86-64 hardware (under Rosetta), so they also check that the IR lowering matches real instructions.

## How to change it

To add an instruction, extend the mnemonic match in `asm.rs` / `asm/scalar.rs` (scalar), `asm/mask.rs` (op-mask) or `lookup_simd` in `asm/simd.rs` (vector) and add a case to the matching test. Because vectors are 64-byte stack locals processed through a scratch buffer, aliasing operands (`addps v, v, v`) already behave as on hardware; keep that property by never writing the destination before all sources are read.

## Configuration

None. Behavior is identical in the interpreter, the browser build and native builds.

## Dependencies

`crates/jaic/src/ast.rs` (`AsmBlock`, `AsmInst`, `AsmOperand`), the asm parser, and the IR builder.
