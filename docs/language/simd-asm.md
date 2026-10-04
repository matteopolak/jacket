# SIMD and `#asm`

## What it is

`#asm` blocks (x64 scalar and vector instructions) and how `jaic` runs them on any CPU: each instruction is lowered to ordinary IR, never to machine code.

## How it works

`crates/jaic/src/sema/asm.rs` lowers general-purpose instructions; `crates/jaic/src/sema/asm/vec.rs` lowers vector (`vec` register) instructions lane by lane. Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory operands and register declarations. Registers are not modeled: pinning (`=== rax`) is accepted and ignored, and a declared `gpr` is a zero-initialized 64-bit local of the enclosing scope.

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
- Flags (CF, ZF, SF, OF) are tracked as IR values within one block; `setcc`/`cmovcc` read them. `lock_`-prefixed memory operations use a compare-and-swap loop.
- Feature modifiers after `#asm` (`AVX`, `AVX2`, `AVX512F`, `BMI2`, ...) are listed in `FEATURES` in `asm.rs`; they never change lowering except for VEX zeroing of upper register bytes.
- EVEX decorations (`AsmInst.evex`, `AsmMem.broadcast`): `[mem]!` broadcasts one element to every lane, `v !z` / `!n` / `!d` / `!u` pick the rounding of `cvtps2dq`, and `dst: &* mask` / `& mask` zero or merge the masked-off lanes after the instruction ran (the mask register is a vec-class local; `kmovb/w/d/q` move to and from it). Test: `tests/stdlib/asm-evex-decorations.jai`.
- Unsupported instructions (string ops, division, x87, mask registers) are a compile error naming the instruction.

The executable examples are `tests/stdlib/lang-asm.jai`, `tests/stdlib/asm-vector-instructions.jai` and `tests/stdlib/machine-x64-intrinsics.jai`; the first two print `ok` under `jaic run`.

## How to change it

To add an instruction, extend the mnemonic match in `asm.rs` (scalar) or the `Lane` operations in `asm/vec.rs` (vector) and add a case to the matching test. Because vectors are 64-byte stack locals processed through a scratch buffer, aliasing operands (`addps v, v, v`) already behave as on hardware; keep that property by never writing the destination before all sources are read.

## Configuration

None. Behavior is identical in the interpreter, the browser build and native builds.

## Dependencies

`crates/jaic/src/ast.rs` (`AsmBlock`, `AsmInst`, `AsmOperand`), the asm parser, and the IR builder.
