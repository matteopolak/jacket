# SIMD and `#asm`

## What it is

`#asm` blocks with x64 scalar, string, SSE through AVX2, FMA, AES and AVX-512 instructions, including op-mask registers. jaic never emits machine code for them: each instruction is lowered to ordinary IR, so `#asm` runs on any CPU, in the interpreter and in the browser. This page is the user-facing view; the instruction list and internals are in [jaic `#asm` blocks](../compiler/asm.md).

## How it works

Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory operands, and declared registers. Registers are not modelled: pinning (`=== rax`) is accepted and ignored, and a declared `gpr` is a zero-initialised 64-bit local.

```jai
x: s64 = 10;
#asm { add x, 5; sub x, 3; }        // x == 12

a := float32.[1, 2, 3, 4];
b := float32.[10, 20, 30, 40];
pa, pb := a.data, b.data;
#asm {
    movups.x v:, [pa];
    movups.x w:, [pb];
    mulps.x v, w;                   // two operands: dst op= src
    movups.x [pa], v;
}
```

- Operand size comes from the `.b/.w/.d/.q` suffix, else the first Jai variable operand, else 64 bits. 32-bit writes zero-extend; 8- and 16-bit writes merge into the low bits.
- Vector width comes from `.x/.y/.z` (16/32/64 bytes), else the block default (256 bits under an `AVX` feature modifier). A leading `v` (`vaddps`) is accepted. Two operands mean `dst op= src`, three mean `dst = a op b`.
- Flags (CF, ZF, SF, OF, PF) are tracked within one block for `setcc`/`cmovcc`. `lock_` prefixed memory operations use a compare-and-swap loop.
- Implicit registers are written as explicit operands: `div hi, lo, d`, `rep_movs.q di, si, c`, `cqo d, a`, `cmpxchg16b d, a, [mem], c, b`. Division by zero or overflow traps.
- Feature modifiers after `#asm` (`AVX`, `AVX2`, `AVX512F`, `BMI2`, any name in `FEATURES` in `asm.rs`) only choose the default vector width and VEX zeroing of upper bytes.
- EVEX decorations: `[mem]!` broadcasts one element; `v !z` / `!n` / `!d` / `!u` pick `cvtps2dq` rounding; `dst: &* mask` and `& mask` zero or merge masked-off lanes. Mask registers (`omr`, `kmask`) are 8-byte locals.
- Unsupported instructions are a compile error naming the instruction. That includes x87 (Jai's `#asm` has none), `syscall`, `push`/`pop`, privileged instructions, F16C, SHA, GFNI and some AVX-512 extensions.
- `rcp*`/`rsqrt*` return exact results; `cpuid` and `xgetbv` report no features.

## How to change it

Add an instruction to the mnemonic match in `sema/asm.rs` or `asm/scalar.rs` (scalar), `asm/mask.rs` (op-mask) or `lookup_simd` in `asm/simd.rs` (vector), and add a case to the matching test.

Vectors are 64-byte stack locals processed through a scratch buffer, so aliased operands (`addps v, v, v`) behave as on hardware. Keep it that way: never write the destination before every source is read.

Tests (each prints `ok`): `tests/stdlib/lang-asm.jai`, `asm-vector-instructions.jai`, `asm-scalar-extended.jai`, `asm-simd-extended.jai`, `asm-avx512-masks.jai`, `asm-evex-decorations.jai`, `machine-x64-intrinsics.jai`. The expected values were recorded on x86-64 hardware (under Rosetta), so they also check the lowering against real instructions.

## Dependencies

`AsmBlock`, `AsmInst`, `AsmOperand` in `ast.rs`, the asm parser, and the IR builder.
