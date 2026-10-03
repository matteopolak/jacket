# Bit_Array

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Bit_Array :: struct`
- `init_bit_array :: (array: *Bit_Array, count: int)`
- `make_bit_array :: (count: int) -> Bit_Array`
- `deinit :: (using array: *Bit_Array)`
- `set_bit :: (using array: *Bit_Array, i: int)`
- `clear_bit :: (using array: *Bit_Array, i: int)`
- `toggle_bit :: (using array: *Bit_Array, i: int)`
- `set_bit_to :: (using array: *Bit_Array, i: int, value: bool)`
- `clear_all_bits :: (using array: *Bit_Array)`
- `set_all_bits :: (using array: *Bit_Array)`
- `toggle_all_bits :: (using array: *Bit_Array)`
- `for_expansion :: (array: *Bit_Array, body: Code, flags: For_Flags) #expand`
- `only_set :: #bake_arguments only_set_or_unset(target_value = true)`
- `only_unset :: #bake_arguments only_set_or_unset(target_value = false)`

## Public type members

#### `Bit_Array` (struct)

- `slots: [] s64`
- `count: int`
- `allocator: Allocator`

## Behavioral contract and constraints

`init_bit_array` allocates `ceil(count/64)` `s64` words through `array.allocator` or, when unset, `context.allocator`. Newly allocated words are zero-initialized by `Basic.NewArray`; calling init twice without deinit leaks/replaces the previous allocation and is unsupported. `make_bit_array` returns the initialized value. `deinit` frees storage with the recorded allocator and nulls `slots.data`; it does not promise to clear `count`.

Indexed access and mutation require `0 <= i < count`; the implementation asserts the upper bound but does not explicitly assert nonnegativity, so negative indexes are invalid input. `set_bit`, `clear_bit`, `toggle_bit`, `set_bit_to`, and `operator []=` modify only the addressed bit. `clear_all_bits` zeroes all allocated words; `set_all_bits` and `toggle_all_bits` affect padding bits in the last word too. Reads and iteration are limited to logical indexes `[0,count)`. Ordinary `for` iteration visits every logical bit in increasing order: `it` is that bit's boolean value and `it_index` is its index. Flags must be zero. `only_set`/`only_unset` visit indexes whose value matches the requested state, yielding the selected boolean as `it` and the original logical index as `it_index`. No resize operation or concurrent-access guarantee is provided.
