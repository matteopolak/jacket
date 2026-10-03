# Random


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `random_get :: (state: *Random_State) -> u64 #no_aoc`
- `random_get :: inline () -> u64`
- `random_seed :: inline (state: *Random_State, new_seed: u64)`
- `random_seed :: inline (new_seed: u64)`
- `random_seed :: inline (state: *Random_State, new_seed: Random_State)`
- `random_seed :: inline (new_seed: Random_State)`
- `Some_Big_Time_Value :: struct`
- `random_seed :: inline (state: *Random_State, time: $T/interface Some_Big_Time_Value)`
- `random_seed :: inline (time: $T/interface Some_Big_Time_Value)`
- `random_get_zero_to_one :: inline (state: *Random_State) -> float`
- `random_get_zero_to_one :: inline () -> float`
- `random_get_zero_to_one_open :: inline (state: *Random_State) -> float`
- `random_get_zero_to_one_open :: inline () -> float`
- `random_get_zero_to_one_new :: (state: *Random_State) -> float64 #no_aoc`
- `random_get_zero_to_one_new :: inline () -> float64`
- `random_get_zero_to_one_open_new :: (state: *Random_State) -> float64`
- `random_get_zero_to_one_open_new :: inline () -> float64`
- `random_get_within_range :: inline (state: *Random_State, min: float, max: float) -> float`
- `random_get_within_range :: inline (min: float, max: float) -> float`
- `Random_State :: struct`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Random_State`

```jai
Random_State :: struct {
    low  : u64;
    high : u64;
}
```
### `Some_Big_Time_Value`

```jai
Some_Big_Time_Value :: struct {
    low:  u64;
    high: s64;
}
```
## Contracts and constraints

The module implements a multiplicative congruential generator over a 128-bit state. Each draw multiplies the state modulo 2^128 by the fixed factor `0x0fc94e3bf4e9ab32866458cd56f5e605`, then returns the new state’s high 64 bits. State-based and context-backed APIs use the same two-limb representation.

A single `u64` seed produces `{low = 1, high = seed}`. A `Random_State` seed or time-like seed copies its high half and forces the low half odd with `low | 1`; this keeps the complete 128-bit state odd, as required for the generator’s intended maximal cycle over odd states. The default context state begins at `{low = 1, high = 0}`.

The 32-bit-float helper `random_get_zero_to_one` uses 24 random bits divided by `2^24 - 1`, so its result is in `[0, 1]`; `random_get_zero_to_one_open` divides by `2^24`, yielding `[0, 1)`. The `_new` double-precision helpers use 64-bit outputs and return values in `[0, 1)`; the open variant constructs a 53-bit fraction. `random_get_within_range` accepts either endpoint order and clamps rounding drift to the closed interval between them.

Calls mutate the supplied or context state. Use separate states for independent streams and synchronize shared concurrent access. The implementation has an x64 assembly path and a portable multiply path; both must preserve wrapping arithmetic for matching sequences.
