# SIMD


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `SIMD_Array :: struct (count: s64, type: Type)`
- `Simd_128 :: union`
- `set :: ($N: s64, factor: $T) -> SIMD_Array(N, T) #no_abc #no_aoc`
- `load :: (base: *void, $N: int, $F: Type) -> SIMD_Array(N, F) #no_abc #no_aoc` (returns the constructed vector)
- `load_from_offsets :: (base: *void, offsets: $O/SIMD_Array, $F: Type) -> SIMD_Array(offsets.count, F) #no_abc #no_aoc`
- `store :: (base: *void, a: $T/SIMD_Array) #no_abc #no_aoc`
- `store_with_offsets :: (a: $T/SIMD_Array, base: *void, offsets: $O/SIMD_Array) #no_abc #no_aoc`
- `get_looping_indices :: ($N: int, $I: Type, $STEP := 1) -> SIMD_Array(N, I) #no_abc #no_aoc`
- `operator + :: (a: $T/SIMD_Array, factor: a.type) -> T #symmetric #no_abc #no_aoc`
- `operator - :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator - :: (factor: $E, a: SIMD_Array($N, E)) -> SIMD_Array(N, E) #no_abc #no_aoc`
- `operator * :: (a: $T/SIMD_Array, factor: a.type) -> T #symmetric #no_abc #no_aoc`
- `operator / :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator / :: (factor: $E, a: SIMD_Array($N, E)) -> SIMD_Array(N, E) #no_abc #no_aoc`
- `operator > :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator >= :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator < :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator <= :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator == :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator != :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `max :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `min :: (a: $T/SIMD_Array, factor: a.type) -> T #no_abc #no_aoc`
- `operator + :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator - :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator * :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator / :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator > :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator >= :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator < :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator <= :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator == :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `operator != :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `max :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `min :: (a: $T/SIMD_Array, b: T) -> T #no_abc #no_aoc`
- `horizontal_add :: (a: $T/SIMD_Array, b: T) -> (interleaved_result: T)`
- `horizontal_sum :: (a: $T/SIMD_Array) -> a.type`
- `get_mask :: (a: $T/SIMD_Array) -> s64`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `SIMD_Array`

```jai
SIMD_Array :: struct (count: s64, type: Type) {
    data: [count] type;
}
```
### `Simd_128`

```jai
Simd_128 :: union  {
    _u8:  SIMD_Array(16, u8);
    _s8:  SIMD_Array(16, s8);
    _u16: SIMD_Array(8,  u16);
    _s16: SIMD_Array(8,  s16);
    _u32: SIMD_Array(4,  u32);
    _s32: SIMD_Array(4,  s32);
    _u64: SIMD_Array(2,  u64);
    _s64: SIMD_Array(2,  s64);

    _float32: SIMD_Array(4, float32);
    _float64: SIMD_Array(2, float64);
}
```
## Behavioral contract

`SIMD_Array(N, T)` is a value with `N` lanes of `T`; operations allocate no storage. `set` fills every lane with its scalar argument. `load(base, N, F)` copies `N` consecutive `F` values beginning at `base` into and returns a vector in increasing-address lane order. It returns that vector; ignoring the result discards the loaded value. `store(base, a)` copies all of `a`'s lanes contiguously in lane order. These contiguous operations use byte copying and establish no stronger alignment precondition.

`load_from_offsets(base, offsets, F)` requires integer offset lanes and reads an `F` at each byte address `base + offsets[i]`; it returns those values in corresponding lane order. `store_with_offsets(a, base, offsets)` writes each input lane to `base + offsets[i]`; offsets are signed byte displacements, not element indexes. Repeated gather offsets repeat a value in result lanes. Repeated scatter destinations are processed in lane order, so later lanes overwrite earlier ones. The module does not validate bounds, pointer validity or alignment beyond asserting that `base` is non-null when there are lanes.

`get_looping_indices(N, I, STEP := 1)` returns `N` index lanes of type `I` with values `i * STEP` for lane index `i` starting at zero. It does not wrap or advance the vector for later iterations; callers update the values themselves. Positive lane counts are the intended use. No overflow or invalid count policy is specified.

Arithmetic operations apply the corresponding scalar operation lane by lane. Comparison operations convert each scalar comparison result to the vector lane type, yielding the type's conversion of true/false (for integer lanes, 1/0; they are not all-bits-set masks). `get_mask` maps truthy lane `i` to bit `i` of its `s64` result. Its useful range is at most 64 lanes; behavior for larger vectors is unspecified. `horizontal_sum` adds all lanes in order and returns one scalar of the lane type.

`horizontal_add(a, b)` requires an even lane count and returns the interleaved pair sums `[a0+a1, b0+b1, a2+a3, b2+b3, ...]` for every pair.

The scalar overloads of `+` and `*` are marked `#symmetric`, so the scalar may be written on either side. `-` and `/` are not commutative, so each has a separate scalar-on-the-left overload: `scalar - vector` and `scalar / vector` compute the scalar operation against every lane. Every operation is a scalar loop over the lanes on all platforms (no hardware vector instructions); integer arithmetic wraps, and division edge cases follow the underlying scalar operation. The doc comments of these procedures live in `tools/generators/generate_simd_lanes.py`, which generates `stdlib/internal/simd/lanes.jai`.
