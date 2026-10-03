# Bit_Operations

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `byte_swap :: (input: $T) -> T #expand`
- `byte_swap_in_place :: (input: *$T) #expand`
- `bit_scan_forward :: (input: $T) -> s32 #expand`
- `bit_scan_reverse :: (input: $T) -> s32 #expand`
- `popcount :: (value: $T) -> s32 #expand`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`byte_swap` returns a byte-reversed value and `byte_swap_in_place` mutates it. Forward and reverse bit scans return one plus the matching bit index, or zero for zero input; `popcount` returns the number of set bits. These are generic over supported integer widths. The module uses x64 instructions where available and a generic fallback elsewhere; no performance guarantee is part of the API.
