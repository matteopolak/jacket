# SIMD and `#asm`

## What it is

`#asm` blocks with x64 scalar, string, SSE through AVX2, FMA, AES and AVX-512 instructions, including op-mask registers. jaic never emits machine code for them: each instruction lowers to ordinary IR, so `#asm` runs on any CPU, in the interpreter and in the browser. Instruction list and internals: [jaic `#asm` blocks](../compiler/asm.md).

## How it works

Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory and declared registers. Registers are not modelled: pinning (`=== rax`) is ignored except that `syscall` reads its pinned registers (Linux x86-64 only).

```jai
x: s64 = 10;
#asm { add x, 5; sub x, 3; }        // x == 12

a := float32.[1, 2, 3, 4];  b := float32.[10, 20, 30, 40];
pa, pb := a.data, b.data;
#asm {
    movups.x v:, [pa];
    movups.x w:, [pb];
    mulps.x v, w;                   // two operands: dst op= src
    movups.x [pa], v;
}
```

- Size comes from the `.b/.w/.d/.q` suffix, else the first variable operand, else 64 bits. 32-bit writes zero-extend; 8/16-bit writes merge. Vector width comes from `.x/.y/.z`, else the block default (256 bits under an `AVX` modifier).
- Flags are tracked within a block for `setcc`/`cmovcc`; `lock_` operations use a compare-and-swap loop. Implicit registers are explicit operands (`div hi, lo, d`).
- A floating-point variable is a vector register; an integer instruction cannot take one (compile error).
- A `name:` destination declares a register of the class the instruction writes.
- Unsupported instructions (x87, `push`/`pop`, privileged, some AVX-512 extensions) are compile errors naming the instruction; reasons are in the compiler asm page. `cpuid` returns real data on x64 only.

## How to change it

Add a mnemonic in `sema/asm.rs` or `asm/scalar.rs`, `asm/mask.rs`, or `lookup_simd` in `asm/simd.rs`, plus a test case. Vectors are 64-byte locals processed through a scratch buffer so aliased operands behave as on hardware; never write the destination before every source is read. Tests: `tests/compiler/regressions/lang-asm.jai`, `asm-vector-instructions.jai`, `asm-avx512-masks.jai` (expected values recorded on x86-64 hardware). AST types: `AsmBlock`, `AsmInst`, `AsmOperand` in `ast.rs`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory operands, and declared registers {#asm.1}. Registers are not modelled: pinning (`=== rax`) is accepted and ignored {#asm.2}, except that `syscall` reads the registers pinned to its arguments (Linux x86-64 hosts and targets only), and a declared `gpr` is a zero-initialised 64-bit local {#asm.3}.
- The first block leaves `x == 12` {#asm.4}; afterwards `a` holds `10, 40, 90, 160` {#asm.5}.
- Operand size comes from the `.b/.w/.d/.q` suffix, else the first Jai variable operand, else 64 bits {#asm.6}. 32-bit writes zero-extend {#asm.7}; 8- and 16-bit writes merge into the low bits {#asm.8}.
- Vector width comes from `.x/.y/.z` (16/32/64 bytes), else the block default (256 bits under an `AVX` feature modifier) {#asm.9}. A leading `v` (`vaddps`) is accepted {#asm.10}. Two operands mean `dst op= src`, three mean `dst = a op b` {#asm.11}.
- Flags (CF, ZF, SF, OF, PF) are tracked within one block for `setcc`/`cmovcc` {#asm.12}. `lock_` prefixed memory operations use a compare-and-swap loop {#asm.13}.
- Implicit registers are written as explicit operands: `div hi, lo, d`, `rep_movs.q di, si, c`, `cqo d, a`, `cmpxchg16b d, a, [mem], c, b` {#asm.15}. Division by zero or overflow traps {#asm.14}.
- Feature modifiers after `#asm` (`AVX`, `AVX2`, `AVX512F`, `BMI2`, any name in `FEATURES` in `asm.rs`) only choose the default vector width and VEX zeroing of upper bytes {#asm.16}.
- EVEX decorations: `[mem]!` broadcasts one element {#asm.17}; `v !z` / `!n` / `!d` / `!u` pick `cvtps2dq` rounding {#asm.18}; `dst: &* mask` and `& mask` zero or merge masked-off lanes {#asm.19}; a masked store writes only selected elements {#asm.20}. Mask registers (`omr`, `kmask`) are 8-byte locals; the `k*` instructions operate on them, and compares, `ptestm` and `pmov*2m` write them {#asm.21}.
- Unsupported instructions are a compile error naming the instruction {#asm.22}. That includes x87 (Jai's `#asm` has none) {#asm.23}, `push`/`pop`, privileged instructions and some AVX-512 extensions (the list is in [jaic `#asm` blocks](../compiler/asm.md#not-supported), with the reason for each).
- F16C (`vcvtph2ps`, `vcvtps2ph` with its imm8 rounding mode), SHA (`sha1rnds4`, `sha1nexte`, `sha1msg1/2`, `sha256rnds2` with the implicit `xmm0` written as a last operand, `sha256msg1/2`) and GFNI (`gf2p8mulb`, `gf2p8affineqb`, `gf2p8affineinvqb`, legacy, VEX and masked EVEX forms) give the bits hardware gives {#asm.26}.
- A floating-point variable is a vector register: `addss x, y`, `sqrtsd x, y` and the other vector instructions read and write it, and `movd`/`movq` move its bits to or from an integer variable {#asm.28}. An integer instruction (`add x, y`) cannot take one, and `.x/.y/.z` sizes only go with vector instructions; both are compile errors that say so {#asm.29}.
- The SSE4.2 string compares `pcmpistri`, `pcmpestri`, `pcmpistrm` and `pcmpestrm` take the implicit registers as operands in the order `ecx` (or `xmm0`), then `eax`, `edx` for the explicit-length forms, the two strings and the control byte (`pcmpistri ecx, a, b, 0x0c`), and match the hardware for every control byte, flags included {#asm.30}. `mpsadbw` (SSE4.1 and AVX2) and the AVX-512 IFMA `pmadd52luq`/`pmadd52huq` are supported too {#asm.31}.
- `rcp*`/`rsqrt*` return exact results {#asm.24}. `cpuid` returns real leaf/subleaf data on x64 {#asm.25}; it cannot execute on an ARM interpreter host. XGETBV execution needs actual XSAVE/OSXSAVE support; the real-instruction implementation is being integrated separately. Literal-zero XGETBV lowering is an open limitation, not a feature query. The catalogue guards this instruction with real feature bits and checks enabled XCR0 state when available.
- A `name:` destination declares a register of the class the instruction writes: a general-purpose register for the vector instructions whose result is an integer (`pmovmskb.x found:, v`, `movmskps/pd`, `pextr*`, `extractps`, `cvt*2si`/`cvt*2usi`), a vector register otherwise {#asm.27}.
