# SIMD and `#asm`

`#asm` blocks with x64 scalar, string, SSE through AVX2, FMA, AES and AVX-512 instructions, including op-mask registers. jaic never emits machine code for them: each instruction lowers to ordinary IR, so `#asm` runs on any CPU, in the interpreter and in the browser. The instruction list and internals are in [jaic `#asm` blocks](../compiler/asm.md).

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

The first block leaves `x == 12` {#asm.4}; afterwards `a` holds `10, 40, 90, 160` {#asm.5}.

## Operands and sizes

- Operands are Jai variables (read and written in place), immediates, `[base + index*scale + disp]` memory and declared registers {#asm.1}.
- Registers aren't modelled. Pinning (`=== rax`) is accepted and ignored {#asm.2}, except that `syscall` reads the registers pinned to its arguments (Linux x86-64 only). A declared `gpr` is a zero-initialised 64-bit local {#asm.3}.
- Size comes from the `.b/.w/.d/.q` suffix, else the first Jai variable operand, else 64 bits {#asm.6}. 32-bit writes zero-extend {#asm.7}; 8- and 16-bit writes merge into the low bits {#asm.8}.
- Vector width comes from `.x/.y/.z` (16/32/64 bytes), else the block default (256 bits under an `AVX` modifier) {#asm.9}. A leading `v` (`vaddps`) is accepted {#asm.10}. Two operands mean `dst op= src`, three mean `dst = a op b` {#asm.11}.
- A feature modifier after `#asm` (`AVX`, `AVX2`, `AVX512F`, `BMI2`, ...) only chooses the default vector width and VEX zeroing of upper bytes {#asm.16}.
- A floating-point variable is a vector register: `addss x, y` and `sqrtsd x, y` read and write it, and `movd`/`movq` move its bits to or from an integer variable {#asm.28}. An integer instruction can't take one, and `.x/.y/.z` sizes go only with vector instructions; both are compile errors {#asm.29}.
- A `name:` destination declares a register of the class the instruction writes: a general-purpose register for vector instructions with an integer result (`pmovmskb.x found:, v`, `movmskps/pd`, `pextr*`, `extractps`, `cvt*2si`), a vector register otherwise {#asm.27}.

## Semantics

- Flags (CF, ZF, SF, OF, PF) are tracked within a block for `setcc`/`cmovcc` {#asm.12}. `lock_` memory operations use a compare-and-swap loop {#asm.13}.
- Implicit registers are explicit operands: `div hi, lo, d`, `rep_movs.q di, si, c`, `cqo d, a`, `cmpxchg16b d, a, [mem], c, b` {#asm.15}. Division by zero or overflow traps {#asm.14}.
- EVEX decorations: `[mem]!` broadcasts one element {#asm.17}; `v !z`/`!n`/`!d`/`!u` pick `cvtps2dq` rounding {#asm.18}; `dst: &* mask` and `& mask` zero or merge masked-off lanes {#asm.19}; a masked store writes only selected elements {#asm.20}. Mask registers (`omr`, `kmask`) are 8-byte locals that the `k*` instructions operate on; compares, `ptestm` and `pmov*2m` write them {#asm.21}.
- F16C (`vcvtph2ps`, `vcvtps2ph`), SHA (`sha1rnds4`, `sha256rnds2` with the implicit `xmm0` as a last operand, and friends) and GFNI (`gf2p8mulb`, `gf2p8affineqb`, `gf2p8affineinvqb`) give the bits hardware gives {#asm.26}.
- The SSE4.2 string compares `pcmpistri`, `pcmpestri`, `pcmpistrm`, `pcmpestrm` take implicit registers as operands: `ecx` (or `xmm0`), then `eax`, `edx` for explicit-length forms, the two strings and the control byte (`pcmpistri ecx, a, b, 0x0c`). They match hardware for every control byte, flags included {#asm.30}. `mpsadbw` and AVX-512 IFMA `pmadd52luq`/`pmadd52huq` are supported too {#asm.31}.
- `rcp*`/`rsqrt*` return exact results {#asm.24}. `cpuid` returns real leaf data on x64 only {#asm.25}; it can't run on an ARM interpreter host.
- Unsupported instructions are a compile error naming the instruction {#asm.22}: x87 (Jai's `#asm` has none) {#asm.23}, `push`/`pop`, privileged instructions and some AVX-512 extensions (see the [compiler page](../compiler/asm.md#not-supported)).

## How to change it

Add a mnemonic in `sema/asm.rs`, `asm/scalar.rs`, `asm/mask.rs`, or `lookup_simd` in `asm/simd.rs`, plus a test case. Vectors are 64-byte locals processed through a scratch buffer so aliased operands behave as on hardware: never write the destination before every source is read. Tests: `tests/compiler/regressions/lang-asm.jai`, `asm-vector-instructions.jai`, `asm-avx512-masks.jai`.
