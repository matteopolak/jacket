# SIMD lowering (LLVM backend)

## What it is

The module `crates/jaic-llvm/src/simd/` turns the SIMD IR intrinsics (`ir::Intrinsic::ByteLookup(width)`, `SignMask(width)`; user-facing description in [intrinsics](../language/intrinsics.md#native-lowering-of-the-simd-intrinsics)) into LLVM IR, choosing the best instruction the target machine has. Every target computes the same result; the choice only affects speed.

## How it works

```
lower.rs  Intrinsic::ByteLookup(w) -> simd::byte_lookup(&cx, w, dst, table, idx)
simd/
  mod.rs        Cx (builder helpers), one dispatch function per intrinsic
  features.rs   Feature / FeatureSet / Target: what the target has
  x86.rs        pshufb: SSSE3 (16 B), AVX2 (32 B), AVX-512BW (64 B)
  aarch64.rs    NEON tbl1
  wasm.rs       simd128 swizzle
  generic.rs    portable per-lane code; the fallback and the reference semantics
```

`lower.rs` only builds a `simd::Cx` (LLVM context, builder, module, `Target`) and calls in. Each dispatch function in `mod.rs` tries the backends in order. A backend returns `Ok(true)` once it has emitted the operation, and `Ok(false)` when its architecture or features do not apply, so `generic` always ends the chain.

### Feature resolution (`features.rs`)

`Target::resolve(arch, cpu, features)` is called from `lib.rs` with the CPU name and feature string of the LLVM `TargetMachine` actually used for codegen (so the baseline CPU, `"native"` and user overrides are all seen). The result is a typed `FeatureSet` (a bit set of `Feature`: x86 `Sse2 .. Avx512vbmi`, aarch64 `Neon/Sve/Sve2`, wasm `Simd128/RelaxedSimd`):

1. The CPU name gives a starting set.
2. `+x` in the feature string adds `x` and what it implies (`avx2` brings `avx`, `sse4.2`, `sse4.1`, `ssse3`, `sse2`).
3. `-x` removes `x` and everything that implies it (`-ssse3` also drops AVX2).
4. Features of another architecture, and names the module does not know (`+bulk-memory`), are ignored.

LLVM's C API cannot resolve what a named CPU implies, and only `LLVMGetHostCPUFeatures` (used for `"native"`, which arrives as a complete `+x,-y` list that the steps above apply on top of the CPU name) reports features at all. So step 1 is the one table in the module: `cpu_features`, keyed by LLVM CPU names, listing only the features the module names. Unknown CPU names get the architecture baseline (SSE2, NEON, nothing for wasm): correct but not the fastest. Behaviour that must hold: the macOS x86-64 baseline `core2` has SSSE3, the Linux/Windows `x86-64` baseline does not.

### Choices today

| Intrinsic | x86-64 | aarch64 | wasm | otherwise |
| --- | --- | --- | --- | --- |
| `byte_lookup` | `llvm.uadd.sat` 0x70 then `pshufb`: AVX-512BW for width 64, AVX2 for width >= 32, SSSE3 otherwise | `llvm.aarch64.neon.tbl1` | `llvm.wasm.swizzle` with `simd128` | per-lane extract/select/insert |
| `sign_mask` | `generic` (`icmp slt`, bitcast, becomes `pmovmskb`) | `generic` | `generic` | `generic` |

`vpermb` (AVX-512 VBMI) is not needed: 512-bit `pshufb` already does a per-16-byte-lane lookup, which is what `byte_lookup` means.

## How to change it

Add an instruction set to an existing intrinsic:
1. If the features are new, add `Feature` variants in `features.rs` (`ALL`, `name`, `family`, `implies`) and the CPUs that have them in `cpu_features`, plus a test.
2. Add the path in the architecture's file, guarded by `cx.target.has(..)`, returning `Ok(false)` when it does not apply. Reuse `Cx::map_blocks` (loads each block, stores the result), `Cx::call` (LLVM intrinsic by name), `Cx::repeat_table` and `Cx::splat`.
3. Put the more specific path first. `generic` stays last.

Add an intrinsic: add the `ir::Intrinsic` variant and its sema/interpreter parts as for any intrinsic ([intrinsics](../language/intrinsics.md)), then in `simd/`: write `generic::<name>` first (the portable form), a dispatch function in `mod.rs`, and a one-line call in `lower.rs`'s `intrinsic`. Then add backends.

Add an architecture family: a `Family` variant and `Family::of` mapping, its features, a new file and a line in each dispatch function.

Gotchas:
- An LLVM intrinsic whose name is wrong is reported at lowering time (`LLVM intrinsic '...' is unavailable`), but one the target cannot select fails only in codegen: check with `-emit_ir` and by building an object.
- The IR for every target is produced on any host, so cross-target checks need no emulator. The metaprogram options set the CPU and features for the program's workspace, and `jaic m.jai -target <triple> -emit_ir x.ll` selects the triple.
- SVE has no backend yet (`aarch64.rs` says where it would go); scalable vectors need a different loop than `map_blocks`.

## Configuration

- `Build_Options.llvm_options.target_system_cpu` and `target_system_features` (the `-mcpu`/`-mattr` equivalents; empty CPU means the triple's baseline, `"native"` the build machine). `cpu_and_features` and `baseline_cpu` in `crates/jaic-llvm/src/lib.rs` fill them in.
- Wasm always gets `+bulk-memory`; `+simd128` turns on the swizzle.

## Dependencies

- `inkwell` (builder, intrinsic lookup by name), through `crate::lower::Error` for errors.
- `jaic::abi::Arch` for the architecture family.
- Tests: `cargo test -p jaic-llvm simd` (feature resolution) and `tests/stdlib/intrinsic-byte-lookup.jai` (semantics, interpreted and native).
