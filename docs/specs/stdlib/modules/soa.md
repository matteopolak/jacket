# Soa


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Soa :: struct (T: Type, N: s64) #modify`
- `operator [] :: (soa: $S/Soa, i: int) -> S.T`
- `operator []= :: (array: *$S/Soa, index: int, value: S.T)`
- `for_expansion :: (array: *$S/Soa, body: Code, flags: For_Flags) #expand`
- `example :: ()`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Soa`

```jai
Soa :: struct (T: Type, N: s64) #modify
```

The generated struct has one member for each non-constant, non-overlapping stored member `m` of `T`: `m: [N] type_of(T.m)`. This is a type-dependent public layout; `Soa` has no fixed list of member names independent of `T`. The generated layout preserves the member names and component types of `T` and omits overlay aliases and compile-time-only members.
## Contracts and constraints

`Soa(T,N)` requires `T` to be a struct and produces one `[N]` backing array per member. Indexed read returns a value assembled from the member arrays; indexed assignment writes each corresponding member slot. The index must be within the fixed extent, as for an ordinary fixed array. The custom loop visits indexes in ascending order; reverse iteration is asserted unsupported. Default iteration yields values; pointer mode yields an address suitable for updating fields in place but is documented as slow. Nested-member assignment through an indexed value is not rewritten into component writes. There is no allocation, growth, or lifetime management: the extent is fixed at `N`. This remains an experimental facility, not a general dynamic container.
