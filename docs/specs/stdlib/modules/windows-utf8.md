# Windows_Utf8


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `utf8_to_wide :: (utf8: string) -> (result_data: *u16, success: bool, result_length_in_bytes: s32)`
- [OS == .WINDOWS] `wide_to_utf8 :: (data: *u16, length : s32 = -1) -> (result: string, success: bool)`


## Contracts and constraints

These helpers convert UTF-8 Jai strings to Windows UTF-16 buffers and back using Kernel32. `utf8_to_wide` returns allocated, zero-terminated `u16` storage and reports the encoded UTF-16 byte length excluding the terminator. Empty input succeeds with a null pointer and length zero; conversion failure returns a null pointer, `false`, and zero. Input length is narrowed to `s32`, and oversized inputs are noted as a robustness limitation. `wide_to_utf8` returns allocated string storage, and the caller must free the result. A length of `-1` means zero-terminated wide input; explicit lengths have compatibility behavior that appends a terminator while excluding it from the returned count. API failure returns `success=false`.
