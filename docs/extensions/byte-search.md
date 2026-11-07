# Byte_Search

Vectorized byte and substring search (the job of `memchr`/`memmem`), built on [SIMD](simd.md). Every search is linear in the text, even with adversarial needles, and returns a position in the whole text or -1.

```jai
Byte_Search :: #import "Extensions/Byte_Search";

at := Byte_Search.find(text, #char "\n");
at  = Byte_Search.find_any(text, #char ",", #char ";");
n  := Byte_Search.count(text, #char "\n");
at  = Byte_Search.find(text, "needle");

finder := Byte_Search.make_finder("needle");      // analyse a needle once
at = Byte_Search.find(*finder, text, start = at + 1);

digits := Byte_Search.make_byte_set(cast([] u8) "0123456789");
at = Byte_Search.find(*digits, text);
```

## How it works

- Byte searches come from one generator (`forward_search`/`backward_search`) taking a mask expression and a scalar test. It ORs four 16-byte chunks per iteration, picks the hitting chunk branch-free, then finishes with single chunks and an overlapping reread.
- `Byte_Set` tests 16 bytes with nibble tables (two lookups and an AND). More than 8 distinct high nibbles falls back to a scalar loop.
- Substring search: `make_finder` picks the needle's two rarest bytes (`rarity`), checks both in the vector loop and verifies with `memcmp`. When candidates keep failing it switches to Two-Way, which is linear time and constant space.
- Byte searches exist as a 16-byte version and a 32-byte one compiled with `#target_features "avx2"`. `dispatch` generates both and the first call picks one with `pick_target` ([choosing code by CPU](simd.md#choosing-code-by-cpu)). `Byte_Set` searches stay 16 bytes wide.

Compile-time calls select the portable 16-byte implementation without ISA probing. Macros that inspect another CPU target's caller `Code` therefore do not attempt that target's feature instructions. Running programs keep normal CPU dispatch.

## How to change it

- New byte search: add an `#insert #run dispatch(...)` line in `#scope_module` using `forward_search`/`backward_search`, then a one-line public procedure.
- Keep vector loops `no_inline`. Inlined into a big caller, LLVM can degrade them to byte loads (`find_pair` fell from 4 GB/s to 1 GB/s).
- The rarity table only needs a rough order (`make_rarity`). The Two-Way switch is `misses * 32 > scanned`.
- Tests: `tests/variants.jai` and `tests/search.jai` compare against naive loops. Run `PYTHONPATH=tools python3 tools/stdlib/stdlib_runtime.py --filter Byte_Search`.

Used by [Regex](regex.md) for match skipping.
