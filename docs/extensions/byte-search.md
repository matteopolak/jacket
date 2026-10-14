# Byte_Search (jaic extension)

## What it is

`Extensions/Byte_Search` finds bytes and substrings with vector instructions: the job of C's `memchr`/`memmem` and Rust's `memchr` crate. It is built on [SIMD](simd.md). It is a jaic extension, not official Jai.

```jai
Byte_Search :: #import "Extensions/Byte_Search";

at := Byte_Search.find(text, #char "\n");              // first newline, or -1
at  = Byte_Search.find_any(text, #char ",", #char ";");
at  = Byte_Search.find_last(text, #char "/");
n  := Byte_Search.count(text, #char "\n");
at  = Byte_Search.find(text, "needle");                // first "needle", or -1

finder := Byte_Search.make_finder("needle");           // analyse a needle once
at = Byte_Search.find(*finder, text, start = at + 1);

digits := Byte_Search.make_byte_set(cast([] u8) "0123456789");
at = Byte_Search.find(*digits, text);                   // first digit
dash := Byte_Search.make_byte_set(.[#char "-"]);
at = Byte_Search.find_pair(*digits, *dash, 4, text);    // a digit with "-" 4 bytes on
```

| Procedure | Finds |
| --- | --- |
| `find(text, byte, start = 0)` | first `byte` at or after `start` |
| `find_any(text, a, b, start)`, `find_any(text, a, b, c, start)` | first of two or three bytes |
| `find_last(text, byte, end = -1)`, `find_last_any(...)` | last before `end` (-1: the end of the text) |
| `count(text, byte)` | how many times `byte` occurs |
| `find(text, needle, start)`, `find_last(text, needle, end)` | substrings |
| `make_finder(needle)`, `find(*Finder, ...)`, `find_last(*Finder, ...)` | a needle searched repeatedly |
| `make_byte_set(bytes)`, `contains(*Byte_Set, byte)`, `find(*Byte_Set, text, start)` | any byte of a set |
| `find_pair(*a, *b, gap, text, start)` | `i` with `text[i]` in `a` and `text[i + gap]` in `b` |
| `find_triple(*a, *b, *c, gap_2, gap_3, text, start)` | the same with a third set at `i + gap_3` |

Every search returns a position in the whole text, or -1. Every search is linear in the text, even with a needle built to defeat it.

## How it works

- **Byte searches:** a single generator, `forward_search`/`backward_search`, emits them through `#insert`. Each takes a mask expression and a scalar test. The loop:
  - compares four 16-byte chunks per iteration and ORs the masks, so it branches once per 64 bytes;
  - on a hit, picks the chunk without branches (which chunk holds it is unpredictable);
  - then compares single chunks;
  - then rereads the last partial chunk overlapping the previous one, shifting off the lanes already checked;
  - uses a scalar loop for texts shorter than 16 bytes.
- **Byte sets** (`Byte_Set`) are nibble tables: each high nibble that occurs gets one of 8 bucket bits, and the low-nibble table lists the buckets holding a byte with that low nibble. Two table lookups (`tbl`/`pshufb`) and an AND test 16 bytes. A set over more than 8 high nibbles (`wide`) falls back to a scalar loop. A set of one byte (`single`) uses a plain compare instead, which is about 3x faster for `find_pair`/`find_triple`.
- **`count`:** subtracts each comparison mask (0xFF = -1) from u8 counters, and empties them every 255 chunks before they can overflow.
- **Substring search** runs in two stages:
  - **Vector loop:** `make_finder` picks the needle's two rarest bytes using a fixed frequency table (`rarity`: English letter order, then whitespace, digits, punctuation, non-ASCII). The loop compares both bytes at their offsets 64 candidate positions at a time and only on a hit looks at each 16, checking every surviving candidate with `memcmp`.
  - **Two-Way:** when candidates keep failing (`misses > 8 && misses * 32 > bytes scanned`), the search switches to Two-Way (Crochemore–Perrin) for the rest of the text. Two-Way is linear time and constant space. `make_finder` computes the factorization for both directions.

Speed on Apple M-series arm64, native `-O2`, versus Rust's `memchr` 2.x:

| Search | 16 MB, Byte_Search | 16 MB, memchr | 256 MB, Byte_Search | 256 MB, memchr |
| --- | --- | --- | --- | --- |
| `find` byte (absent) | 92 GB/s | 89 GB/s | 56–61 GB/s | 67 GB/s |
| `find_last` byte | 99 GB/s | 96 GB/s | 66 GB/s | 67 GB/s |
| `find_any` 3 bytes | 71 GB/s | 34 GB/s | 59 GB/s | 33 GB/s |
| `count` | 78 GB/s | 53 GB/s | | |
| substring, absent | 64 GB/s | 45 GB/s | | |
| iterate a byte matched every ~32 bytes | 2.6 GB/s | 3.1 GB/s | | |
| `find_triple` of single bytes (`the`), every hit | 9 GB/s | | | |
| `find_triple` of single bytes, absent | 44 GB/s | | | |
| `find_pair` of byte sets (`th`), every hit | 4.2 GB/s | | | |

Single-byte search runs at memory bandwidth, the same as memchr. Iterating dense matches is bound by leaving the loop on every hit.

## How to change it

- **New byte searches:** call `forward_search`/`backward_search` with a mask expression over `chunk` and a scalar test over `c`. The generated code expects local variables named in those expressions; `find` declares `a := byte`.
- **Keep the vector loops `no_inline`.** The SIMD lane operations are loops that LLVM turns into vector instructions after inlining them. Inlined into a large caller, such a search can fall back to loading bytes one by one (`find_pair` went from 4 GB/s to 1 GB/s that way), so every procedure that holds a `forward_search`/`backward_search` loop is `no_inline`.
- **The rarity table:** it only needs a rough order. Change `make_rarity`, and keep it free of anything copied from other implementations.
- **The fallback threshold:** `misses * 32 > scanned` sets when Two-Way takes over. Lower it and adversarial needles fall back sooner; raise it and typical text stays longer in the faster vector loop.
- **Tests:** `tests/search.jai` checks every procedure against naive loops:
  - random texts, at every start and end;
  - small alphabets;
  - random byte sets (wide ones and single bytes), with pairs and triples at every gap;
  - periodic and adversarial needles;
  - a 2 MB linear-time check.

  Run it in all four modes with `tools/stdlib_runtime.py --filter Byte_Search`.

## Configuration

None. The vector width is 16 bytes (`Bytes :: u8x16`).

## Dependencies

- [SIMD](simd.md) (and through it the `byte_lookup`/`sign_mask` intrinsics).
- `Basic`.
