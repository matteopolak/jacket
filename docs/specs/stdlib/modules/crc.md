# Crc

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `crc64 :: (s: string) -> u64`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`crc64` processes string bytes in order with the 64-bit ECMA polynomial `0x42F0E1EBA9EA3693`, a non-reflected (MSB-first) update, initial register `0xFFFFFFFFFFFFFFFF`, and final XOR `0xFFFFFFFFFFFFFFFF`. The result is `u64`; the empty string returns the final-XOR of the initial register, zero. The lookup table is an implementation detail, not an exported constant.
