# meow_hash

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `reg :: __reg`
- `MEOW_HASH_VERSION :: 5`
- `MEOW_HASH_VERSION_NAME :: "0.5/calico"`
- `MEOW_PAGESIZE :: 4096`
- `MEOW_PREFETCH :: 4096`
- `MEOW_PREFETCH_LIMIT :: 0x3ff`
- `MEOW_DUMP_STATE :: (T: string, r0: reg, r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, r6: reg, r7: reg) #expand`
- `MEOW_MIX_REG :: (r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, i1: reg, i2: reg, i3: reg, i4: reg) #expand`
- `MEOW_MIX :: (r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, ptr: *void, $OFFSET: s32) #expand`
- `MEOW_SHUFFLE :: (r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, r6: reg) #expand`
- `MeowU64From :: (Hash: reg, $INDEX: int) -> u64 #expand`
- `MeowU32From :: (Hash: reg, $INDEX: int) -> u32 #expand`
- `MeowBegin :: (Seed: [128] u8, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg) #expand`
- `MeowAbsorbBlocks :: (BlockCount: u64, Source: *void, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg) #expand`
- `MeowEnd :: (Len: u64, Source: *void, r1: reg, r3: reg, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg, Store128: *void) #expand`
- `MeowHash :: (Seed: [128] u8, Len: u64, Source: *void, h0: reg) #expand`
- `meow_state :: struct`
- `MeowBegin :: (State: *meow_state, Seed: [128] u8)`
- `MeowAbsorb :: (State: *meow_state, Len: u64, Source: *void)`
- `MeowEnd :: (State: *meow_state, h0: reg, Store128: *void = null) #expand`

- `MeowDefaultSeed: [128] u8 = u8.[     0x32, 0x43, 0xF6, 0xA8, 0x88, 0x5A, 0x30, 0x8D,     0x31, 0x31, 0x98, 0xA2, 0xE0, 0x37, 0x07, 0x34,     0x4A, 0x40, 0x93, 0x82, 0x22, 0x99, 0xF3, 0x1D,     0x00, 0x82, 0xEF, 0xA9, 0x8E, 0xC4, 0xE6, 0xC8,     0x94, 0x52, 0x82, 0x1E, 0x63, 0x8D, 0x01, 0x37,     0x7B, 0xE5, 0x46, 0x6C, 0xF3, 0x4E, 0x90, 0xC6,     0xCC, 0x0A, 0xC2, 0x9B, 0x7C, 0x97, 0xC5, 0x0D,     0xD3, 0xF8, 0x4D, 0x5B, 0x5B, 0x54, 0x70, 0x91,     0x79, 0x21, 0x6D, 0x5D, 0x98, 0x97, 0x9F, 0xB1,     0xBD, 0x13, 0x10, 0xBA, 0x69, 0x8D, 0xFB, 0x5A,     0xC2, 0xFF, 0xD7, 0x2D, 0xBD, 0x01, 0xAD, 0xFB,     0x7B, 0x8E, 0x1A, 0xFE, 0xD6, 0xA2, 0x67, 0xE9,     0x6B, 0xA7, 0xC9, 0x04, 0x5F, 0x12, 0xC7, 0xF9,     0x92, 0x4A, 0x19, 0x94, 0x7B, 0x39, 0x16, 0xCF,     0x70, 0x80, 0x1F, 0x2E, 0x28, 0x58, 0xEF, 0xC1,     0x66, 0x36, 0x92, 0x0D, 0x87, 0x15, 0x74, 0xE6 ]`
- `MeowMaskLen: [32] u8 = u8.[255,255,255,255, 255,255,255,255, 255,255,255,255, 255,255,255,255, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0]`
- `MeowShiftAdjust: [32] u8 = u8.[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]`

## Public type members

#### `meow_state` (struct)

- `RegisterFile: [256] u8`
- `TotalLengthInBytes: u64`
- `BufferLen: u32`
- `Buffer: [256] u8`
- `Pad: [32] u8`

## Behavioral contract and constraints

This module is a Jai port of Meow Hash and exposes the hash state, update, and finalization surface. It is a non-cryptographic hash; callers must follow the input-length/alignment requirements of the selected routine and provide enough readable input. No cryptographic or cross-version stability guarantee is made.

`reg` aliases Preload's distinct `__reg` operand type. In this module it denotes a compiler-managed 128-bit SIMD register operand for inline assembly, despite the source-level `__reg` declaration having the distinct `u16` type. It is not a 16-bit scalar value or a general-purpose storage representation. The public `MeowU64From` and `MeowU32From` expansions extract one indexed 64-bit or 32-bit lane from a register into the declared scalar result; their register arguments must remain SIMD register operands through expansion.

`MeowDefaultSeed`, `MeowMaskLen`, and `MeowShiftAdjust` are mutable initialized public arrays (`: Type = initializer`). The hash routines read these arrays when called, so element edits affect subsequent calls process-wide. They are not compile-time constants.
