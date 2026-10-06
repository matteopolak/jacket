# SIMD and `#asm`

## What it is

`#asm` blocks with x64 scalar, string, SSE through AVX2, FMA, AES and AVX-512 instructions, including op-mask registers. jaic never emits machine code for them: each instruction is lowered to ordinary IR, so `#asm` runs on any CPU, in the interpreter and in the browser. This page is the user-facing view; the instruction list and internals are in [jaic `#asm` blocks](../compiler/asm.md).

## How it works

Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory operands, and declared registers {#asm.1}. Registers are not modelled: pinning (`=== rax`) is accepted and ignored {#asm.2}, and a declared `gpr` is a zero-initialised 64-bit local {#asm.3}.

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

The first block leaves `x == 12` {#asm.4}; afterwards `a` holds `10, 40, 90, 160` {#asm.5}.

- Operand size comes from the `.b/.w/.d/.q` suffix, else the first Jai variable operand, else 64 bits {#asm.6}. 32-bit writes zero-extend {#asm.7}; 8- and 16-bit writes merge into the low bits {#asm.8}.
- Vector width comes from `.x/.y/.z` (16/32/64 bytes), else the block default (256 bits under an `AVX` feature modifier) {#asm.9}. A leading `v` (`vaddps`) is accepted {#asm.10}. Two operands mean `dst op= src`, three mean `dst = a op b` {#asm.11}.
- Flags (CF, ZF, SF, OF, PF) are tracked within one block for `setcc`/`cmovcc` {#asm.12}. `lock_` prefixed memory operations use a compare-and-swap loop {#asm.13}.
- Implicit registers are written as explicit operands: `div hi, lo, d`, `rep_movs.q di, si, c`, `cqo d, a`, `cmpxchg16b d, a, [mem], c, b` {#asm.15}. Division by zero or overflow traps {#asm.14}.
- Feature modifiers after `#asm` (`AVX`, `AVX2`, `AVX512F`, `BMI2`, any name in `FEATURES` in `asm.rs`) only choose the default vector width and VEX zeroing of upper bytes {#asm.16}.
- EVEX decorations: `[mem]!` broadcasts one element {#asm.17}; `v !z` / `!n` / `!d` / `!u` pick `cvtps2dq` rounding {#asm.18}; `dst: &* mask` and `& mask` zero or merge masked-off lanes {#asm.19}; a masked store writes only selected elements {#asm.20}. Mask registers (`omr`, `kmask`) are 8-byte locals; the `k*` instructions operate on them, and compares, `ptestm` and `pmov*2m` write them {#asm.21}.
- Unsupported instructions are a compile error naming the instruction {#asm.22}. That includes x87 (Jai's `#asm` has none) {#asm.23}, `syscall`, `push`/`pop`, privileged instructions, F16C, SHA, GFNI and some AVX-512 extensions.
- `rcp*`/`rsqrt*` return exact results {#asm.24}; `cpuid` and `xgetbv` report no features {#asm.25}.

## How to change it

Add an instruction to the mnemonic match in `sema/asm.rs` or `asm/scalar.rs` (scalar), `asm/mask.rs` (op-mask) or `lookup_simd` in `asm/simd.rs` (vector), and add a case to the matching test.

Vectors are 64-byte stack locals processed through a scratch buffer, so aliased operands (`addps v, v, v`) behave as on hardware. Keep it that way: never write the destination before every source is read.

Tests (each prints `ok`): `tests/stdlib/lang-asm.jai`, `asm-vector-instructions.jai`, `asm-scalar-extended.jai`, `asm-simd-extended.jai`, `asm-avx512-masks.jai`, `asm-evex-decorations.jai`, `machine-x64-intrinsics.jai`. The expected values were recorded on x86-64 hardware (under Rosetta), so they also check the lowering against real instructions.

## Dependencies

`AsmBlock`, `AsmInst`, `AsmOperand` in `ast.rs`, the asm parser, and the IR builder.
