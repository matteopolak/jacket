# jaic `#asm` blocks

## What it is

Jai's `#asm { ... }` inline assembly. jaic never emits machine code for it: each x64 instruction is parsed into an AST and lowered to ordinary IR operations on the Jai variables used as operands. A block therefore behaves the same in the interpreter, the browser and the LLVM backend, on any host CPU; an arm64 Mac runs x64 `#asm` correctly. General-purpose, BMI/ADX, string, division, SSE through AVX2, FMA, AES/PCLMUL and common AVX-512 instructions, including op-mask (`k`) registers, are supported. Anything else is rejected with an error naming the instruction.

The user-facing summary is [SIMD and `#asm`](../language/simd-asm.md); this page is the reference.

## How it works

- **Parsing** (`parser/asm.rs`) is mnemonic-agnostic and produces `ast::AsmBlock` (`ExprKind::Asm`): feature
  modifiers (`#asm AVX2 {`), then `;`-separated items. An item is an instruction (`mnemonic[.size|?size] operands`) or a
  declaration/pin statement (`x: gpr;`, `t: gpr === a;`, `x === a;`). Operands are a variable/constant expression, a
  memory operand `[base + index*scale +/- disp]` (each term a unary expression, so `*x` and `(size_of(u64))` work) or
  an inline declaration (`tmp:`, `tmp: gpr === 15`). `===` lexes as `==` `=` and is recognised by adjacency.
- **Lowering** is split by instruction family. `asm_inst` in `sema/asm.rs` tries, in order:
  1. `lookup_op` (`sema/asm.rs`): the integer core (mov, ALU, shifts, bit ops, setcc/cmovcc, atomics).
  2. `asm_scalar_inst` (`sema/asm/scalar.rs`): division, widening, double shifts, BMI/ADX, CRC32, string
     instructions, `cmpxchg8b/16b`, flags transfer, hints.
  3. `asm_mask_inst` (`sema/asm/mask.rs`): the `k*` op-mask instructions.
  4. `asm_vec_inst` (`sema/asm/vec.rs`): vector instructions; `lookup_vec` falls back to `simd::lookup_simd`
     (`sema/asm/simd.rs`) for the larger SIMD set.
- **Registers.** Variables are read and written in place through their stack slots. Declared registers are locals
  added to the *enclosing* scope (blocks are not scopes; macros see them). They start at zero. Classes: `gpr` (8
  bytes), `vec` (64 bytes; `xmm/ymm/zmm` by width), `str` (MMX, a `vec` used at 8 bytes), `omr`/`kmask` (an 8-byte
  op-mask). An inline declaration `name:` takes its class from the position: a gpr in scalar instructions, a `vec` in
  vector ones, an `omr` in `k*` instructions and as the destination of a compare-into-mask. Registers are not
  modelled: `===` pins are parsed and ignored. A declared register's value only means something inside the block
  that declares it; reload values in each block.
- **Operation size**: explicit suffix (`.b/.w/.d/.q`, `.8/.16/.32/.64`, or `?T` / `?BITS`), else the size of the first
  Jai variable operand, else 64 bits. 32-bit writes zero-extend into a 64-bit destination; 8/16-bit writes merge.
- **Implicit registers are explicit operands**, in Jai's order:

  | Instruction | Form | Meaning |
  | --- | --- | --- |
  | `mul`/`imul` | `mul hi, lo, src` | `hi:lo = lo * src` |
  | `div`/`idiv` | `div hi, lo, d` (`div.b ax, d`) | `lo = hi:lo / d`, `hi = remainder` |
  | `cbw/cwde/cdqe` | `cdqe a` | sign-extend the low half of `a` |
  | `cwd/cdq/cqo` | `cqo d, a` | `d` = sign of `a` |
  | `mulx` | `mulx hi, lo, src, d` | `hi:lo = d * src`, flags untouched |
  | `rep_movs.q` | `rep_movs.q di, si, c` | copy `c` elements, advance `di`, `si`, clear `c` |
  | `rep_stos.d` | `rep_stos.d di, a, c` | fill |
  | `rep_lods.b` | `rep_lods.b a, si, c` | load |
  | `repe_cmps.b` | `repe_cmps.b di, si, c` | flags from `[si] - [di]` |
  | `repne_scas.b` | `repne_scas.b di, a, c` | flags from `a - [di]` |
  | `cmpxchg16b` | `cmpxchg16b d, a, [mem], c, b` | compare `d:a` with 16 bytes, store `c:b` |
  | `xlat` | `xlat table, a` | `al = [table + al]` |
  | `cmpxchg` | `cmpxchg dest, src, acc` | |

  String instructions need a size suffix and register operands; without a `rep*` prefix they run once.
- **Flags** CF/ZF/SF/OF/PF are IR values tracked per block (unset flags read as 0); `setcc`/`cmovcc`
  code evaluates conditions from them, including `p/pe/np/po`. PF is kept lazily as the low result byte. The
  direction flag is tracked statically: `std` makes later string instructions in the same block walk downwards,
  `cld` resets it, and each block starts with DF clear (as the ABI guarantees). `lahf`/`sahf` move SF/ZF/PF/CF.
- **Division** traps (`#DE` on hardware: `Intrinsic::Trap`) on a zero divisor or a quotient that does not fit, like
  hardware. 128-by-64 division uses a fast path when the high half is zero, otherwise a restoring shift-subtract loop.
- **Atomics.** `lock_` read-modify-write instructions on memory, and `xchg` with memory, use a compare-and-swap loop
  (`Intrinsic::CompareAndSwap`). `lock_ cmpxchg16b` (and `8b`) is atomic with respect to other `lock_ cmpxchg16b`
  users only: it takes a global spin lock (`Compiler::asm_pair_lock`), since the IR has no 128-bit CAS.
- **Vectors** (`vec.rs`, `simd.rs`). A `vec` register is a 64-byte local; each instruction runs lane by lane on memory
  into a scratch buffer that is then copied to the destination, so aliasing operands work. Size: `.x/.y/.z`
  (16/32/64 bytes; `.q` = 8 for MMX forms), else 32 in a block with an AVX feature, 64 with AVX-512, otherwise 16.
  A leading `v` is accepted (`vpaddd`). Two operands mean `dst op= src`, three `dst = a op b`. With an AVX feature, a
  register write clears the bytes above the written size (VEX); otherwise they are kept. A Jai variable operand is
  memory at its address (integers act as general-purpose registers for `movd`/`movq`/`movmsk*`/`pextr*`).
- **Masks.** An `omr` register is an 8-byte local. `dst: &k` (merge) / `dst: &*k` (zero) apply after the instruction
  ran; a masked store to memory writes only selected elements. Compares (`pcmp*`, `cmpps`) whose destination is an
  `omr` write a bit per lane, and a `&k` on them ANDs the result. Instructions that consume the mask themselves
  (compress, expand, blendm, gather, scatter) handle it directly; EVEX gather and scatter clear the mask, as hardware
  does.
- **IR** additions: `Intrinsic::{Popcount, Ctlz, Cttz, Bswap}` (value, width in bits) and `Intrinsic::Fma` (three
  `F64` bit patterns, single rounding). They are implemented in `interp/mod.rs` and in `jaic-llvm`
  (`llvm.ctpop/ctlz/cttz/bswap/fma`). AES S-box and inverse tables are generated in Rust into one read-only global
  (`Compiler::asm_aes_tables`) on first use.
- `__reg` (alias of `Code`) macro parameters bind to the caller's operand by name.

### Supported instructions

Scalar: mov, movnti, movzx/movsx, movbe, lea, xchg, xadd, cmpxchg, cmpxchg8b/16b, add, sub, adc, sbb, and, or, xor,
cmp, test, inc, dec, neg, not, shl/sal/shr/sar/rol/ror, rcl/rcr, shld/shrd, bt/bts/btr/btc, bsf, bsr, popcnt, lzcnt,
tzcnt, bswap, mul, imul, div, idiv, cbw/cwde/cdqe, cwd/cdq/cqo, BMI1/2 (andn, bextr, blsr/blsi/blsmsk, bzhi, pdep,
pext, shlx/shrx/sarx, rorx, mulx), adcx/adox, crc32 (`crc32d`/`crc32q`, CRC-32C), setcc, cmovcc, clc/stc/cmc,
cld/std, lahf/sahf, xlat, movs/stos/lods/cmps/scas with `rep_`/`repe_`/`repz_`/`repne_`/`repnz_`, nop/pause/fences,
prefetch*/clflush*/clwb/cldemote (no-ops that still evaluate the address), int3 (`int 3`), rdtsc/rdtscp (a monotonic
counter), rdrand/rdseed, rdpid (0), cpuid and xgetbv (all outputs 0), stmxcsr (0x1f80), ldmxcsr (ignored).

Op-mask: kmov, kand, kandn, kor, kxor, kxnor, knot, kadd, kshiftl, kshiftr, ktest, kortest (each with b/w/d/q), and
kunpckbw/wd/dq.

Vector (any `ps/pd/ss/sd` or `b/w/d/q` variant that exists on hardware):

- Moves: `mov{u,a}{ps,pd}`, `movdq{u,a}{,8,16,32,64}`, `lddqu`, `movnt*`, `movd/movq`, `movss/movsd`,
  `movhps/movlps/movhpd/movlpd`, `movhlps/movlhps`, `movddup/movshdup/movsldup`, broadcasts (`broadcastss/sd`,
  `pbroadcastb/w/d/q`, `broadcast{i,f}{128,32x2,32x4,64x2,32x8,64x4}`, `pbroadcastmb2q/mw2d`).
- Integer: `padd/psub` (also saturating `s/us`), `pmull`, `pmulh{,u,rs}w`, `pmuludq/pmuldq`, `pmaddwd`, `pmaddubsw`,
  `psadbw`, `pavgb/w`, `pmin/pmax` (signed/unsigned), `pabs`, `psign`, shifts by immediate/register/vector
  (`psll/psrl/psra`, `psllv/psrlv/psrav`, `pslldq/psrldq`), rotates (`prol/pror/prolv/prorv`), horizontal
  (`phadd/phsub{w,d,sw}`, `phminposuw`), `pcmpeq/pcmpgt` and AVX-512 `pcmp{,u}{b,w,d,q}` with a predicate,
  `vpopcnt`, `plzcnt`, `pconflict`, `pternlog`, `pdpbusd/pdpwssd` (VNNI), `ptest`, `ptestm/ptestnm`.
- Float: `add/sub/mul/div/min/max/sqrt`, `rcp/rsqrt` (and the `14`/`28` forms), `round`/`rndscale`, `cmp` with all
  32 predicates, `comis/ucomis`, `hadd/hsub`, `addsub`, `dpps/dppd`, FMA
  (`f[n]m{add,sub}{132,213,231}{ps,pd,ss,sd}`, `fmaddsub/fmsubadd`).
- Shuffles: `pshufd/pshufhw/pshuflw`, `pshufb`, `shufps/shufpd`, unpacks (`unpck{l,h}{ps,pd}`, `punpck{l,h}{bw,wd,dq,qdq}`),
  `palignr`, `valignd/q`, blends (`blendps/pd`, `pblendw/d`, `blendv*`, `pblendvb`, `blendmps/pd`, `pblendm*`),
  permutes (`permps/pd`, `perm{d,q,w,b}`, `permilps/pd`, `perm2i128/f128`, `shuf{i,f}{32x4,64x2}`,
  `permt2*`, `permi2*`), inserts/extracts (`insert/extract{i,f}{128,32x4,64x2,32x8,64x4}`, `insertps`, `extractps`,
  `pinsr*`, `pextr*`), `compress/expand` (`ps/pd`, `pcompress*`, `pexpand*`) to registers or memory.
- Conversions: `cvt{t,}{ps,pd,ss,sd}2{dq,udq,qq,uqq,si,usi}`, `cvt{dq,udq,qq,uqq}2{ps,pd}`, `cvtsi2ss/sd`,
  `cvtusi2ss/sd`, `cvtps2pd/cvtpd2ps`, `cvtss2sd/cvtsd2ss`, packs (`packss/packus`), sign/zero extension
  (`pmovsx/pmovzx`), down-conversion (`pmov`, `pmovs`, `pmovus` to registers or memory), `pmovm2*`, `pmov*2m`,
  `movmskps/pd`, `pmovmskb`.
- Memory: gathers (`gather*`, `pgather*`; AVX2 vector-mask and AVX-512 `&k` forms), scatters (`scatter*`,
  `pscatter*`), `maskmovps/pd`, `pmaskmovd/q`.
- Crypto: `aesenc/aesenclast/aesdec/aesdeclast/aesimc/aeskeygenassist`, `pclmulqdq` (and the `lqlq`-style aliases).
- `zeroupper`.

EVEX decorations: `[mem]!` (embedded broadcast), `!z/!n/!d/!u` rounding on `cvtps2dq`, `&k` / `&*k` masks.

### Not supported

Compile error `unsupported #asm instruction 'x'`:

- **x87** (`fld`, `fadd`, ...): Jai's `#asm` has no x87 instructions or registers (the `str` class is MMX), so there
  is nothing to accept.
- `syscall`, `int n` other than 3, `push`/`pop`, `call`/`jmp`/`jcc` (Jai `#asm` has no labels), I/O and privileged
  instructions, `pcmpestri/pcmpistri`, F16C (`cvtph2ps/cvtps2ph`), `getexp/getmant/scalef/fpclass/range/reduce`,
  GFNI, SHA, `mpsadbw`, `pmadd52*`, BF16/FP16 arithmetic, AMX.

Known differences from hardware: `rcp*/rsqrt*` return the exact result (hardware approximates to 12 or 14 bits);
single-precision FMA computes in double and rounds once more (exact except in rare double-rounding cases); MXCSR is
not modeled (round-to-nearest, no exceptions); `cpuid`/`xgetbv` report no features, so runtime dispatch on them picks
the baseline path.

## How to change it

- New integer instruction: an `Op` in `lookup_op` plus an arm in `asm_inst` (`sema/asm.rs`), or an `XOp` in
  `scalar.rs` when it has implicit operands or a loop. Reuse `asm_read`/`asm_write` (size-aware operand access),
  `rmw_begin`/`rmw_end` (atomic when `lock_`) and `asm_alu` for flag-setting arithmetic.
- New op-mask instruction: a `KOp` and `lookup_kop` entry in `mask.rs`.
- New vector instruction: a `VOp`/`Lane` in `vec.rs` for the simple lane-wise forms, otherwise an `SOp` in `simd.rs`:
  add the mnemonic to `lookup_simd`, give it an element size (`elem_size`, used for masking), a destination class
  (`dst_class`) if it writes a mask or gpr, and an arm in `asm_simd`. Helpers: `simd_args` (sources), `simd_out`
  (store with masking), `load_lane`/`store_lane`, `when` (conditional block).
- New condition code: `Cond` + `lookup_cond` + `eval_cond`.
- Tests: add a case to the matching `tests/stdlib/asm-*.jai`. For instructions the host can run, record the expected
  value on real hardware (an x86-64 C program with intrinsics or inline asm, run natively or under Rosetta with
  `ROSETTA_ADVERTISE_AVX=1` for AVX2/FMA/BMI) rather than deriving it from the implementation. AVX-512 has no
  hardware here, so `asm-avx512-masks.jai` computes expectations with plain Jai code.
- Gotchas:
  - A `Val` defined inside a loop or CAS retry is only valid after the loop exit dominates it; keep loop-carried
    values in stack slots (`udiv128`, `asm_string`).
  - The IR `select` is arithmetic on both arms, so a float-to-int conversion must clamp before converting (an
    out-of-range `FToS` is poison in LLVM even on the unselected arm).
  - x86 masks shift counts first (`& 31`/`& 63`); IR shifts are total, so only the masking is needed.

## Configuration

None. Feature modifiers after `#asm` are validated against `FEATURES` in `sema/asm.rs` (every CPUID feature-flag
name of `Machine_X64`, plus older spellings such as `AVX512VBMI`). They only choose the default vector width
(`AVX*` = 256 bits, `AVX512*` = 512 bits). `#if CPU == .X64` still selects Jai-level fallbacks as usual. Jai has no
arm64 `#asm`, so there is nothing to add for arm64; x64 blocks simply run there through the IR.

## Dependencies

`ir` (builder, `Intrinsic::{CompareAndSwap, CycleCounter, Pause, DebugBreak, Trap, Popcount, Ctlz, Cttz, Bswap,
Fma}`), `sema::calls` (macro expansion, `Code` parameters), `interp` and `jaic-llvm` for the intrinsics. Tests:
`tests/stdlib/lang-asm.jai`, `asm-vector-instructions.jai`, `asm-evex-decorations.jai`, `asm-scalar-extended.jai`,
`asm-simd-extended.jai`, `asm-avx512-masks.jai` (swept by `tools/jaic-sweep.py stdlib`; the last three also built and
run natively by `asm_instructions_run_natively` in `crates/jaic-cli/tests/native.rs`) and the `asm_*` parser tests.
