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

For an external partial lane, `finish` builds the 16-byte shuffle window from residual bytes only, zero outside them. A window inside the object is loaded directly; one that runs past the end is made from a load of the object's last 16 bytes shifted down with `MeowTailShift`; objects under 16 bytes are copied into a zeroed local array. The old 4096-byte alignment rule remains solely to select the legacy shuffle window and mutable table index; it does not authorize any caller-memory load. No readable allocation or page padding is required.

Default digests, seeds and stored states are unchanged. `MeowMaskLen` and `MeowShiftAdjust` remain mutable: valid residual bytes retain the legacy transformations. Selecting a byte outside the residual object now yields zero. Previous results that depended on neighboring memory are not guaranteed. The streamed overload still loads within its own 256-byte buffer, so a mutated mask may expose stale buffered bytes; default masks exclude them. Portable hashing is an independent default-table oracle, not an oracle for mutated native tables.

## Changing and verifying the implementation

Keep tail construction in `stdlib/internal/hash/meow/finalize.jai` separate from mixing. Any wider load must prove its entire interval belongs to the residual object, including a direct finalizer whose earlier blocks are in another allocation. Preserve shuffle and mask ordering. Do not replace object bounds with page mapping checks.

`stdlib/meow_hash/tests/exact.jai` uses direct C `malloc`/`free` to cover exact heap extents (including aligned allocator slack that Basic may otherwise reserve), plus stack inputs, all short lengths and alignments, direct finalization, streaming splits, dirty states and protected pages on POSIX. Existing binary fixtures preserve digest and stored-state compatibility, including table mutations. `Extensions/Meow_Portable` tests compare independent scalar results. The diagnostic driver selects an AES-capable x64 host; an ARM portable pass does not establish native x64 safety.

After building `jaic`, run `PYTHONPATH=tools python3 tools/meow/validate.py` on Linux x64. It saves commands, exit codes and output under `target/meow-evidence`, establishes the historical ASan overflow at the native hash callsite (`#expand` instructions inherit callsite debug locations), then runs native suites at O0/O2 with and without ASan. `CARGO_TARGET_DIR` selects the compiler build directory. Use the repository sanitizer policy (`ASAN_OPTIONS=detect_leaks=0:halt_on_error=1`): these suites check invalid accesses, while process-lifetime allocations are handled separately by the general harness. No timing results come from sanitizer builds.

The implementation depends on Basic allocation/copy and compiler SIMD lowering. AES operations may be software-lowered: inspect generated instructions before describing measurements as hardware AES throughput. ASan must instrument the lowered SIMD memory loads, not merely be linked into the executable. The driven boundary harness verifies a poisoned prefix address, a readable first input byte and a poisoned first suffix byte inside its own allocation. ASan shadow granularity can leave bytes immediately before an unaligned input readable, so this does not establish byte-exact prefix coverage at every offset. The suffix check establishes the exact logical end independently of allocator rounding. POSIX guard tests use the actual OS page size and complement object-bound checks.

## Comparing performance

`tools/meow/benchmark.py` builds matched binaries with one compiler and copied stdlibs, replacing only the baseline finalizer from the pinned revision. Inputs include padding for the old implementation. On a quiet x64 host:

```sh
PYTHONPATH=tools python3 tools/meow/benchmark.py prepare --compiler target/debug/jaic --out /tmp/meow-benchmark
PYTHONPATH=tools python3 tools/meow/benchmark.py run --out /tmp/meow-benchmark --execution-label 'native Linux x64; inspect instruction lowering'
```

The default covers 16 boundary lengths, four offsets, two initial seeds and five one-shot/disjoint/edit/streaming modes. Each timed case hashes 5,000 times below 1 KiB, 2,000 times at 1 KiB, or four times at 1 MiB. One validated warmup pair precedes five interleaved measured pairs. `--smoke`, `--full`, `--warmups`, `--pairs` and `--timeout` control coverage and limits. Use `--full --timeout 600` to allow more time for the larger matrix. Per-iteration opaque source/seed pointers keep hash work inside the loop, including empty inputs; their compiler-barrier overhead is included in both variants.

Keep the output directory's metadata, raw measurements and failures. Both digest-lane aggregates must match, but this does not replace exhaustive correctness tests. Disjoint data is not necessarily cache-cold. Short timings, hosted contention and Rosetta limit interpretation; report execution mode and variability, and inspect generated code before making hardware AES claims. Git must retain the pinned baseline. See [benchmark methods](../tools/benchmarks.md).
