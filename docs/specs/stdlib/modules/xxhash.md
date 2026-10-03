# xxHash


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `XXH_VERSION_MAJOR :: 0`
- `XXH_VERSION_MINOR :: 7`
- `XXH_VERSION_RELEASE :: 0`
- `XXH_VERSION_NUMBER :: (XXH_VERSION_MAJOR *100*100 + XXH_VERSION_MINOR *100 + XXH_VERSION_RELEASE)`
- `XXH_versionNumber :: () -> u32`
- `native_XXH32 :: (input: *void, len: u32, seed: u32) -> u32`
- `XXH32_state_t :: struct`
- `XXH32_reset :: (state: *XXH32_state_t, seed: u32)`
- `XXH32_update :: (state: *XXH32_state_t, input: *void, len: u32)`
- `XXH32_digest :: (state: *XXH32_state_t) -> u32`
- `native_XXH64 :: (input: *void, len: u32, seed: u32) -> u64`
- `XXH64_state_t :: struct`
- `XXH64_reset :: (state: *XXH64_state_t, seed: u64)`
- `XXH64_update :: (state: *XXH64_state_t, input: *void, len: u32)`
- `XXH64_digest :: (state: *XXH64_state_t) -> u64`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `XXH32_state_t`

```jai
XXH32_state_t :: struct {
   total_len_32 : u32;
   large_len : u32;
   v1 : u32;
   v2 : u32;
   v3 : u32;
   v4 : u32;
   mem32 : [4]u32;
   memsize : u32;
   reserved : u32;
}
```
### `XXH64_state_t`

```jai
XXH64_state_t :: struct {
    total_len: u64;
    v1: u64;
    v2: u64;
    v3: u64;
    v4: u64;
    mem64: [4]u64;
    memsize: u32;
    reserved: [2]u32;
}
```
## Contracts and constraints

This module implements the xxHash 0.7.0 family with one-shot and streaming state APIs for 32-bit and 64-bit hashes. Streaming use follows reset → update zero or more times → digest; the caller owns state and supplies valid input memory for each update. Empty input is supported by the algorithm, but pointer/length validity remains a caller precondition. Hash outputs are deterministic for a given version, seed, and byte sequence; the module’s exposed version constants identify the bundled API revision.
