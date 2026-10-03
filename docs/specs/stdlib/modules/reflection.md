# Reflection


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `get_array_count_and_data :: (pointer: *void, info: *Type_Info_Array) -> (count: s64, data: *void)`
- `set_value_from_string :: (any: Any, text: string) -> bool`
- `set_value_from_string :: (any: Any, text: *string) -> bool`
- `range_check_and_store :: (value: $T, info: *Type_Info_Integer, pointer: *void) -> (success: bool, low: T, high: T)`
- `signed_integer_range_from_size :: (size_in_bytes: int) -> (low: s64, high: s64) #no_aoc`
- `unsigned_integer_range_from_size :: (size_in_bytes: int) -> (low: u64, high: u64) #no_aoc`
- `get_struct_field_info :: (info: *Type_Info_Struct, name: string) -> *Type_Info_Struct_Member`
- `enum_name_to_value :: ($T: Type, name: string) -> T, bool`
- `enum_value_to_name :: (e_value: $Enum) -> string`
- `get_enum_value :: (data: *void, info: *Type_Info_Enum) -> s64`
- `set_enum_value :: (data: *void, info: *Type_Info_Enum, value: s64)`


## Behavioral contract

Helpers inspect `Any` and `Type_Info` to read/write scalar values, resolve enum names, find struct fields, and obtain array count/data from fixed arrays, views, or resizable arrays. `set_value_from_string` advances the supplied mutable string cursor only as parsing consumes input; it supports integer, float, bool, quoted string, and enum values, rejecting unsupported types and out-of-range integer conversions. Strings currently have no escape-sequence support, and boolean token boundaries are not checked. Unknown enum names return false; unknown numeric enum values have no name. Depends on runtime reflection metadata and string parsers.
