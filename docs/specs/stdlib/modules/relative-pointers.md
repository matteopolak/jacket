# Relative_Pointers


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Relative_Pointer :: struct (Storage_Type: Type, Pointer_To_Type: Type)`
- `Relative_String :: struct (Storage_Type: Type)`
- `decode :: inline (p: *Relative_Pointer) -> (result: *p.Pointer_To_Type)`
- `encode :: (p: *Relative_Pointer, absolute: *p.Pointer_To_Type)`
- `decode :: (p: *Relative_String) -> string`
- `decode_relative_pointer :: (relative_pointer_size: s32, address: *void) -> (pointer: *void, offset: s64)`
- `decode_relative_pointer :: (relative_pointer_size: s32, pointer_address: *void, base_address: *void) -> (pointer: *void, offset: s64)`
- `decode_relative_pointer_static :: ($type: Type, address: *void) -> (pointer: *void)`
- `write_relative_pointer :: ($type: Type, pointer_address: *void, target_address: *void)`
- `write_relative_pointer :: ($type: Type, pointer_address: *void, base_address: *void, target_address: *void)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Relative_Pointer`

```jai
Relative_Pointer :: struct (Storage_Type: Type, Pointer_To_Type: Type) {
    value: Storage_Type;

    #assert (Storage_Type == s8) || (Storage_Type == s16) || (Storage_Type == s32) || (Storage_Type == s64);
}
```
### `Relative_String`

```jai
Relative_String :: struct (Storage_Type: Type) {
    count: Storage_Type;
    data:  Storage_Type;

    #assert (Storage_Type == s8) || (Storage_Type == s16) || (Storage_Type == s32) || (Storage_Type == s64);
}
```
## Behavioral contract

`Relative_Pointer(Storage_Type, Pointer_To_Type)` stores a signed 8-, 16-, 32-, or 64-bit displacement in `value`. The sign-bit mask for the selected width is `1 << (width - 1)`: a null target is encoded as zero; a non-null target is encoded as its signed displacement XORed with that mask. Decoding XORs the stored value with the same mask to recover the signed offset. This reserves zero for null while allowing a target at the relative base (offset zero). The one-address helpers use the storage field address as the base; the overloads that accept `base_address` use that explicit base. The displacement is relative to the base, not an absolute address.

`Relative_String` stores its count and relative data pointer in the selected signed width; `decode` returns a non-owning string view. The container can be relocated with its references only when targets move compatibly and remain within the representable displacement range. No bounds, lifetime, or target-layout validation is provided. Depends on pointer arithmetic and compile-time width specialization.
