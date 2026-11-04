# MeowHash input safety

Native `meow_hash` accepts exact input extents at arbitrary alignments. One-shot hashing reads `Len` bytes, streaming updates read each chunk's `Len` bytes, and register finalization reads only the `Len & 255` residual bytes at its supplied pointer. `MeowAbsorbBlocks` requires `BlockCount * 256` readable bytes. Null is valid for zero-length input. Seeds and non-null stored-state outputs still require 128 bytes.

```jai,ignore
bytes := NewArray(3, u8);
defer free(bytes.data);
bytes[0] = 1; bytes[1] = 2; bytes[2] = 3;
#asm { pxor hash:, hash; }
MeowHash(MeowDefaultSeed, 3, bytes.data, hash);
low := MeowU64From(hash, 0);
```

## Read boundaries and tables

Each mixing lane reads exactly 32 bytes: four 16-byte loads at offsets 0, 1, 15 and 16. Eight lanes therefore read one complete 256-byte block. Finalization processes only complete 32-byte residual lanes and, when present, the complete 16-byte piece immediately preceding the partial lane.

For an external partial lane, `finish` reconstructs a zero-initialized 16-byte shuffle window using individually checked residual byte indices. The SIMD load reads that local array. The old 4096-byte alignment rule remains solely to select the legacy shuffle window and mutable table index; it does not authorize any caller-memory load. No readable allocation or page padding is required.

Default digests, seeds and stored states are unchanged. `MeowMaskLen` and `MeowShiftAdjust` remain mutable: valid residual bytes retain the legacy transformations. Selecting a byte outside the residual object now yields zero. Previous results that depended on neighboring memory are not guaranteed. The streamed overload still loads within its own 256-byte buffer, so a mutated mask may expose stale buffered bytes; default masks exclude them. Portable hashing is an independent default-table oracle, not an oracle for mutated native tables.

## Changing and verifying the implementation

Keep tail construction in `stdlib/internal/hash/meow/finalize.jai` separate from mixing. Any wider load must prove its entire interval belongs to the residual object, including a direct finalizer whose earlier blocks are in another allocation. Preserve shuffle and mask ordering. Do not replace object bounds with page mapping checks.

`stdlib/meow_hash/tests/exact.jai` uses direct C `malloc`/`free` to cover exact heap extents (including aligned allocator slack that Basic may otherwise reserve), plus stack inputs, all short lengths and alignments, direct finalization, streaming splits, dirty states and protected pages on POSIX. Existing binary fixtures preserve digest and stored-state compatibility, including table mutations. `Extensions/Meow_Portable` tests compare independent scalar results. The diagnostic driver selects an AES-capable x64 host; an ARM portable pass does not establish native x64 safety.

After building `jaic`, run `PYTHONPATH=tools python3 tools/meow/validate.py` on Linux x64. It saves commands, exit codes and output under `target/meow-evidence`, establishes the historical ASan overflow at the native hash callsite (`#expand` instructions inherit callsite debug locations), then runs native suites at O0/O2 with and without ASan. `CARGO_TARGET_DIR` selects the compiler build directory. Use the repository sanitizer policy (`ASAN_OPTIONS=detect_leaks=0:halt_on_error=1`): these suites check invalid accesses, while process-lifetime allocations are handled separately by the general harness. No timing results come from sanitizer builds.

The implementation depends on Basic allocation/copy and compiler SIMD lowering. AES operations may be software-lowered: inspect generated instructions before describing measurements as hardware AES throughput. ASan must instrument the lowered SIMD memory loads, not merely be linked into the executable. POSIX guard tests use the actual OS page size and complement object-bound checks.
