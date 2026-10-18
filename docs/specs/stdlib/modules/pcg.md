# PCG


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `random_get :: inline () -> u32`
- `random_get_within_bound :: inline (bound: u32) -> u32`
- `random_get_zero_to_one :: () -> float`
- `random_get_within_range :: (min: float, max: float) -> float`
- `random_seed :: (new_seed : u32)`
- `random_get :: (state : *u64) -> u32`
- `random_get_within_bound :: (state: *u64, range: u32) -> u32`
- `random_get_within_bound2 :: (state: *u64, range: u32) -> u32`
- `random_get_zero_to_one :: (state: *u64) -> float`
- `random_get_within_range :: (state: *u64, min: float, max: float) -> float`
- `random_seed :: (state: *u64, new_seed : u32)`


## Behavioral contract

The module provides state-pointer APIs and convenience overloads backed by a module-global `u64` state. Each draw transforms the pre-advance value with the XSH-RR output step: xor the old value with its right shift by 18, shift right by 27 and truncate to `u32`, then rotate right by the top five bits of the old value. It then advances with wrapping arithmetic modulo 2^64:

```jai
state = state * multiplier + increment;
```
The generator uses multiplier `6364136223846793005` and increment `1442695040888963407`; these values define the public sequence and are not importer-visible constants. Seeding sets the state to `cast(u64)new_seed * multiplier + increment`, also modulo 2^64; the first subsequent draw uses that seeded state as its pre-advance input. The global convenience API starts from a private state initialized to the increment value.

`random_get_within_bound` and `random_get_within_bound2` require a nonzero bound (asserted) and return values in `[0, range)`. The float helpers scale generated values into a requested range; no min/max ordering check is specified.

The module is explicitly marked untested and potentially broken. Its global state is not thread-local, so concurrent callers can race. State lifetime and synchronization are the caller's responsibility; no statistical quality guarantee is specified.

### Expected sequence example

For the state-pointer API, seeding with `42` produces state `0x91778aed87ee5eb1`. The first five `random_get` results are:

| Draw | Result (`u32`) | State after draw (`u64`) |
| ---: | ---: | ---: |
| 1 | `0x75830bbd` | `0x39b7f8a5c64cf56c` |
| 2 | `0x0e6dfdb2` | `0x69afc5a5e88b394b` |
| 3 | `0xce19afdf` | `0xa161c43f85f3c87e` |
| 4 | `0xd8cfe2c3` | `0xae1e2aac8d0e4175` |
| 5 | `0x012b061e` | `0x06b6f019aa7b0de0` |

This vector follows the public seed, pre-advance output, and wrapping state-update contract. It provides a compatibility check for those formulas; it does not guarantee that the generator is statistically suitable.
