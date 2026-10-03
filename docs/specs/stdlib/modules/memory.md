# Memory

## Public declaration inventory

### Module parameters

- `with_candidates := false` enables alternate copy candidate procedures.
- `with_unwraps := false` enables fixed-width copy procedures and unwrap variants.


Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `copy :: (destination: *void, source: *void, count: s64)`

### Optional copy candidate and unwrap declarations

- `copy :: (destination: *void, source: *void, count: s64)`

- [with_unwraps] `avx_copy_256 :: inline (destination: *void, source: *void)`
- [with_candidates] `copy_0 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_10_sse_128 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_11_sse_256 :: (destination: *void, source: *void, count: s64)`
- [with_unwraps] `copy_128 :: inline (destination: *void, source: *void)`
- [with_candidates] `copy_12_sse_512 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_13_sse_1024 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_1_u64 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_20_avx_256 :: (destination: *void, source: *void, count: s64)`
- [with_unwraps] `copy_256 :: inline (destination: *void, source: *void)`
- [with_unwraps] `copy_256_expand_array :: (destination: *void, source: *void) #expand`
- [with_unwraps] `copy_256_expand_assign :: (destination: *void, source: *void) #expand`
- [with_unwraps] `copy_256_expand_index :: (destination: *void, source: *void) #expand`
- [with_unwraps] `copy_256_inline_array :: inline (destination: *void, source: *void)`
- [with_unwraps] `copy_256_inline_assign :: inline (destination: *void, source: *void)`
- [with_unwraps] `copy_256_inline_index :: inline (destination: *void, source: *void)`
- [with_candidates] `copy_2_u64_128 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_3_u64_256 :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_4_u64_256_asm :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_5_asm_rep :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `copy_6_asm_rep_bytes :: (destination: *void, source: *void, count: s64)`
- [with_unwraps] `copy_u64_unwrap :: (destination: *void, source: *void, count: s64, $call: type_of(copy_256))`
- [with_unwraps] `copy_u64_unwrap_nocall_array :: (destination: *void, source: *void, count: s64)`
- [with_unwraps] `copy_u64_unwrap_nocall_asm :: (destination: *void, source: *void, count: s64)`
- [with_unwraps] `copy_u64_unwrap_nocall_assign :: (destination: *void, source: *void, count: s64)`
- [with_unwraps] `copy_u64_unwrap_nocall_index :: (destination: *void, source: *void, count: s64)`
- [with_candidates] `has_avx : enum`
- [with_unwraps] `sse_copy_1024 :: inline (destination: *void, source: *void)`
- [with_unwraps] `sse_copy_128 :: inline (destination: *void, source: *void)`
- [with_unwraps] `sse_copy_256 :: inline (destination: *void, source: *void)`
- [with_unwraps] `sse_copy_512 :: inline (destination: *void, source: *void)`

## Public type members

`has_avx` is an exported enum-valued state under `with_candidates`:

```jai
has_avx : enum {
    UNDETERMINED :: -1;
    NO;
    YES;
} = .UNDETERMINED;
```
## Behavioral contract and constraints

`copy(destination, source, count)` copies exactly `count` bytes and supports overlapping source/destination ranges with the same result as a move operation. A zero count performs no byte writes. Both ranges must be valid for `count` bytes and `count` must be nonnegative; the procedure returns no status. Optional candidate and unwrap declarations are present only when their corresponding module parameter is enabled; both parameters default to false. In this API, numeric suffixes on `copy_128` and `copy_256` count bits: they copy 16 and 32 bytes respectively. `sse_copy_128/256/512/1024` suffixes likewise name bit counts, copying 16/32/64/128 bytes. `avx_copy_256` copies 32 bytes and requires X64. These fixed-width routines directly access the requested region, so both ranges must cover the full width; overlap behavior is not guaranteed for these variants. Candidate and unwrap routines are specialized low-level alternatives; callers must meet any encoded fixed-size/alignment preconditions. The module marks compare/set routines incomplete, so no broader contract is specified.
