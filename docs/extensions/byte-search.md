# Byte_Search (`Extensions/Byte_Search`)

## What it is

Vectorized byte and substring search (the job of `memchr`/`memmem`), built on [SIMD](simd.md). Every search is linear in the text, even with adversarial needles, and returns a position in the whole text or -1.

```jai
Byte_Search :: #import "Extensions/Byte_Search";

at := Byte_Search.find(text, #char "\n");               // first newline
at  = Byte_Search.find_any(text, #char ",", #char ";"); // first of 2 or 3 bytes
at  = Byte_Search.find_last(text, #char "/");
n  := Byte_Search.count(text, #char "\n");
at  = Byte_Search.find(text, "needle");

finder := Byte_Search.make_finder("needle");            // analyse a needle once
at = Byte_Search.find(*finder, text, start = at + 1);

digits := Byte_Search.make_byte_set(cast([] u8) "0123456789");
at = Byte_Search.find(*digits, text);                   // first byte in the set
at = Byte_Search.find_pair(*digits, *dash, 4, text);    // text[i] in a and text[i+4] in b
```

Also `find_last_any`, `contains(*Byte_Set, byte)`, `find_triple(*a, *b, *c, gap_2, gap_3, text, start)`.

## How it works

- Byte searches come from one generator (`forward_search`/`backward_search`, `#insert`) taking a mask expression and a scalar test: four 16-byte chunks per iteration OR-ed (one branch per 64 bytes), branch-free choice of the hitting chunk, then single chunks, then an overlapping reread of the last partial chunk; texts under 16 bytes use a scalar loop. Speed matches Rust's `memchr` for single bytes and is faster for multi-byte sets, `count` and substring search.
- `Byte_Set` uses nibble tables (high nibbles get one of 8 bucket bits; two table lookups and an AND test 16 bytes). More than 8 high nibbles (`wide`) falls back to a scalar loop; a one-byte set (`single`) uses a plain compare, about 3x faster for `find_pair`/`find_triple`.
- `count` subtracts each mask from u8 counters and flushes every 255 chunks.
- Substring search: `make_finder` picks the needle's two rarest bytes by a rough frequency table (`rarity`); the vector loop checks both at their offsets and verifies survivors with `memcmp`. When candidates keep failing (`misses > 8 && misses * 32 > scanned`) it switches to Two-Way (Crochemore-Perrin), linear time and constant space; `make_finder` computes the factorization for both directions.

- **Two widths.** The searches that compare bytes (`find`, `find_any`, `find_last`, `find_last_any`, `count`, and the `Finder` searches) exist as a 16-byte version for every processor and a 32-byte version compiled with `#target_features "avx2"`. `dispatch`/`wrappers` in the module generate the pair, a table and the public procedure from one body; the first call picks the version with `pick_target` ([choosing code by CPU](simd.md#choosing-code-by-cpu)), so no call checks the processor. The 32-byte version hands texts under 64 bytes to the 16-byte one. `Byte_Set` searches stay 16 bytes wide: their table lookups (`pshufb`) cannot use the wider registers yet.

## How to change it

- **New byte search:** add an `#insert #run dispatch(...)` line next to the others in `#scope_module`, with `forward_search`/`backward_search` for the body: a mask expression over `chunk` and a scalar test over `c`; the generated code expects the local variables those expressions name (the setup string declares `a := byte`). Then a one-line public procedure that calls it.
- **Keep vector loops `no_inline`.** SIMD lane operations are loops LLVM vectorizes after inlining; inlined into a big caller they can degrade to byte loads (`find_pair` went from 4 GB/s to 1 GB/s). Every procedure holding such a loop is `no_inline`.
- **Rarity table:** only a rough order is needed; change `make_rarity`.
- **Fallback threshold:** lowering `misses * 32 > scanned` moves adversarial needles to Two-Way sooner; raising it keeps typical text in the faster vector loop.
- Tests: `tests/variants.jai` runs the 16- and 32-byte version of every dispatched search against naive loops (it `#load`s the module to reach them; the 32-byte one runs on any machine except x64 without AVX2). `tests/search.jai` checks every procedure against naive loops (random texts at every start/end, small alphabets, random sets with pairs/triples at every gap, periodic needles, a 2 MB linear-time check). Run with `PYTHONPATH=tools python3 tools/stdlib/stdlib_runtime.py --filter Byte_Search`.

## Configuration and dependencies

None; widths are 16 bytes and, on AVX2 x64, 32. Depends on [SIMD](simd.md) (`byte_lookup`, `sign_mask` intrinsics) and `Basic`. Used by [Regex](regex.md) for match skipping.
