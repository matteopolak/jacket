# Hash

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `sdbm_hash :: (data: *void, size: int, h := HASH_INIT) -> u32`
- `sdbm_float_hash :: inline (f: *float, count: int, h := HASH_INIT) -> u32`
- `djb2_hash :: (s: string) -> u32`
- `FNV_64_PRIME : u64 : 0x100000001b3`
- `FNV_64_OFFSET_BIAS : u64 : 0xcbf29ce484222325`
- `fnv1a_hash :: inline (val : u64, h := FNV_64_OFFSET_BIAS) -> u64 #no_aoc`
- `fnv1a_hash :: (data: *void, size: int, h := FNV_64_OFFSET_BIAS) -> u64`
- `knuth_hash :: inline (x : u64) -> u64 #no_aoc`
- `get_hash :: (x : $T, h := HASH_INIT) -> u32 #modify`
- `get_hash :: (s: string, h := HASH_INIT) -> u32`
- `get_hash :: (array: [] $T, h := HASH_INIT) -> u32`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

Provides sdbm, djb2, FNV-1a, and Knuth multiplicative hashes. Byte/string/array routines process exactly the supplied byte count; zero length returns the input seed (or the algorithm’s default seed) without reading memory. `sdbm_hash` and byte-array `get_hash` default to seed 5381. The local default seed constant is explicitly `u32`, so the default `h` parameter of `sdbm_hash`, `sdbm_float_hash`, and generic `get_hash` is `u32`; the seed constant itself is not an exported module identifier. `djb2_hash` always starts from 5381. `fnv1a_hash(val: u64, h := FNV_64_OFFSET_BIAS)` consumes the entire scalar as one `u64` value: it XORs `val` with `h` once and multiplies once by `FNV_64_PRIME`. It does not process the scalar as eight ordered bytes. The byte-pointer overload instead applies that scalar step separately to each byte in ascending address order. `get_hash(string)` uses the byte-pointer overload and truncates to `u32`; `get_hash([]T)` hashes the array’s raw element bytes, so it is representation-dependent and does not include a type or length marker. For scalar integer/bool/enum/pointer values, `get_hash(x, h)` converts `x` to `u64` with `cast,no_check`, XORs that value with `h`, applies `knuth_hash`, then returns the upper 32 bits as `u32`: `cast(u32)(knuth_hash((cast,no_check(u64) x) ^ h) >> 32)`. Negative integer behavior follows Jai’s `cast,no_check(u64)` conversion before the XOR. Scalar floats are hashed from their byte representation with sdbm. Generic scalar `get_hash` supports integer, boolean, enum, pointer, and float only; unsupported types fail specialization. `sdbm_float_hash` treats IEEE negative zero as positive zero, while scalar float `get_hash` and raw array hashing do not normalize it. `FNV_64_PRIME` is `0x100000001b3`; `FNV_64_OFFSET_BIAS` is `0xcbf29ce484222325`; FNV-1a defaults to the offset basis. `knuth_hash` multiplies by `11400714819323198485` (`0x9E3779B97F4A7C15`). These hashes are deterministic but non-cryptographic; outputs may be folded/truncated for table indexing, and no collision-resistance guarantee is provided.
