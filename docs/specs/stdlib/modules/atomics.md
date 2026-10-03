# Atomics

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `atomic_swap :: atomic_exchange`
- `atomic_read :: atomic_load`
- `atomic_write :: atomic_store`
- `atomic_bit_test_and_set :: (dest: *$T, $$i: s64) -> bool`
- `IsAtomicScalar :: (T: Type) -> bool #compile_time`
- `atomic_add :: (dest: *$T, to_add: $I) -> T #modify`
- `atomic_increment :: (dest: *$T) #modify`
- `atomic_decrement :: (dest: *$T) -> bool #modify`
- `atomic_and :: (dest: *$T, value: T) #modify`
- `atomic_or :: (dest: *$T, value: T) #modify`
- `atomic_xor :: (dest: *$T, value: $I) #modify`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

The exported aliases preserve the legacy names for atomic load, store, and exchange. `atomic_add` returns the prior value; pointer increments are measured in bytes. `atomic_decrement` reports whether the new value is zero. Bit test-and-set returns the old bit and asserts a 2-, 4-, or 8-byte target; one-byte targets are explicitly unsupported. The source documents negative bit indices as counting from the high end. Detailed ordering guarantees are not specified in this module.
