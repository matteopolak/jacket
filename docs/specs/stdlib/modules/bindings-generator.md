# Bindings_Generator

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `TARGET_OS : type_of(OS)`
- `DO_DEBUG :: false`
- `DEBUG_ALIGNMENT :: false`
- `DEBUG_BITFIELDS :: false`
- `LIBCLANG_VERSION :: 21`
- `Library :: struct`
- `Generate_Bindings_Options :: struct #type_info_none`
- `Declaration_Visit_Result :: enum`
- `Declaration_Visitor :: #type (decl: *Declaration, parent_decl: *Declaration) -> Declaration_Visit_Result`
- `Declaration_Visitor_With_Data :: #type (decl: *Declaration, parent_decl: *Declaration, user_data: *void) -> Declaration_Visit_Result`
- `ensure_local_libclang_exists :: ()`
- `ensure_local_libclang_exists :: (source_path: string)`
- `RESERVED_NAMES :: string`
- `RESERVED_TYPE_NAMES :: string`
- `TYPEDEFS_TO_UNWRAP :: string`
- `TYPEDEF_PREFIXES_TO_UNWRAP :: string`
- `MACROS_TO_IGNORE :: string`
- `Mimic_Spacing_Flags :: enum_flags`
- `Library_Info :: struct`
- `Strip_Flags :: enum_flags`
- `Get_Function_Arguments_For_Printing_Func :: #type (func: *Function, type: *Function_Type, mode: Arg_Print_Mode) -> []*Declaration`
- `TOKENS_TO_REPLACE :: #insert #run generate_replacements_array()`
- `generate_replacements_array :: () -> string`
- `normalize_options :: (original_options: Generate_Bindings_Options) -> Generate_Bindings_Options, bool`
- `generate_bindings :: (options: Generate_Bindings_Options, filename: string, log_results := true) -> success: bool`
- `generate_bindings :: (_options: Generate_Bindings_Options, log_results := false) -> generated_result: String_Builder, success: bool`
- `Bridge_Data :: struct`
- `generate_bridge :: (_options: Generate_Bindings_Options, filename: string) -> success: bool`
- `create_omit_cpp_functions_visitor :: (visitor: Declaration_Visitor, visitor_with_data: Declaration_Visitor_With_Data, visitor_data: *void) -> Declaration_Visitor_With_Data, *void`
- `Cpp_Functions_Visitor_Data :: struct`
- `omit_cpp_functions_visitor :: (decl: *Declaration, parent_decl: *Declaration, user_data: *void) -> Declaration_Visit_Result`
- `append_cpp_wrapper :: (builder: *String_Builder, func: *Function, existing_declarations: *[..] *Declaration)`
- `avoid_cpp_keywords :: (name: string) -> string`
- `append_cpp_expression :: (builder: *String_Builder, expression: *Expression, target_type: *CType = null, current_scope: *Declaration = null) -> valid: bool`
- `append_cpp_literal :: (builder: *String_Builder, lit: *Literal, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> valid: bool`
- `append_cpp_type_modifiers :: (builder: *String_Builder, ty: *CType, flags: Print_Decl_Flags)`
- `append_cpp_type :: (builder: *String_Builder, ty: *CType, current_scope: *Declaration, flags: Print_Decl_Flags = 0)`
- `run_generator :: (generator: *Generator, options: Generate_Bindings_Options) -> bool`
- `get_default_system_include_paths :: (os := OS, cpu := CPU) -> [] string`
- `get_libc_paths :: (os := OS, cpu := CPU) -> libc_path: string, libc_platform_path: string, success: bool`
- `change_type_to_enum :: (decl: *Declaration, enum_name: string) -> bool`
- `change_type_to_enum :: (decl: *Declaration, _enum: *Enum)`
- `change_return_type_to_enum :: (decl: *Declaration, enum_name: string) -> bool`
- `change_return_type_to_enum :: (decl: *Declaration, _enum: *Enum)`
- `change_return_type_to_enum :: (name: string, function_type: *Function_Type, enum_name: string) -> bool`
- `change_return_type_to_enum :: (name: string, function_type: *Function_Type, _enum: *Enum)`
- `change_argument_type_to_enum :: (decl: *Declaration, argument_name: string, enum_name: string) -> bool`
- `change_argument_type_to_enum :: (decl: *Declaration, argument_name: string, _enum: *Enum) -> bool`
- `change_argument_type_to_enum :: (name: string, function_type: *Function_Type, argument_name: string, enum_name: string) -> bool`
- `change_argument_type_to_enum :: (name: string, function_type: *Function_Type, argument_name: string, _enum: *Enum) -> bool`
- `create_type_of_enum :: (_enum: *Enum) -> *CType`
- `change_return_type :: (decl: *Declaration, new_return_type: *CType)`
- `change_return_type :: (name: string, function_type: *Function_Type, new_return_type: *CType)`
- `find_parent_enum :: (decl: *Declaration, enum_name: string) -> *Enum`
- `visit_declarations :: (visitor: Declaration_Visitor_With_Data, data: *void, decl: *Declaration, parent: *Declaration = null, depth := 0)`
- `visit_declarations :: (visitor: Declaration_Visitor, decl: *Declaration, parent: *Declaration = null, depth := 0)`
- `omit_unnecessary_typedefs_and_macros :: (decl: *Declaration, parent: *Declaration, user_data: *void) -> Declaration_Visit_Result`
- `check_macro_value :: (decl: *Declaration)`
- `ComUuid :: struct`
- `Generator :: struct #type_info_none`
- `Token_Info :: struct`
- `Number_Flags :: enum_flags u8`
- `CType :: struct`
- `is_void_type :: (t: *CType) -> bool`
- `is_bool_type :: (t: *CType) -> bool`
- `Function_Type :: struct`
- `is_same_type :: (a: *CType, b: *CType) -> bool`
- `is_same_expression :: (a: *Expression, b: *Expression) -> bool`
- `C_Kind :: enum`
- `Expression :: struct`
- `Type_Reference :: struct`
- `Literal :: struct`
- `is_same_literal :: (a: *Literal, b: *Literal) -> bool`
- `Macro_Value :: struct`
- `Unary_Operator :: struct`
- `Binary_Operator :: struct`
- `Struct_Literal :: struct`
- `Paren_Expression :: struct`
- `Location :: struct`
- `Comment :: struct`
- `prepend_comment :: (comment: *Comment, text: string)`
- `is_same_location :: (a: Location, b: Location) -> bool`
- `Declaration :: struct`
- `Declaration_Flags :: enum_flags`
- `Function :: struct`
- `Typedef :: struct`
- `Enum :: struct`
- `Bitfield :: struct`
- `Bitfield_Status :: struct`
- `maybe_unwrap_bitfield_declaration :: (decl: *Declaration) -> *Declaration`
- `Declaration_Overlay :: struct`
- `Struct :: struct`
- `is_struct_field :: (decl: *Declaration) -> is_field: bool, anon_struct: *Struct`
- `Struct_Extra_Info :: struct`
- `Namespace :: struct`
- `Block :: struct`
- `TAB_STRING :: " "`
- `Arg_Print_Mode :: enum`
- `_default_get_func_args_for_printing :: (func: *Function, type: *Function_Type, mode: Arg_Print_Mode) -> []*Declaration`
- `maybe_add_spacing :: (builder: *String_Builder, last_location: *Location, location: Location, decl_flags: Declaration_Flags, mimic_type: Mimic_Spacing_Flags)`
- `maybe_add_spacing :: (builder: *String_Builder, last_location: *Location, location: Location, decl_flags: Declaration_Flags, mimic := true)`
- `create_pointer_to :: (pointee: *CType) -> *CType`
- `get_int_type :: (size: int, signed: bool) -> *CType`
- `create_type :: (ty: CXType, cur: CXCursor, cursor_is_accurate := true, loc := #caller_location, type_arg_index := 0, substitute_children: []CXCursor = .[]) -> *CType`
- `create_type_inner :: (ty: CXType, cur: CXCursor, cursor_is_accurate := true, loc := #caller_location, type_arg_index := 0, substitute_children: []CXCursor = .[]) -> *CType`
- `create_elaborated_type :: (ty: CXType, cur: CXCursor) -> *CType`
- `create_elaborated_type :: (ty: CXType, children: [] CXCursor, location_cur: CXCursor) -> *CType, int`
- `count_namespace_refs :: (children: [] CXCursor) -> int`
- `fill_template_arguments :: (ty: CXType, arguments: *[..] *Expression, children: [] CXCursor, location_cur: CXCursor) -> bool, int`
- `fill_template_arguments_for_expression :: (expr: *Literal, arguments: *[..] *Expression, children: [] CXCursor, parent: CXCursor) -> bool`
- `get_clang_type_alignment :: (ty: CXType) -> s64`
- `create_function_type :: (cur: CXCursor, ty: CXType, func: *Function = null, cursor_is_accurate := true, loc := #caller_location) -> *Function_Type`
- `maybe_convert_pointer_type_to_fixed_array_type :: (cur: CXCursor, type: *CType) -> *CType`
- `maybe_convert_fixed_array_type :: (type: *CType) -> *CType, changed: bool`
- `is_incomplete_array :: (type: *CType) -> bool`
- `looks_like_valid_identifier :: (identifier: string) -> bool`
- `should_unwrap_typedef :: (cur: CXCursor) -> bool`
- `should_unwrap_typedef :: (name: string) -> bool`
- `append_indentation :: (builder: *String_Builder)`
- `print_type_to_builder :: (builder: *String_Builder, ty: *CType, current_scope: *Declaration, flags: Print_Decl_Flags = 0, existing_declarations: *[..] *Declaration = null)`
- `append_template_instantiation_params :: (builder: *String_Builder, params: [] *Expression, type_params: [] *Declaration, current_scope: *Declaration)`
- `print_function_type_to_builder :: (builder: *String_Builder, type: *Function_Type, current_scope: *Declaration, func: *Function = null, flags: Print_Decl_Flags = 0)`
- `append_parent_path :: (builder: *String_Builder, decl: *Declaration, current_scope: *Declaration, location: Location, separator := ".", skip_flattened_namespaces := true) -> bool`
- `type_to_string :: (ty: *CType, current_scope: *Declaration, flags: Print_Decl_Flags = 0) -> string`
- `as_str :: (cur: CXCursor, all_info := false) -> string`
- `as_str :: (ty: CXType) -> string`
- `is_zero_int_expression :: (expression: *Expression) -> bool`
- `is_pointer_type :: (ty: *CType) -> bool`
- `find_underlying_type :: (type: *CType) -> *CType`
- `is_function_type :: (ty: *CType) -> bool`
- `is_16byte_integer_type :: (ty: *CType) -> bool`
- `strip_type :: (type: *CType, typedefs := false, enums := false, pointers := false) -> *CType`
- `create_variable_declaration :: (cur: CXCursor, parent: CXCursor, is_function_argument := false, parent_function: *Function = null) -> *Declaration, extra_parent_flags: Declaration_Flags`
- `is_extern_declaration :: (decl: *Declaration) -> bool`
- `try_find_struct_in_type :: (type: *CType) -> *Struct`
- `maybe_convert_expression_zero_to_null :: (decl: *Declaration)`
- `convert_children_to_expression :: (cur: CXCursor) -> *Expression`
- `create_expression_from_cursor :: (cur: CXCursor, parent_cursor: CXCursor) -> *Expression, skip: bool`
- `parse_call_expression :: (cur: CXCursor, expect_type_ref: bool) -> *Expression`
- `_create_declaration :: (name: string, type: *CType, cur: CXCursor) -> *Declaration`
- `_create_declaration :: (name: string, type: *CType, location: Location = .{}, comment: Comment = .{}) -> *Declaration`
- `find_referenced_declaration :: (cur: CXCursor) -> *Declaration`
- `create_null_literal :: () -> *Literal`
- `create_literal :: (raw_tokens_str: string, location: Location = .{}) -> *Literal`
- `create_literal :: (macro_value: Macro_Value, location: Location) -> *Literal`
- `create_literal :: (cur: CXCursor, parent_cursor: CXCursor) -> *Literal`
- `create_reference_expression :: (cur: CXCursor, location: Location) -> *Literal`
- `convert_objc_selector_name_to_function_identifier :: (s: *string)`
- `create_function :: (cur: CXCursor, parent: CXCursor) -> *Function`
- `add_access_flags :: (decl: *Declaration, cur: CXCursor)`
- `check_if_foreign_function_should_be_omitted :: (func: *Function)`
- `is_char_pointer_type :: (type: CType) -> bool`
- `create_typedef :: (cur: CXCursor) -> *Typedef`
- `type_has_name :: (type: *CType) -> bool`
- `fill_comment :: (comment: *Comment, cur: CXCursor, location: Location)`
- `is_prefix_comment :: (comment: Comment) -> bool`
- `fill_location_and_comment_if_needed :: (location: *Location, comment: *Comment, cur: CXCursor)`
- `fill_location :: (location: *Location, cur: CXCursor)`
- `create_empty_struct_type :: (name: string) -> *CType`
- `debug_print_cursor :: (cur: CXCursor)`
- `create_struct :: (cur: CXCursor, resolving_type_only := false) -> *Struct`
- `find_first_member_offset :: (outer_struct_type: CXType, outer_struct: *Struct, _struct: *Struct) -> int`
- `maybe_add_to_bitfield :: (using status: *Bitfield_Status, decl: *Declaration) -> *Declaration`
- `finish_bitfield :: (using bitfield: *Bitfield, index: int)`
- `append_parent_field_path :: (builder: *String_Builder, decl: *Declaration, current_scope: *Declaration, separator := ".") -> bool`
- `append_bitfield_helpers :: (builder: *String_Builder, bitfield: *Bitfield, scope: *Struct)`
- `append_maybe_left_shift :: (builder: *String_Builder, value: string, shift: int)`
- `append_maybe_right_shift :: (builder: *String_Builder, value: string, shift: int)`
- `create_namespace :: (cur: CXCursor, ignore_non_structs := false) -> *Namespace`
- `create_enum :: (cur: CXCursor, resolving_type_only := false) -> *Enum`
- `create_enum :: (name: string) -> *Enum`
- `compute_enum_names :: (_enum: *Enum, already_stripped := false)`
- `create_macro_definition :: (cur: CXCursor) -> *Declaration`
- `find_or_create_enum_for_macro :: (name: string, declaration: *Declaration) -> *Enum, is_new_enum: bool`
- `get_scoped_token_string :: (token: CXToken) -> string #expand`
- `is_token :: (token: CXToken, expected: string) -> bool`
- `convert_float_literal :: (c_value: string) -> value: string, success: bool`
- `convert_literal_value :: (c_value: string) -> value: string, success: bool, was_string := false`
- `is_hex_digit :: (c: u32) -> bool`
- `is_valid_int_literal_suffix :: (suffix: string) -> bool`
- `is_valid_float_literal_suffix :: (suffix: string) -> bool`
- `create_objc_interface :: (cur: CXCursor, resolving_type_only := false) -> *Struct`
- `create_objc_type_param :: (cur: CXCursor) -> *Declaration`
- `create_declaration :: (cur: CXCursor, parent: CXCursor, ignore_non_structs := false) -> *Declaration`
- `create_declaration_inner :: (cur: CXCursor, parent: CXCursor, ignore_non_structs := false) -> *Declaration`
- `get_name_for_cursor :: (cur: CXCursor) -> string`
- `handle_toplevel_cursors :: (cur: CXCursor, parent: CXCursor, client_data: CXClientData) -> CXChildVisitResult #c_call`
- `macro_literals_are_equal :: (a: *Literal, b: *Literal) -> bool`
- `macro_values_are_equal :: (a: Macro_Value, b: Macro_Value) -> bool`
- `raw_string_from_macro_tokens :: (value: Macro_Value) -> string`
- `size_from_number_flags :: (flags: Number_Flags) -> u32`
- `init_generator :: (using generator: *Generator, options: Generate_Bindings_Options)`
- `shared_type :: (t: *CType) -> *CType`
- `add_type_flags :: (t: *CType, flags: CType.Flags) -> *CType`
- `deinit_generator :: (generator: *Generator)`
- `print_expression_to_builder :: (builder: *String_Builder, expression: *Expression, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> did_print_enum: bool`
- `is_target_type_unsigned :: (target_type: *CType = null) -> bool`
- `is_target_type_signed :: (target_type: *CType = null) -> bool`
- `append_as_unsigned_integer_hex :: (builder: *String_Builder, value: s64, target_type: *CType)`
- `append_as_signed_integer_hex :: (builder: *String_Builder, value: u64, target_type: *CType)`
- `exceeds_signed_int_value :: (value: u64, target_type: *CType) -> bool`
- `print_literal_to_builder :: (builder: *String_Builder, lit: *Literal, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> did_print_enum: bool`
- `function_has_a_valist_arg :: (func: *Function) -> bool`
- `is_va_list :: (type: *CType) -> bool`
- `Print_Decl_Flags :: enum_flags`
- `print_declaration_to_builder :: (builder: *String_Builder, _decl: *Declaration, current_scope: *Declaration, flags: Print_Decl_Flags = 0, align_declarations := 0, existing_declarations: *[..] *Declaration = null, explicit_alignment := 0)`
- `append_prefix_comment :: (builder: *String_Builder, comment: Comment, indent: bool)`
- `append_postfix_comment :: (builder: *String_Builder, comment: Comment)`
- `can_omit_type :: (decl: *Declaration, flags: Print_Decl_Flags, type_to_print: *CType) -> bool`
- `is_operator :: (decl: *Declaration) -> bool`
- `is_operator_name :: (name: string) -> bool`
- `is_assignment_operator :: (name: string) -> bool`
- `avoid_reserved_names :: (name: string, avoid_type_names := false) -> string`
- `avoid_name_clash :: (existing_declarations: *[..] *Declaration, decl: *Declaration, allow_overloads := true, insert := true, try_parent_prefix := false)`
- `find_name_clash :: (existing_declarations: [] *Declaration, decl: *Declaration, allow_overloads: bool) -> found: bool, other: *Declaration`
- `replace_operator_name :: (name: string, keep_operators_allowed_in_jai := false) -> string`
- `get_type_name :: (type: *CType, output_name := true, strip_pointers := false) -> string`
- `get_type_alignment :: (type: *CType, decl: *Declaration) -> alignment: s64, is_natural_alignment: bool`
- `print_objc_selector_identifier :: (builder: *String_Builder, ident: string)`
- `function_is_missing_foreign_library :: (func: *Function) -> bool`
- `print_function :: (builder: *String_Builder, func: *Function, current_scope: *Declaration, foreign_name: string)`
- `append_calling_convention_directives :: (builder: *String_Builder, type: *Function_Type, assume_foreign := false)`
- `is_targeting_apple_os :: () -> bool`
- `fix_macos_foreign_name :: (name: *string)`
- `append_elsewhere :: (builder: *String_Builder, decl: *Declaration, decl_name: string, foreign_name: string)`
- `get_enum_value_name :: (_enum: *Enum, name: string) -> string`
- `print_enum :: (builder: *String_Builder, _enum: *Enum)`
- `print_enum_body :: (builder: *String_Builder, _enum: *Enum)`
- `string_from_macro_value :: (value: Macro_Value, _enum: *Enum = null, name: string = "") -> string`
- `maybe_append_cast :: (builder: *String_Builder, tokens: [] CXToken, start_index: int, _enum: *Enum) -> bool, end_index: int`
- `count_asterisks :: (tokens: [] CXToken) -> int`
- `expect_tokens :: (tokens: [] CXToken, start_index: int, expected: .. string) -> bool`
- `find_closing_paren_token_index :: (tokens: [] CXToken, start_index: int) -> int`
- `find_and_check_macro_identifier :: (name: string, _enum: *Enum = null, ignore_omit_flag := false) -> found: bool, output_name: string, is_of_type: *CType, parent_enum: *Enum`
- `append_integer :: (builder: *String_Builder, value: $T, as_hex: bool)`
- `stripped_name :: (name: string) -> string`
- `is_valid_identifier_start :: (char: u8) -> bool`
- `print_struct_vtable :: (builder: *String_Builder, _struct: *Struct, extra_info: Struct_Extra_Info)`
- `Generate_Bindings_Options :: struct #type_info_none`
- `handle_toplevel_cursors :: (cur: CXCursor, parent: CXCursor, client_data: CXClientData) -> CXChildVisitResult #c_call`
- `get_name_for_cursor :: (cur: CXCursor) -> string`
- `as_str :: (cur: CXCursor, all_info := false) -> string`
- `as_str :: (ty: CXType) -> string`
- `Location :: struct`
- `fill_location :: (location: *Location, cur: CXCursor)`
- `log :: (cur: CXCursor, format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0)`
- `log_error :: (cur: CXCursor, format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0)`
- `log :: (location: Location, format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0)`
- `log_error :: (location: Location, format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0)`
- `to_string :: (location: Location) -> string`

## Compatibility constants and literal conversion

`Declaration.macro_value_result` is a nested enum with the implicit integer values `NEEDS_CHECKING = 0`, `KEEP = 1`, and `DISCARD = 2`. Its zero-initialized state is `NEEDS_CHECKING`; consumers should treat the other states as the generator's decision to preserve or omit a macro declaration.

The public name compatibility arrays have these values:

- `RESERVED_NAMES`: `cast`, `context`, `false`, `is_constant`, `null`, `remove`, `true`, `type_info`, `xx`, `interface`, `Any`, `bool`, `float`, `float32`, `float64`, `int`, `s16`, `s32`, `s64`, `s8`, `string`, `u16`, `u32`, `u64`, `u8`, `v128`, `void`.
- `RESERVED_TYPE_NAMES`: `Type`.
- `TYPEDEFS_TO_UNWRAP`: `int8_t`, `uint8_t`, `int16_t`, `uint16_t`, `int32_t`, `uint32_t`, `int64_t`, `uint64_t`, `intptr_t`, `size_t`, `ssize_t`, `__ssize_t`, `__size_t`, `off_t`, `loff_t`, `off64_t`, `daddr_t`, `__daddr_t`, `caddr_t`, `__caddr_t`, `swblk_t`, `__int8_t`, `__uint8_t`, `__int16_t`, `__uint16_t`, `__int32_t`, `__uint32_t`, `__int64_t`, `__uint64_t`, `__u8`, `__u16`, `__u32`, `__u64`, `__s8`, `__s16`, `__s32`, `__s64`, `u_int8_t`, `u_int16_t`, `u_int32_t`, `u_int64_t`, `u_char`, `u_int`, `u_short`, `u_long`, `u_quad_t`, `u_quad`, `quad_t`, `qaddr_t`, `uint`, `ushort`, `ulong`, `uchar`, `uintptr_t`, `boolean_t`, `wchar_t`, `BYTE`, `CHAR`, `UCHAR`, `DOUBLE`, `DWORD`, `DWORD_PTR`, `FLOAT`, `FLOAT32`, `FLOAT64`, `INT`, `INT16`, `INT32`, `INT64`, `INT8`, `INT_PTR`, `LONG`, `LONGLONG`, `LONG_PTR`, `PWSTR`, `LPCSTR`, `LPCVOID`, `LPCWSTR`, `LPSTR`, `LPVOID`, `LPWSTR`, `LPDWORD`, `SHORT`, `UINT`, `UINT16`, `UINT32`, `UINT64`, `UINT8`, `UINT_PTR`, `ULONG`, `ULONG64`, `ULONG_PTR`, `ULONGLONG`, `USHORT`, `WCHAR`, `WORD`, `FARPROC`, `integer_t`, `natural_t`, `SInt8`, `SInt16`, `SInt32`, `SInt64`, `UInt8`, `UInt16`, `UInt32`, `UInt64`, `UniChar`.
- `TYPEDEF_PREFIXES_TO_UNWRAP`: `__darwin_`, `__kernel_`.
- `MACROS_TO_IGNORE`: `__GCC_HAVE_DWARF2_CFI_ASM`, `WIN32_LEAN_AND_MEAN`.

`TOKENS_TO_REPLACE` is a compile-time array of two-string entries. Its mappings are:

| C token | Jai spelling |
| --- | --- |
| `FLT_MIN`, `FLT_MAX` | `FLOAT32_MIN`, `FLOAT32_MAX` |
| `DBL_MIN`, `DBL_MAX` | `FLOAT64_MIN`, `FLOAT64_MAX` |
| `SCHAR_MIN`, `SCHAR_MAX` | `S8_MIN`, `S8_MAX` |
| `UCHAR_MIN`, `UCHAR_MAX` | `0`, `U8_MAX` |
| `SHRT_MIN`, `SHRT_MAX` | `S16_MIN`, `S16_MAX` |
| `USHRT_MIN`, `USHRT_MAX` | `0`, `U16_MAX` |
| `INT_MIN`, `INT_MAX` | `S32_MIN`, `S32_MAX` |
| `UINT_MIN`, `UINT_MAX` | `0`, `U32_MAX` |
| `LLONG_MIN`, `LLONG_MAX` | `S64_MIN`, `S64_MAX` |
| `ULLONG_MIN`, `ULLONG_MAX` | `0`, `U64_MAX` |
| `INT8_MIN`, `INT8_MAX`, `UINT8_MAX` | `S8_MIN`, `S8_MAX`, `U8_MAX` |
| `INT16_MIN`, `INT16_MAX`, `UINT16_MAX` | `S16_MIN`, `S16_MAX`, `U16_MAX` |
| `INT32_MIN`, `INT32_MAX`, `UINT32_MAX` | `S32_MIN`, `S32_MAX`, `U32_MAX` |
| `INT64_MIN`, `INT64_MAX`, `UINT64_MAX` | `S64_MIN`, `S64_MAX`, `U64_MAX` |
| `LONG_MIN`, `LONG_MAX`, `ULONG_MIN`, `ULONG_MAX` on Windows | `S32_MIN`, `S32_MAX`, `0`, `U32_MAX` |
| `LONG_MIN`, `LONG_MAX`, `ULONG_MIN`, `ULONG_MAX` on other targets | `S64_MIN`, `S64_MAX`, `0`, `U64_MAX` |

`TARGET_OS` is initialized from the module parameter whose default is `xx OS`. Thus it follows the compiler target OS, rather than an unconditionally host-native setting; callers can override the parameter. `Generate_Bindings_Options.os` defaults to this `TARGET_OS`, and `.cpu` defaults to `CPU`.

For literal conversion, a failed parse returns the empty string and `success = false`. A successful result may be a slice of the caller's input or a newly formatted string; the public signature does not promise a specific allocator or ownership duration. Do not retain a returned string beyond the lifetime of its input unless the caller copies it. Integer suffixes accepted are optional `u`/`U`, then optional `z`/`Z`, `l`/`L`, or `ll`/`LL` according to the parser's suffix validation. Floating suffixes accepted are empty, `f`, or `F`. C octal literals are rewritten as hexadecimal Jai literals; binary prefixes are recognized in the leading-zero path; integer spellings with a nontrivial leading zero in the ordinary path are rejected. Hexadecimal floating literals are not supported. A quoted string is retained as-is and reports `was_string = true`, but a quote within its interior is rejected. Float conversion validates the suffix and inserts a fractional `.0` before an exponent when needed. These procedures are compatibility converters, not general C/C++ expression parsers.

### Unix-specific declarations

- `CINDEX_VERSION_MAJOR :: 0`
- `CINDEX_VERSION_MINOR :: 64`
- `CXErrorCode :: enum u32`
- `CXString :: struct`
- `CXStringSet :: struct`
- `clang_getCString :: (_string: CXString) -> *u8 #foreign libclang`
- `clang_disposeString :: (_string: CXString) -> void #foreign libclang`
- `clang_disposeStringSet :: (set: *CXStringSet) -> void #foreign libclang`
- `clang_getBuildSessionTimestamp :: () -> u64 #foreign libclang`
- `CXVirtualFileOverlayImpl :: struct`
- `CXVirtualFileOverlay :: *CXVirtualFileOverlayImpl`
- `clang_VirtualFileOverlay_create :: (options: u32) -> CXVirtualFileOverlay #foreign libclang`
- `clang_VirtualFileOverlay_addFileMapping :: (unknown0: CXVirtualFileOverlay, virtualPath: *u8, realPath: *u8) -> CXErrorCode #foreign libclang`
- `clang_VirtualFileOverlay_setCaseSensitivity :: (unknown0: CXVirtualFileOverlay, caseSensitive: s32) -> CXErrorCode #foreign libclang`
- `clang_VirtualFileOverlay_writeToBuffer :: (unknown0: CXVirtualFileOverlay, options: u32, out_buffer_ptr: **u8, out_buffer_size: *u32) -> CXErrorCode #foreign libclang`
- `clang_free :: (buffer: *void) -> void #foreign libclang`
- `clang_VirtualFileOverlay_dispose :: (unknown0: CXVirtualFileOverlay) -> void #foreign libclang`
- `CXModuleMapDescriptorImpl :: struct`
- `CXModuleMapDescriptor :: *CXModuleMapDescriptorImpl`
- `clang_ModuleMapDescriptor_create :: (options: u32) -> CXModuleMapDescriptor #foreign libclang`
- `clang_ModuleMapDescriptor_setFrameworkModuleName :: (unknown0: CXModuleMapDescriptor, name: *u8) -> CXErrorCode #foreign libclang`
- `clang_ModuleMapDescriptor_setUmbrellaHeader :: (unknown0: CXModuleMapDescriptor, name: *u8) -> CXErrorCode #foreign libclang`
- `clang_ModuleMapDescriptor_writeToBuffer :: (unknown0: CXModuleMapDescriptor, options: u32, out_buffer_ptr: **u8, out_buffer_size: *u32) -> CXErrorCode #foreign libclang`
- `clang_ModuleMapDescriptor_dispose :: (unknown0: CXModuleMapDescriptor) -> void #foreign libclang`
- `CXFile :: *void`
- `clang_getFileName :: (SFile: CXFile) -> CXString #foreign libclang`
- `clang_getFileTime :: (SFile: CXFile) -> time_t #foreign libclang`
- `CXFileUniqueID :: struct`
- `clang_getFileUniqueID :: (file: CXFile, outID: *CXFileUniqueID) -> s32 #foreign libclang`
- `clang_File_isEqual :: (file1: CXFile, file2: CXFile) -> s32 #foreign libclang`
- `clang_File_tryGetRealPathName :: (file: CXFile) -> CXString #foreign libclang`
- `CXSourceLocation :: struct`
- `CXSourceRange :: struct`
- `clang_getNullLocation :: () -> CXSourceLocation #foreign libclang`
- `clang_equalLocations :: (loc1: CXSourceLocation, loc2: CXSourceLocation) -> u32 #foreign libclang`
- `clang_isBeforeInTranslationUnit :: (loc1: CXSourceLocation, loc2: CXSourceLocation) -> u32 #foreign libclang`
- `clang_Location_isInSystemHeader :: (location: CXSourceLocation) -> s32 #foreign libclang`
- `clang_Location_isFromMainFile :: (location: CXSourceLocation) -> s32 #foreign libclang`
- `clang_getNullRange :: () -> CXSourceRange #foreign libclang`
- `clang_getRange :: (begin: CXSourceLocation, end: CXSourceLocation) -> CXSourceRange #foreign libclang`
- `clang_equalRanges :: (range1: CXSourceRange, range2: CXSourceRange) -> u32 #foreign libclang`
- `clang_Range_isNull :: (range: CXSourceRange) -> s32 #foreign libclang`
- `clang_getExpansionLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getPresumedLocation :: (location: CXSourceLocation, filename: *CXString, line: *u32, column: *u32) -> void #foreign libclang`
- `clang_getInstantiationLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getSpellingLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getFileLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getRangeStart :: (range: CXSourceRange) -> CXSourceLocation #foreign libclang`
- `clang_getRangeEnd :: (range: CXSourceRange) -> CXSourceLocation #foreign libclang`
- `CXSourceRangeList :: struct`
- `clang_disposeSourceRangeList :: (ranges: *CXSourceRangeList) -> void #foreign libclang`
- `CXDiagnosticSeverity :: enum u32`
- `CXDiagnostic :: *void`
- `CXDiagnosticSet :: *void`
- `clang_getNumDiagnosticsInSet :: (Diags: CXDiagnosticSet) -> u32 #foreign libclang`
- `clang_getDiagnosticInSet :: (Diags: CXDiagnosticSet, Index: u32) -> CXDiagnostic #foreign libclang`
- `CXLoadDiag_Error :: enum u32`
- `clang_loadDiagnostics :: (file: *u8, error: *CXLoadDiag_Error, errorString: *CXString) -> CXDiagnosticSet #foreign libclang`
- `clang_disposeDiagnosticSet :: (Diags: CXDiagnosticSet) -> void #foreign libclang`
- `clang_getChildDiagnostics :: (D: CXDiagnostic) -> CXDiagnosticSet #foreign libclang`
- `clang_disposeDiagnostic :: (Diagnostic: CXDiagnostic) -> void #foreign libclang`
- `CXDiagnosticDisplayOptions :: enum u32`
- `clang_formatDiagnostic :: (Diagnostic: CXDiagnostic, Options: u32) -> CXString #foreign libclang`
- `clang_defaultDiagnosticDisplayOptions :: () -> u32 #foreign libclang`
- `clang_getDiagnosticSeverity :: (unknown0: CXDiagnostic) -> CXDiagnosticSeverity #foreign libclang`
- `clang_getDiagnosticLocation :: (unknown0: CXDiagnostic) -> CXSourceLocation #foreign libclang`
- `clang_getDiagnosticSpelling :: (unknown0: CXDiagnostic) -> CXString #foreign libclang`
- `clang_getDiagnosticOption :: (Diag: CXDiagnostic, Disable: *CXString) -> CXString #foreign libclang`
- `clang_getDiagnosticCategory :: (unknown0: CXDiagnostic) -> u32 #foreign libclang`
- `clang_getDiagnosticCategoryName :: (Category: u32) -> CXString #foreign libclang`
- `clang_getDiagnosticCategoryText :: (unknown0: CXDiagnostic) -> CXString #foreign libclang`
- `clang_getDiagnosticNumRanges :: (unknown0: CXDiagnostic) -> u32 #foreign libclang`
- `clang_getDiagnosticRange :: (Diagnostic: CXDiagnostic, Range: u32) -> CXSourceRange #foreign libclang`
- `clang_getDiagnosticNumFixIts :: (Diagnostic: CXDiagnostic) -> u32 #foreign libclang`
- `clang_getDiagnosticFixIt :: (Diagnostic: CXDiagnostic, FixIt: u32, ReplacementRange: *CXSourceRange) -> CXString #foreign libclang`
- `CXIndex :: *void`
- `CXTargetInfoImpl :: struct`
- `CXTargetInfo :: *CXTargetInfoImpl`
- `CXTranslationUnitImpl :: struct`
- `CXTranslationUnit :: *CXTranslationUnitImpl`
- `CXClientData :: *void`
- `CXUnsavedFile :: struct`
- `CXAvailabilityKind :: enum u32`
- `CXVersion :: struct`
- `CXCursor_ExceptionSpecificationKind :: enum u32`
- `clang_createIndex :: (excludeDeclarationsFromPCH: s32, displayDiagnostics: s32) -> CXIndex #foreign libclang`
- `clang_disposeIndex :: (index: CXIndex) -> void #foreign libclang`
- `CXChoice :: enum u32`
- `CXGlobalOptFlags :: enum u32`
- `CXIndexOptions :: struct`
- `set_ExcludeDeclarationsFromPCH :: (fields: *CXIndexOptions, value: bool)`
- `get_ExcludeDeclarationsFromPCH :: (fields: CXIndexOptions) -> bool`
- `set_DisplayDiagnostics :: (fields: *CXIndexOptions, value: bool)`
- `get_DisplayDiagnostics :: (fields: CXIndexOptions) -> bool`
- `set_StorePreamblesInMemory :: (fields: *CXIndexOptions, value: bool)`
- `get_StorePreamblesInMemory :: (fields: CXIndexOptions) -> bool`
- `clang_createIndexWithOptions :: (options: *CXIndexOptions) -> CXIndex #foreign libclang`
- `clang_CXIndex_setGlobalOptions :: (unknown0: CXIndex, options: u32) -> void #foreign libclang`
- `clang_CXIndex_getGlobalOptions :: (unknown0: CXIndex) -> u32 #foreign libclang`
- `clang_CXIndex_setInvocationEmissionPathOption :: (unknown0: CXIndex, Path: *u8) -> void #foreign libclang`
- `clang_isFileMultipleIncludeGuarded :: (tu: CXTranslationUnit, file: CXFile) -> u32 #foreign libclang`
- `clang_getFile :: (tu: CXTranslationUnit, file_name: *u8) -> CXFile #foreign libclang`
- `clang_getFileContents :: (tu: CXTranslationUnit, file: CXFile, size: *u64) -> *u8 #foreign libclang`
- `clang_getLocation :: (tu: CXTranslationUnit, file: CXFile, line: u32, column: u32) -> CXSourceLocation #foreign libclang`
- `clang_getLocationForOffset :: (tu: CXTranslationUnit, file: CXFile, offset: u32) -> CXSourceLocation #foreign libclang`
- `clang_getSkippedRanges :: (tu: CXTranslationUnit, file: CXFile) -> *CXSourceRangeList #foreign libclang`
- `clang_getAllSkippedRanges :: (tu: CXTranslationUnit) -> *CXSourceRangeList #foreign libclang`
- `clang_getNumDiagnostics :: (Unit: CXTranslationUnit) -> u32 #foreign libclang`
- `clang_getDiagnostic :: (Unit: CXTranslationUnit, Index: u32) -> CXDiagnostic #foreign libclang`
- `clang_getDiagnosticSetFromTU :: (Unit: CXTranslationUnit) -> CXDiagnosticSet #foreign libclang`
- `clang_getTranslationUnitSpelling :: (CTUnit: CXTranslationUnit) -> CXString #foreign libclang`
- `clang_createTranslationUnitFromSourceFile :: (CIdx: CXIndex, source_filename: *u8, num_clang_command_line_args: s32, clang_command_line_args: **u8, num_unsaved_files: u32, unsaved_files: *CXUnsavedFile) -> CXTranslationUnit #foreign libclang`
- `clang_createTranslationUnit :: (CIdx: CXIndex, ast_filename: *u8) -> CXTranslationUnit #foreign libclang`
- `clang_createTranslationUnit2 :: (CIdx: CXIndex, ast_filename: *u8, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang`
- `CXTranslationUnit_Flags :: enum_flags u32`
- `clang_defaultEditingTranslationUnitOptions :: () -> u32 #foreign libclang`
- `clang_parseTranslationUnit :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32) -> CXTranslationUnit #foreign libclang`
- `clang_parseTranslationUnit2 :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang`
- `clang_parseTranslationUnit2FullArgv :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang`
- `CXSaveTranslationUnit_Flags :: enum_flags u32`
- `clang_defaultSaveOptions :: (TU: CXTranslationUnit) -> u32 #foreign libclang`
- `CXSaveError :: enum u32`
- `clang_saveTranslationUnit :: (TU: CXTranslationUnit, FileName: *u8, options: u32) -> s32 #foreign libclang`
- `clang_suspendTranslationUnit :: (unknown0: CXTranslationUnit) -> u32 #foreign libclang`
- `clang_disposeTranslationUnit :: (unknown0: CXTranslationUnit) -> void #foreign libclang`
- `CXReparse_Flags :: enum_flags u32`
- `clang_defaultReparseOptions :: (TU: CXTranslationUnit) -> u32 #foreign libclang`
- `clang_reparseTranslationUnit :: (TU: CXTranslationUnit, num_unsaved_files: u32, unsaved_files: *CXUnsavedFile, options: u32) -> s32 #foreign libclang`
- `CXTUResourceUsageKind :: enum u32`
- `clang_getTUResourceUsageName :: (kind: CXTUResourceUsageKind) -> *u8 #foreign libclang`
- `CXTUResourceUsageEntry :: struct`
- `CXTUResourceUsage :: struct`
- `clang_getCXTUResourceUsage :: (TU: CXTranslationUnit) -> CXTUResourceUsage #foreign libclang`
- `clang_disposeCXTUResourceUsage :: (usage: CXTUResourceUsage) -> void #foreign libclang`
- `clang_getTranslationUnitTargetInfo :: (CTUnit: CXTranslationUnit) -> CXTargetInfo #foreign libclang`
- `clang_TargetInfo_dispose :: (Info: CXTargetInfo) -> void #foreign libclang`
- `clang_TargetInfo_getTriple :: (Info: CXTargetInfo) -> CXString #foreign libclang`
- `clang_TargetInfo_getPointerWidth :: (Info: CXTargetInfo) -> s32 #foreign libclang`
- `CXCursorKind :: enum u32`
- `CXCursor :: struct`
- `clang_getNullCursor :: () -> CXCursor #foreign libclang`
- `clang_getTranslationUnitCursor :: (unknown0: CXTranslationUnit) -> CXCursor #foreign libclang`
- `clang_equalCursors :: (unknown0: CXCursor, unknown1: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isNull :: (cursor: CXCursor) -> s32 #foreign libclang`
- `clang_hashCursor :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_getCursorKind :: (unknown0: CXCursor) -> CXCursorKind #foreign libclang`
- `clang_isDeclaration :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isInvalidDeclaration :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_isReference :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isExpression :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isStatement :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isAttribute :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_Cursor_hasAttrs :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_isInvalid :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isTranslationUnit :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isPreprocessing :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isUnexposed :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `CXLinkageKind :: enum u32`
- `clang_getCursorLinkage :: (cursor: CXCursor) -> CXLinkageKind #foreign libclang`
- `CXVisibilityKind :: enum u32`
- `clang_getCursorVisibility :: (cursor: CXCursor) -> CXVisibilityKind #foreign libclang`
- `clang_getCursorAvailability :: (cursor: CXCursor) -> CXAvailabilityKind #foreign libclang`
- `CXPlatformAvailability :: struct`
- `clang_getCursorPlatformAvailability :: (cursor: CXCursor, always_deprecated: *s32, deprecated_message: *CXString, always_unavailable: *s32, unavailable_message: *CXString, availability: *CXPlatformAvailability, availability_size: s32) -> s32 #foreign libclang`
- `clang_disposeCXPlatformAvailability :: (availability: *CXPlatformAvailability) -> void #foreign libclang`
- `CXLanguageKind :: enum u32`
- `clang_getCursorLanguage :: (cursor: CXCursor) -> CXLanguageKind #foreign libclang`
- `CXTLSKind :: enum u32`
- `clang_getCursorTLSKind :: (cursor: CXCursor) -> CXTLSKind #foreign libclang`
- `clang_Cursor_getTranslationUnit :: (unknown0: CXCursor) -> CXTranslationUnit #foreign libclang`
- `CXCursorSetImpl :: struct`
- `CXCursorSet :: *CXCursorSetImpl`
- `clang_createCXCursorSet :: () -> CXCursorSet #foreign libclang`
- `clang_disposeCXCursorSet :: (cset: CXCursorSet) -> void #foreign libclang`
- `clang_CXCursorSet_contains :: (cset: CXCursorSet, cursor: CXCursor) -> u32 #foreign libclang`
- `clang_CXCursorSet_insert :: (cset: CXCursorSet, cursor: CXCursor) -> u32 #foreign libclang`
- `clang_getCursorSemanticParent :: (cursor: CXCursor) -> CXCursor #foreign libclang`
- `clang_getCursorLexicalParent :: (cursor: CXCursor) -> CXCursor #foreign libclang`
- `clang_getOverriddenCursors :: (cursor: CXCursor, overridden: **CXCursor, num_overridden: *u32) -> void #foreign libclang`
- `clang_disposeOverriddenCursors :: (overridden: *CXCursor) -> void #foreign libclang`
- `clang_getIncludedFile :: (cursor: CXCursor) -> CXFile #foreign libclang`
- `clang_getCursor :: (unknown0: CXTranslationUnit, unknown1: CXSourceLocation) -> CXCursor #foreign libclang`
- `clang_getCursorLocation :: (unknown0: CXCursor) -> CXSourceLocation #foreign libclang`
- `clang_getCursorExtent :: (unknown0: CXCursor) -> CXSourceRange #foreign libclang`
- `CXTypeKind :: enum u32`
- `CXCallingConv :: enum u32`
- `CXType :: struct`
- `clang_getCursorType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getTypeSpelling :: (CT: CXType) -> CXString #foreign libclang`
- `clang_getTypedefDeclUnderlyingType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getEnumDeclIntegerType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getEnumConstantDeclValue :: (C: CXCursor) -> s64 #foreign libclang`
- `clang_getEnumConstantDeclUnsignedValue :: (C: CXCursor) -> u64 #foreign libclang`
- `clang_Cursor_isBitField :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_getFieldDeclBitWidth :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getNumArguments :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getArgument :: (C: CXCursor, i: u32) -> CXCursor #foreign libclang`
- `CXTemplateArgumentKind :: enum u32`
- `clang_Cursor_getNumTemplateArguments :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getTemplateArgumentKind :: (C: CXCursor, I: u32) -> CXTemplateArgumentKind #foreign libclang`
- `clang_Cursor_getTemplateArgumentType :: (C: CXCursor, I: u32) -> CXType #foreign libclang`
- `clang_Cursor_getTemplateArgumentValue :: (C: CXCursor, I: u32) -> s64 #foreign libclang`
- `clang_Cursor_getTemplateArgumentUnsignedValue :: (C: CXCursor, I: u32) -> u64 #foreign libclang`
- `clang_equalTypes :: (A: CXType, B: CXType) -> u32 #foreign libclang`
- `clang_getCanonicalType :: (T: CXType) -> CXType #foreign libclang`
- `clang_isConstQualifiedType :: (T: CXType) -> u32 #foreign libclang`
- `clang_Cursor_isMacroFunctionLike :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isMacroBuiltin :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isFunctionInlined :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_isVolatileQualifiedType :: (T: CXType) -> u32 #foreign libclang`
- `clang_isRestrictQualifiedType :: (T: CXType) -> u32 #foreign libclang`
- `clang_getAddressSpace :: (T: CXType) -> u32 #foreign libclang`
- `clang_getTypedefName :: (CT: CXType) -> CXString #foreign libclang`
- `clang_getPointeeType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getUnqualifiedType :: (CT: CXType) -> CXType #foreign libclang`
- `clang_getNonReferenceType :: (CT: CXType) -> CXType #foreign libclang`
- `clang_getTypeDeclaration :: (T: CXType) -> CXCursor #foreign libclang`
- `clang_getDeclObjCTypeEncoding :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Type_getObjCEncoding :: (type: CXType) -> CXString #foreign libclang`
- `clang_getTypeKindSpelling :: (K: CXTypeKind) -> CXString #foreign libclang`
- `clang_getFunctionTypeCallingConv :: (T: CXType) -> CXCallingConv #foreign libclang`
- `clang_getResultType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getExceptionSpecificationType :: (T: CXType) -> s32 #foreign libclang`
- `clang_getNumArgTypes :: (T: CXType) -> s32 #foreign libclang`
- `clang_getArgType :: (T: CXType, i: u32) -> CXType #foreign libclang`
- `clang_Type_getObjCObjectBaseType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_getNumObjCProtocolRefs :: (T: CXType) -> u32 #foreign libclang`
- `clang_Type_getObjCProtocolDecl :: (T: CXType, i: u32) -> CXCursor #foreign libclang`
- `clang_Type_getNumObjCTypeArgs :: (T: CXType) -> u32 #foreign libclang`
- `clang_Type_getObjCTypeArg :: (T: CXType, i: u32) -> CXType #foreign libclang`
- `clang_isFunctionTypeVariadic :: (T: CXType) -> u32 #foreign libclang`
- `clang_getCursorResultType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getCursorExceptionSpecificationType :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_isPODType :: (T: CXType) -> u32 #foreign libclang`
- `clang_getElementType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getNumElements :: (T: CXType) -> s64 #foreign libclang`
- `clang_getArrayElementType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getArraySize :: (T: CXType) -> s64 #foreign libclang`
- `clang_Type_getNamedType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_isTransparentTagTypedef :: (T: CXType) -> u32 #foreign libclang`
- `CXTypeNullabilityKind :: enum u32`
- `clang_Type_getNullability :: (T: CXType) -> CXTypeNullabilityKind #foreign libclang`
- `CXTypeLayoutError :: enum s32`
- `clang_Type_getAlignOf :: (T: CXType) -> s64 #foreign libclang`
- `clang_Type_getClassType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_getSizeOf :: (T: CXType) -> s64 #foreign libclang`
- `clang_Type_getOffsetOf :: (T: CXType, S: *u8) -> s64 #foreign libclang`
- `clang_Type_getModifiedType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_getValueType :: (CT: CXType) -> CXType #foreign libclang`
- `clang_Cursor_getOffsetOfField :: (C: CXCursor) -> s64 #foreign libclang`
- `clang_Cursor_isAnonymous :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isAnonymousRecordDecl :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isInlineNamespace :: (C: CXCursor) -> u32 #foreign libclang`
- `CXRefQualifierKind :: enum u32`
- `clang_Type_getNumTemplateArguments :: (T: CXType) -> s32 #foreign libclang`
- `clang_Type_getTemplateArgumentAsType :: (T: CXType, i: u32) -> CXType #foreign libclang`
- `clang_Type_getCXXRefQualifier :: (T: CXType) -> CXRefQualifierKind #foreign libclang`
- `clang_isVirtualBase :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_getOffsetOfBase :: (Parent: CXCursor, Base: CXCursor) -> s64 #foreign libclang`
- `CX_CXXAccessSpecifier :: enum u32`
- `clang_getCXXAccessSpecifier :: (unknown0: CXCursor) -> CX_CXXAccessSpecifier #foreign libclang`
- `CX_StorageClass :: enum u32`
- `CX_BinaryOperatorKind :: enum u32`
- `clang_Cursor_getBinaryOpcode :: (C: CXCursor) -> CX_BinaryOperatorKind #foreign libclang`
- `clang_Cursor_getBinaryOpcodeStr :: (Op: CX_BinaryOperatorKind) -> CXString #foreign libclang`
- `clang_Cursor_getStorageClass :: (unknown0: CXCursor) -> CX_StorageClass #foreign libclang`
- `clang_getNumOverloadedDecls :: (cursor: CXCursor) -> u32 #foreign libclang`
- `clang_getOverloadedDecl :: (cursor: CXCursor, index: u32) -> CXCursor #foreign libclang`
- `clang_getIBOutletCollectionType :: (unknown0: CXCursor) -> CXType #foreign libclang`
- `CXChildVisitResult :: enum u32`
- `CXCursorVisitor :: #type (cursor: CXCursor, parent: CXCursor, client_data: CXClientData) -> CXChildVisitResult #c_call`
- `clang_visitChildren :: (parent: CXCursor, visitor: CXCursorVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `clang_getCursorUSR :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCClass :: (class_name: *u8) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCCategory :: (class_name: *u8, category_name: *u8) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCProtocol :: (protocol_name: *u8) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCIvar :: (name: *u8, classUSR: CXString) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCMethod :: (name: *u8, isInstanceMethod: u32, classUSR: CXString) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCProperty :: (property: *u8, classUSR: CXString) -> CXString #foreign libclang`
- `clang_getCursorSpelling :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getSpellingNameRange :: (unknown0: CXCursor, pieceIndex: u32, options: u32) -> CXSourceRange #foreign libclang`
- `CXPrintingPolicy :: *void`
- `CXPrintingPolicyProperty :: enum u32`
- `clang_PrintingPolicy_getProperty :: (Policy: CXPrintingPolicy, Property: CXPrintingPolicyProperty) -> u32 #foreign libclang`
- `clang_PrintingPolicy_setProperty :: (Policy: CXPrintingPolicy, Property: CXPrintingPolicyProperty, Value: u32) -> void #foreign libclang`
- `clang_getCursorPrintingPolicy :: (unknown0: CXCursor) -> CXPrintingPolicy #foreign libclang`
- `clang_PrintingPolicy_dispose :: (Policy: CXPrintingPolicy) -> void #foreign libclang`
- `clang_getCursorPrettyPrinted :: (Cursor: CXCursor, Policy: CXPrintingPolicy) -> CXString #foreign libclang`
- `clang_getTypePrettyPrinted :: (CT: CXType, cxPolicy: CXPrintingPolicy) -> CXString #foreign libclang`
- `clang_getFullyQualifiedName :: (CT: CXType, Policy: CXPrintingPolicy, WithGlobalNsPrefix: u32) -> CXString #foreign libclang`
- `clang_getCursorDisplayName :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_getCursorReferenced :: (unknown0: CXCursor) -> CXCursor #foreign libclang`
- `clang_getCursorDefinition :: (unknown0: CXCursor) -> CXCursor #foreign libclang`
- `clang_isCursorDefinition :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_getCanonicalCursor :: (unknown0: CXCursor) -> CXCursor #foreign libclang`
- `clang_Cursor_getObjCSelectorIndex :: (unknown0: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_isDynamicCall :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getReceiverType :: (C: CXCursor) -> CXType #foreign libclang`
- `CXObjCPropertyAttrKind :: enum u32`
- `clang_Cursor_getObjCPropertyAttributes :: (C: CXCursor, reserved: u32) -> u32 #foreign libclang`
- `clang_Cursor_getObjCPropertyGetterName :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getObjCPropertySetterName :: (C: CXCursor) -> CXString #foreign libclang`
- `CXObjCDeclQualifierKind :: enum u32`
- `clang_Cursor_getObjCDeclQualifiers :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isObjCOptional :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isVariadic :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isExternalSymbol :: (C: CXCursor, language: *CXString, definedIn: *CXString, isGenerated: *u32) -> u32 #foreign libclang`
- `clang_Cursor_getCommentRange :: (C: CXCursor) -> CXSourceRange #foreign libclang`
- `clang_Cursor_getRawCommentText :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getBriefCommentText :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getMangling :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getCXXManglings :: (unknown0: CXCursor) -> *CXStringSet #foreign libclang`
- `clang_Cursor_getObjCManglings :: (unknown0: CXCursor) -> *CXStringSet #foreign libclang`
- `clang_Cursor_getGCCAssemblyTemplate :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_isGCCAssemblyHasGoto :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyNumOutputs :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyNumInputs :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyInput :: (Cursor: CXCursor, Index: u32, Constraint: *CXString, Expr: *CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyOutput :: (Cursor: CXCursor, Index: u32, Constraint: *CXString, Expr: *CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyNumClobbers :: (Cursor: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyClobber :: (Cursor: CXCursor, Index: u32) -> CXString #foreign libclang`
- `clang_Cursor_isGCCAssemblyVolatile :: (Cursor: CXCursor) -> u32 #foreign libclang`
- `CXModule :: *void`
- `clang_Cursor_getModule :: (C: CXCursor) -> CXModule #foreign libclang`
- `clang_getModuleForFile :: (unknown0: CXTranslationUnit, unknown1: CXFile) -> CXModule #foreign libclang`
- `clang_Module_getASTFile :: (Module: CXModule) -> CXFile #foreign libclang`
- `clang_Module_getParent :: (Module: CXModule) -> CXModule #foreign libclang`
- `clang_Module_getName :: (Module: CXModule) -> CXString #foreign libclang`
- `clang_Module_getFullName :: (Module: CXModule) -> CXString #foreign libclang`
- `clang_Module_isSystem :: (Module: CXModule) -> s32 #foreign libclang`
- `clang_Module_getNumTopLevelHeaders :: (unknown0: CXTranslationUnit, Module: CXModule) -> u32 #foreign libclang`
- `clang_Module_getTopLevelHeader :: (unknown0: CXTranslationUnit, Module: CXModule, Index: u32) -> CXFile #foreign libclang`
- `clang_CXXConstructor_isConvertingConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXConstructor_isCopyConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXConstructor_isDefaultConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXConstructor_isMoveConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXField_isMutable :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isDefaulted :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isDeleted :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isPureVirtual :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isStatic :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isVirtual :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isCopyAssignmentOperator :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isMoveAssignmentOperator :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isExplicit :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXRecord_isAbstract :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_EnumDecl_isScoped :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isConst :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_getTemplateCursorKind :: (C: CXCursor) -> CXCursorKind #foreign libclang`
- `clang_getSpecializedCursorTemplate :: (C: CXCursor) -> CXCursor #foreign libclang`
- `clang_getCursorReferenceNameRange :: (C: CXCursor, NameFlags: u32, PieceIndex: u32) -> CXSourceRange #foreign libclang`
- `CXNameRefFlags :: enum u32`
- `CXTokenKind :: enum u32`
- `CXToken :: struct`
- `clang_getToken :: (TU: CXTranslationUnit, Location: CXSourceLocation) -> *CXToken #foreign libclang`
- `clang_getTokenKind :: (unknown0: CXToken) -> CXTokenKind #foreign libclang`
- `clang_getTokenSpelling :: (unknown0: CXTranslationUnit, unknown1: CXToken) -> CXString #foreign libclang`
- `clang_getTokenLocation :: (unknown0: CXTranslationUnit, unknown1: CXToken) -> CXSourceLocation #foreign libclang`
- `clang_getTokenExtent :: (unknown0: CXTranslationUnit, unknown1: CXToken) -> CXSourceRange #foreign libclang`
- `clang_tokenize :: (TU: CXTranslationUnit, Range: CXSourceRange, Tokens: **CXToken, NumTokens: *u32) -> void #foreign libclang`
- `clang_annotateTokens :: (TU: CXTranslationUnit, Tokens: *CXToken, NumTokens: u32, Cursors: *CXCursor) -> void #foreign libclang`
- `clang_disposeTokens :: (TU: CXTranslationUnit, Tokens: *CXToken, NumTokens: u32) -> void #foreign libclang`
- `clang_getCursorKindSpelling :: (Kind: CXCursorKind) -> CXString #foreign libclang`
- `clang_getDefinitionSpellingAndExtent :: (unknown0: CXCursor, startBuf: **u8, endBuf: **u8, startLine: *u32, startColumn: *u32, endLine: *u32, endColumn: *u32) -> void #foreign libclang`
- `clang_enableStackTraces :: () -> void #foreign libclang`
- `clang_executeOnThread :: (fn: #type (unknown0: *void) -> void #c_call, user_data: *void, stack_size: u32) -> void #foreign libclang`
- `CXCompletionString :: *void`
- `CXCompletionResult :: struct`
- `CXCompletionChunkKind :: enum u32`
- `clang_getCompletionChunkKind :: (completion_string: CXCompletionString, chunk_number: u32) -> CXCompletionChunkKind #foreign libclang`
- `clang_getCompletionChunkText :: (completion_string: CXCompletionString, chunk_number: u32) -> CXString #foreign libclang`
- `clang_getCompletionChunkCompletionString :: (completion_string: CXCompletionString, chunk_number: u32) -> CXCompletionString #foreign libclang`
- `clang_getNumCompletionChunks :: (completion_string: CXCompletionString) -> u32 #foreign libclang`
- `clang_getCompletionPriority :: (completion_string: CXCompletionString) -> u32 #foreign libclang`
- `clang_getCompletionAvailability :: (completion_string: CXCompletionString) -> CXAvailabilityKind #foreign libclang`
- `clang_getCompletionNumAnnotations :: (completion_string: CXCompletionString) -> u32 #foreign libclang`
- `clang_getCompletionAnnotation :: (completion_string: CXCompletionString, annotation_number: u32) -> CXString #foreign libclang`
- `clang_getCompletionParent :: (completion_string: CXCompletionString, kind: *CXCursorKind) -> CXString #foreign libclang`
- `clang_getCompletionBriefComment :: (completion_string: CXCompletionString) -> CXString #foreign libclang`
- `clang_getCursorCompletionString :: (cursor: CXCursor) -> CXCompletionString #foreign libclang`
- `CXCodeCompleteResults :: struct`
- `clang_getCompletionNumFixIts :: (results: *CXCodeCompleteResults, completion_index: u32) -> u32 #foreign libclang`
- `clang_getCompletionFixIt :: (results: *CXCodeCompleteResults, completion_index: u32, fixit_index: u32, replacement_range: *CXSourceRange) -> CXString #foreign libclang`
- `CXCodeComplete_Flags :: enum_flags u32`
- `CXCompletionContext :: enum_flags u32`
- `clang_defaultCodeCompleteOptions :: () -> u32 #foreign libclang`
- `clang_codeCompleteAt :: (TU: CXTranslationUnit, complete_filename: *u8, complete_line: u32, complete_column: u32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32) -> *CXCodeCompleteResults #foreign libclang`
- `clang_sortCodeCompletionResults :: (Results: *CXCompletionResult, NumResults: u32) -> void #foreign libclang`
- `clang_disposeCodeCompleteResults :: (Results: *CXCodeCompleteResults) -> void #foreign libclang`
- `clang_codeCompleteGetNumDiagnostics :: (Results: *CXCodeCompleteResults) -> u32 #foreign libclang`
- `clang_codeCompleteGetDiagnostic :: (Results: *CXCodeCompleteResults, Index: u32) -> CXDiagnostic #foreign libclang`
- `clang_codeCompleteGetContexts :: (Results: *CXCodeCompleteResults) -> u64 #foreign libclang`
- `clang_codeCompleteGetContainerKind :: (Results: *CXCodeCompleteResults, IsIncomplete: *u32) -> CXCursorKind #foreign libclang`
- `clang_codeCompleteGetContainerUSR :: (Results: *CXCodeCompleteResults) -> CXString #foreign libclang`
- `clang_codeCompleteGetObjCSelector :: (Results: *CXCodeCompleteResults) -> CXString #foreign libclang`
- `clang_getClangVersion :: () -> CXString #foreign libclang`
- `clang_toggleCrashRecovery :: (isEnabled: u32) -> void #foreign libclang`
- `CXInclusionVisitor :: #type (included_file: CXFile, inclusion_stack: *CXSourceLocation, include_len: u32, client_data: CXClientData) -> void #c_call`
- `clang_getInclusions :: (tu: CXTranslationUnit, visitor: CXInclusionVisitor, client_data: CXClientData) -> void #foreign libclang`
- `CXEvalResultKind :: enum u32`
- `CXEvalResult :: *void`
- `clang_Cursor_Evaluate :: (C: CXCursor) -> CXEvalResult #foreign libclang`
- `clang_EvalResult_getKind :: (E: CXEvalResult) -> CXEvalResultKind #foreign libclang`
- `clang_EvalResult_getAsInt :: (E: CXEvalResult) -> s32 #foreign libclang`
- `clang_EvalResult_getAsLongLong :: (E: CXEvalResult) -> s64 #foreign libclang`
- `clang_EvalResult_isUnsignedInt :: (E: CXEvalResult) -> u32 #foreign libclang`
- `clang_EvalResult_getAsUnsigned :: (E: CXEvalResult) -> u64 #foreign libclang`
- `clang_EvalResult_getAsDouble :: (E: CXEvalResult) -> float64 #foreign libclang`
- `clang_EvalResult_getAsStr :: (E: CXEvalResult) -> *u8 #foreign libclang`
- `clang_EvalResult_dispose :: (E: CXEvalResult) -> void #foreign libclang`
- `CXVisitorResult :: enum u32`
- `CXCursorAndRangeVisitor :: struct`
- `CXResult :: enum u32`
- `clang_findReferencesInFile :: (cursor: CXCursor, file: CXFile, visitor: CXCursorAndRangeVisitor) -> CXResult #foreign libclang`
- `clang_findIncludesInFile :: (TU: CXTranslationUnit, file: CXFile, visitor: CXCursorAndRangeVisitor) -> CXResult #foreign libclang`
- `CXIdxClientFile :: *void`
- `CXIdxClientEntity :: *void`
- `CXIdxClientContainer :: *void`
- `CXIdxClientASTFile :: *void`
- `CXIdxLoc :: struct`
- `CXIdxIncludedFileInfo :: struct`
- `CXIdxImportedASTFileInfo :: struct`
- `CXIdxEntityKind :: enum u32`
- `CXIdxEntityLanguage :: enum u32`
- `CXIdxEntityCXXTemplateKind :: enum u32`
- `CXIdxAttrKind :: enum u32`
- `CXIdxAttrInfo :: struct`
- `CXIdxEntityInfo :: struct`
- `CXIdxContainerInfo :: struct`
- `CXIdxIBOutletCollectionAttrInfo :: struct`
- `CXIdxDeclInfoFlags :: enum u32`
- `CXIdxDeclInfo :: struct`
- `CXIdxObjCContainerKind :: enum u32`
- `CXIdxObjCContainerDeclInfo :: struct`
- `CXIdxBaseClassInfo :: struct`
- `CXIdxObjCProtocolRefInfo :: struct`
- `CXIdxObjCProtocolRefListInfo :: struct`
- `CXIdxObjCInterfaceDeclInfo :: struct`
- `CXIdxObjCCategoryDeclInfo :: struct`
- `CXIdxObjCPropertyDeclInfo :: struct`
- `CXIdxCXXClassDeclInfo :: struct`
- `CXIdxEntityRefKind :: enum u32`
- `CXSymbolRole :: enum u32`
- `CXIdxEntityRefInfo :: struct`
- `IndexerCallbacks :: struct`
- `clang_index_isEntityObjCContainerKind :: (unknown0: CXIdxEntityKind) -> s32 #foreign libclang`
- `clang_index_getObjCContainerDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCContainerDeclInfo #foreign libclang`
- `clang_index_getObjCInterfaceDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCInterfaceDeclInfo #foreign libclang`
- `clang_index_getObjCCategoryDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCCategoryDeclInfo #foreign libclang`
- `clang_index_getObjCProtocolRefListInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCProtocolRefListInfo #foreign libclang`
- `clang_index_getObjCPropertyDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCPropertyDeclInfo #foreign libclang`
- `clang_index_getIBOutletCollectionAttrInfo :: (unknown0: *CXIdxAttrInfo) -> *CXIdxIBOutletCollectionAttrInfo #foreign libclang`
- `clang_index_getCXXClassDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxCXXClassDeclInfo #foreign libclang`
- `clang_index_getClientContainer :: (unknown0: *CXIdxContainerInfo) -> CXIdxClientContainer #foreign libclang`
- `clang_index_setClientContainer :: (unknown0: *CXIdxContainerInfo, unknown1: CXIdxClientContainer) -> void #foreign libclang`
- `clang_index_getClientEntity :: (unknown0: *CXIdxEntityInfo) -> CXIdxClientEntity #foreign libclang`
- `clang_index_setClientEntity :: (unknown0: *CXIdxEntityInfo, unknown1: CXIdxClientEntity) -> void #foreign libclang`
- `CXIndexAction :: *void`
- `clang_IndexAction_create :: (CIdx: CXIndex) -> CXIndexAction #foreign libclang`
- `clang_IndexAction_dispose :: (unknown0: CXIndexAction) -> void #foreign libclang`
- `CXIndexOptFlags :: enum u32`
- `clang_indexSourceFile :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, out_TU: *CXTranslationUnit, TU_options: u32) -> s32 #foreign libclang`
- `clang_indexSourceFileFullArgv :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, out_TU: *CXTranslationUnit, TU_options: u32) -> s32 #foreign libclang`
- `clang_indexTranslationUnit :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, unknown1: CXTranslationUnit) -> s32 #foreign libclang`
- `clang_indexLoc_getFileLocation :: (loc: CXIdxLoc, indexFile: *CXIdxClientFile, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_indexLoc_getCXSourceLocation :: (loc: CXIdxLoc) -> CXSourceLocation #foreign libclang`
- `CXFieldVisitor :: #type (C: CXCursor, client_data: CXClientData) -> CXVisitorResult #c_call`
- `clang_Type_visitFields :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `clang_visitCXXBaseClasses :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `clang_visitCXXMethods :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `CXBinaryOperatorKind :: enum u32`
- `clang_getBinaryOperatorKindSpelling :: (kind: CXBinaryOperatorKind) -> CXString #foreign libclang`
- `clang_getCursorBinaryOperatorKind :: (cursor: CXCursor) -> CXBinaryOperatorKind #foreign libclang`
- `CXUnaryOperatorKind :: enum u32`
- `clang_getUnaryOperatorKindSpelling :: (kind: CXUnaryOperatorKind) -> CXString #foreign libclang`
- `clang_getCursorUnaryOperatorKind :: (cursor: CXCursor) -> CXUnaryOperatorKind #foreign libclang`
- `CXRemapping :: *void`
- `clang_getRemappings :: (unknown0: *u8) -> CXRemapping #foreign libclang`
- `clang_getRemappingsFromFileList :: (unknown0: **u8, unknown1: u32) -> CXRemapping #foreign libclang`
- `clang_remap_getNumFiles :: (unknown0: CXRemapping) -> u32 #foreign libclang`
- `clang_remap_getFilenames :: (unknown0: CXRemapping, unknown1: u32, unknown2: *CXString, unknown3: *CXString) -> void #foreign libclang`
- `clang_remap_dispose :: (unknown0: CXRemapping) -> void #foreign libclang`

### Windows-specific declarations

- `CINDEX_VERSION_MAJOR :: 0`
- `CINDEX_VERSION_MINOR :: 64`
- `CXErrorCode :: enum s32`
- `CXString :: struct`
- `CXStringSet :: struct`
- `clang_getCString :: (_string: CXString) -> *u8 #foreign libclang`
- `clang_disposeString :: (_string: CXString) -> void #foreign libclang`
- `clang_disposeStringSet :: (set: *CXStringSet) -> void #foreign libclang`
- `clang_getBuildSessionTimestamp :: () -> u64 #foreign libclang`
- `CXVirtualFileOverlayImpl :: struct`
- `CXVirtualFileOverlay :: *CXVirtualFileOverlayImpl`
- `clang_VirtualFileOverlay_create :: (options: u32) -> CXVirtualFileOverlay #foreign libclang`
- `clang_VirtualFileOverlay_addFileMapping :: (unknown0: CXVirtualFileOverlay, virtualPath: *u8, realPath: *u8) -> CXErrorCode #foreign libclang`
- `clang_VirtualFileOverlay_setCaseSensitivity :: (unknown0: CXVirtualFileOverlay, caseSensitive: s32) -> CXErrorCode #foreign libclang`
- `clang_VirtualFileOverlay_writeToBuffer :: (unknown0: CXVirtualFileOverlay, options: u32, out_buffer_ptr: **u8, out_buffer_size: *u32) -> CXErrorCode #foreign libclang`
- `clang_free :: (buffer: *void) -> void #foreign libclang`
- `clang_VirtualFileOverlay_dispose :: (unknown0: CXVirtualFileOverlay) -> void #foreign libclang`
- `CXModuleMapDescriptorImpl :: struct`
- `CXModuleMapDescriptor :: *CXModuleMapDescriptorImpl`
- `clang_ModuleMapDescriptor_create :: (options: u32) -> CXModuleMapDescriptor #foreign libclang`
- `clang_ModuleMapDescriptor_setFrameworkModuleName :: (unknown0: CXModuleMapDescriptor, name: *u8) -> CXErrorCode #foreign libclang`
- `clang_ModuleMapDescriptor_setUmbrellaHeader :: (unknown0: CXModuleMapDescriptor, name: *u8) -> CXErrorCode #foreign libclang`
- `clang_ModuleMapDescriptor_writeToBuffer :: (unknown0: CXModuleMapDescriptor, options: u32, out_buffer_ptr: **u8, out_buffer_size: *u32) -> CXErrorCode #foreign libclang`
- `clang_ModuleMapDescriptor_dispose :: (unknown0: CXModuleMapDescriptor) -> void #foreign libclang`
- `CXFile :: *void`
- `clang_getFileName :: (SFile: CXFile) -> CXString #foreign libclang`
- `clang_getFileTime :: (SFile: CXFile) -> time_t #foreign libclang`
- `CXFileUniqueID :: struct`
- `clang_getFileUniqueID :: (file: CXFile, outID: *CXFileUniqueID) -> s32 #foreign libclang`
- `clang_File_isEqual :: (file1: CXFile, file2: CXFile) -> s32 #foreign libclang`
- `clang_File_tryGetRealPathName :: (file: CXFile) -> CXString #foreign libclang`
- `CXSourceLocation :: struct`
- `CXSourceRange :: struct`
- `clang_getNullLocation :: () -> CXSourceLocation #foreign libclang`
- `clang_equalLocations :: (loc1: CXSourceLocation, loc2: CXSourceLocation) -> u32 #foreign libclang`
- `clang_isBeforeInTranslationUnit :: (loc1: CXSourceLocation, loc2: CXSourceLocation) -> u32 #foreign libclang`
- `clang_Location_isInSystemHeader :: (location: CXSourceLocation) -> s32 #foreign libclang`
- `clang_Location_isFromMainFile :: (location: CXSourceLocation) -> s32 #foreign libclang`
- `clang_getNullRange :: () -> CXSourceRange #foreign libclang`
- `clang_getRange :: (begin: CXSourceLocation, end: CXSourceLocation) -> CXSourceRange #foreign libclang`
- `clang_equalRanges :: (range1: CXSourceRange, range2: CXSourceRange) -> u32 #foreign libclang`
- `clang_Range_isNull :: (range: CXSourceRange) -> s32 #foreign libclang`
- `clang_getExpansionLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getPresumedLocation :: (location: CXSourceLocation, filename: *CXString, line: *u32, column: *u32) -> void #foreign libclang`
- `clang_getInstantiationLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getSpellingLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getFileLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_getRangeStart :: (range: CXSourceRange) -> CXSourceLocation #foreign libclang`
- `clang_getRangeEnd :: (range: CXSourceRange) -> CXSourceLocation #foreign libclang`
- `CXSourceRangeList :: struct`
- `clang_disposeSourceRangeList :: (ranges: *CXSourceRangeList) -> void #foreign libclang`
- `CXDiagnosticSeverity :: enum s32`
- `CXDiagnostic :: *void`
- `CXDiagnosticSet :: *void`
- `clang_getNumDiagnosticsInSet :: (Diags: CXDiagnosticSet) -> u32 #foreign libclang`
- `clang_getDiagnosticInSet :: (Diags: CXDiagnosticSet, Index: u32) -> CXDiagnostic #foreign libclang`
- `CXLoadDiag_Error :: enum s32`
- `clang_loadDiagnostics :: (file: *u8, error: *CXLoadDiag_Error, errorString: *CXString) -> CXDiagnosticSet #foreign libclang`
- `clang_disposeDiagnosticSet :: (Diags: CXDiagnosticSet) -> void #foreign libclang`
- `clang_getChildDiagnostics :: (D: CXDiagnostic) -> CXDiagnosticSet #foreign libclang`
- `clang_disposeDiagnostic :: (Diagnostic: CXDiagnostic) -> void #foreign libclang`
- `CXDiagnosticDisplayOptions :: enum s32`
- `clang_formatDiagnostic :: (Diagnostic: CXDiagnostic, Options: u32) -> CXString #foreign libclang`
- `clang_defaultDiagnosticDisplayOptions :: () -> u32 #foreign libclang`
- `clang_getDiagnosticSeverity :: (unknown0: CXDiagnostic) -> CXDiagnosticSeverity #foreign libclang`
- `clang_getDiagnosticLocation :: (unknown0: CXDiagnostic) -> CXSourceLocation #foreign libclang`
- `clang_getDiagnosticSpelling :: (unknown0: CXDiagnostic) -> CXString #foreign libclang`
- `clang_getDiagnosticOption :: (Diag: CXDiagnostic, Disable: *CXString) -> CXString #foreign libclang`
- `clang_getDiagnosticCategory :: (unknown0: CXDiagnostic) -> u32 #foreign libclang`
- `clang_getDiagnosticCategoryName :: (Category: u32) -> CXString #foreign libclang`
- `clang_getDiagnosticCategoryText :: (unknown0: CXDiagnostic) -> CXString #foreign libclang`
- `clang_getDiagnosticNumRanges :: (unknown0: CXDiagnostic) -> u32 #foreign libclang`
- `clang_getDiagnosticRange :: (Diagnostic: CXDiagnostic, Range: u32) -> CXSourceRange #foreign libclang`
- `clang_getDiagnosticNumFixIts :: (Diagnostic: CXDiagnostic) -> u32 #foreign libclang`
- `clang_getDiagnosticFixIt :: (Diagnostic: CXDiagnostic, FixIt: u32, ReplacementRange: *CXSourceRange) -> CXString #foreign libclang`
- `CXIndex :: *void`
- `CXTargetInfoImpl :: struct`
- `CXTargetInfo :: *CXTargetInfoImpl`
- `CXTranslationUnitImpl :: struct`
- `CXTranslationUnit :: *CXTranslationUnitImpl`
- `CXClientData :: *void`
- `CXUnsavedFile :: struct`
- `CXAvailabilityKind :: enum s32`
- `CXVersion :: struct`
- `CXCursor_ExceptionSpecificationKind :: enum s32`
- `clang_createIndex :: (excludeDeclarationsFromPCH: s32, displayDiagnostics: s32) -> CXIndex #foreign libclang`
- `clang_disposeIndex :: (index: CXIndex) -> void #foreign libclang`
- `CXChoice :: enum s32`
- `CXGlobalOptFlags :: enum s32`
- `CXIndexOptions :: struct`
- `set_ExcludeDeclarationsFromPCH :: (fields: *CXIndexOptions, value: bool)`
- `get_ExcludeDeclarationsFromPCH :: (fields: CXIndexOptions) -> bool`
- `set_DisplayDiagnostics :: (fields: *CXIndexOptions, value: bool)`
- `get_DisplayDiagnostics :: (fields: CXIndexOptions) -> bool`
- `set_StorePreamblesInMemory :: (fields: *CXIndexOptions, value: bool)`
- `get_StorePreamblesInMemory :: (fields: CXIndexOptions) -> bool`
- `clang_createIndexWithOptions :: (options: *CXIndexOptions) -> CXIndex #foreign libclang`
- `clang_CXIndex_setGlobalOptions :: (unknown0: CXIndex, options: u32) -> void #foreign libclang`
- `clang_CXIndex_getGlobalOptions :: (unknown0: CXIndex) -> u32 #foreign libclang`
- `clang_CXIndex_setInvocationEmissionPathOption :: (unknown0: CXIndex, Path: *u8) -> void #foreign libclang`
- `clang_isFileMultipleIncludeGuarded :: (tu: CXTranslationUnit, file: CXFile) -> u32 #foreign libclang`
- `clang_getFile :: (tu: CXTranslationUnit, file_name: *u8) -> CXFile #foreign libclang`
- `clang_getFileContents :: (tu: CXTranslationUnit, file: CXFile, size: *u64) -> *u8 #foreign libclang`
- `clang_getLocation :: (tu: CXTranslationUnit, file: CXFile, line: u32, column: u32) -> CXSourceLocation #foreign libclang`
- `clang_getLocationForOffset :: (tu: CXTranslationUnit, file: CXFile, offset: u32) -> CXSourceLocation #foreign libclang`
- `clang_getSkippedRanges :: (tu: CXTranslationUnit, file: CXFile) -> *CXSourceRangeList #foreign libclang`
- `clang_getAllSkippedRanges :: (tu: CXTranslationUnit) -> *CXSourceRangeList #foreign libclang`
- `clang_getNumDiagnostics :: (Unit: CXTranslationUnit) -> u32 #foreign libclang`
- `clang_getDiagnostic :: (Unit: CXTranslationUnit, Index: u32) -> CXDiagnostic #foreign libclang`
- `clang_getDiagnosticSetFromTU :: (Unit: CXTranslationUnit) -> CXDiagnosticSet #foreign libclang`
- `clang_getTranslationUnitSpelling :: (CTUnit: CXTranslationUnit) -> CXString #foreign libclang`
- `clang_createTranslationUnitFromSourceFile :: (CIdx: CXIndex, source_filename: *u8, num_clang_command_line_args: s32, clang_command_line_args: **u8, num_unsaved_files: u32, unsaved_files: *CXUnsavedFile) -> CXTranslationUnit #foreign libclang`
- `clang_createTranslationUnit :: (CIdx: CXIndex, ast_filename: *u8) -> CXTranslationUnit #foreign libclang`
- `clang_createTranslationUnit2 :: (CIdx: CXIndex, ast_filename: *u8, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang`
- `CXTranslationUnit_Flags :: enum_flags s32`
- `clang_defaultEditingTranslationUnitOptions :: () -> u32 #foreign libclang`
- `clang_parseTranslationUnit :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32) -> CXTranslationUnit #foreign libclang`
- `clang_parseTranslationUnit2 :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang`
- `clang_parseTranslationUnit2FullArgv :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang`
- `CXSaveTranslationUnit_Flags :: enum_flags s32`
- `clang_defaultSaveOptions :: (TU: CXTranslationUnit) -> u32 #foreign libclang`
- `CXSaveError :: enum s32`
- `clang_saveTranslationUnit :: (TU: CXTranslationUnit, FileName: *u8, options: u32) -> s32 #foreign libclang`
- `clang_suspendTranslationUnit :: (unknown0: CXTranslationUnit) -> u32 #foreign libclang`
- `clang_disposeTranslationUnit :: (unknown0: CXTranslationUnit) -> void #foreign libclang`
- `CXReparse_Flags :: enum_flags s32`
- `clang_defaultReparseOptions :: (TU: CXTranslationUnit) -> u32 #foreign libclang`
- `clang_reparseTranslationUnit :: (TU: CXTranslationUnit, num_unsaved_files: u32, unsaved_files: *CXUnsavedFile, options: u32) -> s32 #foreign libclang`
- `CXTUResourceUsageKind :: enum s32`
- `clang_getTUResourceUsageName :: (kind: CXTUResourceUsageKind) -> *u8 #foreign libclang`
- `CXTUResourceUsageEntry :: struct`
- `CXTUResourceUsage :: struct`
- `clang_getCXTUResourceUsage :: (TU: CXTranslationUnit) -> CXTUResourceUsage #foreign libclang`
- `clang_disposeCXTUResourceUsage :: (usage: CXTUResourceUsage) -> void #foreign libclang`
- `clang_getTranslationUnitTargetInfo :: (CTUnit: CXTranslationUnit) -> CXTargetInfo #foreign libclang`
- `clang_TargetInfo_dispose :: (Info: CXTargetInfo) -> void #foreign libclang`
- `clang_TargetInfo_getTriple :: (Info: CXTargetInfo) -> CXString #foreign libclang`
- `clang_TargetInfo_getPointerWidth :: (Info: CXTargetInfo) -> s32 #foreign libclang`
- `CXCursorKind :: enum s32`
- `CXCursor :: struct`
- `clang_getNullCursor :: () -> CXCursor #foreign libclang`
- `clang_getTranslationUnitCursor :: (unknown0: CXTranslationUnit) -> CXCursor #foreign libclang`
- `clang_equalCursors :: (unknown0: CXCursor, unknown1: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isNull :: (cursor: CXCursor) -> s32 #foreign libclang`
- `clang_hashCursor :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_getCursorKind :: (unknown0: CXCursor) -> CXCursorKind #foreign libclang`
- `clang_isDeclaration :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isInvalidDeclaration :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_isReference :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isExpression :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isStatement :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isAttribute :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_Cursor_hasAttrs :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_isInvalid :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isTranslationUnit :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isPreprocessing :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `clang_isUnexposed :: (unknown0: CXCursorKind) -> u32 #foreign libclang`
- `CXLinkageKind :: enum s32`
- `clang_getCursorLinkage :: (cursor: CXCursor) -> CXLinkageKind #foreign libclang`
- `CXVisibilityKind :: enum s32`
- `clang_getCursorVisibility :: (cursor: CXCursor) -> CXVisibilityKind #foreign libclang`
- `clang_getCursorAvailability :: (cursor: CXCursor) -> CXAvailabilityKind #foreign libclang`
- `CXPlatformAvailability :: struct`
- `clang_getCursorPlatformAvailability :: (cursor: CXCursor, always_deprecated: *s32, deprecated_message: *CXString, always_unavailable: *s32, unavailable_message: *CXString, availability: *CXPlatformAvailability, availability_size: s32) -> s32 #foreign libclang`
- `clang_disposeCXPlatformAvailability :: (availability: *CXPlatformAvailability) -> void #foreign libclang`
- `CXLanguageKind :: enum s32`
- `clang_getCursorLanguage :: (cursor: CXCursor) -> CXLanguageKind #foreign libclang`
- `CXTLSKind :: enum s32`
- `clang_getCursorTLSKind :: (cursor: CXCursor) -> CXTLSKind #foreign libclang`
- `clang_Cursor_getTranslationUnit :: (unknown0: CXCursor) -> CXTranslationUnit #foreign libclang`
- `CXCursorSetImpl :: struct`
- `CXCursorSet :: *CXCursorSetImpl`
- `clang_createCXCursorSet :: () -> CXCursorSet #foreign libclang`
- `clang_disposeCXCursorSet :: (cset: CXCursorSet) -> void #foreign libclang`
- `clang_CXCursorSet_contains :: (cset: CXCursorSet, cursor: CXCursor) -> u32 #foreign libclang`
- `clang_CXCursorSet_insert :: (cset: CXCursorSet, cursor: CXCursor) -> u32 #foreign libclang`
- `clang_getCursorSemanticParent :: (cursor: CXCursor) -> CXCursor #foreign libclang`
- `clang_getCursorLexicalParent :: (cursor: CXCursor) -> CXCursor #foreign libclang`
- `clang_getOverriddenCursors :: (cursor: CXCursor, overridden: **CXCursor, num_overridden: *u32) -> void #foreign libclang`
- `clang_disposeOverriddenCursors :: (overridden: *CXCursor) -> void #foreign libclang`
- `clang_getIncludedFile :: (cursor: CXCursor) -> CXFile #foreign libclang`
- `clang_getCursor :: (unknown0: CXTranslationUnit, unknown1: CXSourceLocation) -> CXCursor #foreign libclang`
- `clang_getCursorLocation :: (unknown0: CXCursor) -> CXSourceLocation #foreign libclang`
- `clang_getCursorExtent :: (unknown0: CXCursor) -> CXSourceRange #foreign libclang`
- `CXTypeKind :: enum s32`
- `CXCallingConv :: enum s32`
- `CXType :: struct`
- `clang_getCursorType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getTypeSpelling :: (CT: CXType) -> CXString #foreign libclang`
- `clang_getTypedefDeclUnderlyingType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getEnumDeclIntegerType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getEnumConstantDeclValue :: (C: CXCursor) -> s64 #foreign libclang`
- `clang_getEnumConstantDeclUnsignedValue :: (C: CXCursor) -> u64 #foreign libclang`
- `clang_Cursor_isBitField :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_getFieldDeclBitWidth :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getNumArguments :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getArgument :: (C: CXCursor, i: u32) -> CXCursor #foreign libclang`
- `CXTemplateArgumentKind :: enum s32`
- `clang_Cursor_getNumTemplateArguments :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getTemplateArgumentKind :: (C: CXCursor, I: u32) -> CXTemplateArgumentKind #foreign libclang`
- `clang_Cursor_getTemplateArgumentType :: (C: CXCursor, I: u32) -> CXType #foreign libclang`
- `clang_Cursor_getTemplateArgumentValue :: (C: CXCursor, I: u32) -> s64 #foreign libclang`
- `clang_Cursor_getTemplateArgumentUnsignedValue :: (C: CXCursor, I: u32) -> u64 #foreign libclang`
- `clang_equalTypes :: (A: CXType, B: CXType) -> u32 #foreign libclang`
- `clang_getCanonicalType :: (T: CXType) -> CXType #foreign libclang`
- `clang_isConstQualifiedType :: (T: CXType) -> u32 #foreign libclang`
- `clang_Cursor_isMacroFunctionLike :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isMacroBuiltin :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isFunctionInlined :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_isVolatileQualifiedType :: (T: CXType) -> u32 #foreign libclang`
- `clang_isRestrictQualifiedType :: (T: CXType) -> u32 #foreign libclang`
- `clang_getAddressSpace :: (T: CXType) -> u32 #foreign libclang`
- `clang_getTypedefName :: (CT: CXType) -> CXString #foreign libclang`
- `clang_getPointeeType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getUnqualifiedType :: (CT: CXType) -> CXType #foreign libclang`
- `clang_getNonReferenceType :: (CT: CXType) -> CXType #foreign libclang`
- `clang_getTypeDeclaration :: (T: CXType) -> CXCursor #foreign libclang`
- `clang_getDeclObjCTypeEncoding :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Type_getObjCEncoding :: (type: CXType) -> CXString #foreign libclang`
- `clang_getTypeKindSpelling :: (K: CXTypeKind) -> CXString #foreign libclang`
- `clang_getFunctionTypeCallingConv :: (T: CXType) -> CXCallingConv #foreign libclang`
- `clang_getResultType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getExceptionSpecificationType :: (T: CXType) -> s32 #foreign libclang`
- `clang_getNumArgTypes :: (T: CXType) -> s32 #foreign libclang`
- `clang_getArgType :: (T: CXType, i: u32) -> CXType #foreign libclang`
- `clang_Type_getObjCObjectBaseType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_getNumObjCProtocolRefs :: (T: CXType) -> u32 #foreign libclang`
- `clang_Type_getObjCProtocolDecl :: (T: CXType, i: u32) -> CXCursor #foreign libclang`
- `clang_Type_getNumObjCTypeArgs :: (T: CXType) -> u32 #foreign libclang`
- `clang_Type_getObjCTypeArg :: (T: CXType, i: u32) -> CXType #foreign libclang`
- `clang_isFunctionTypeVariadic :: (T: CXType) -> u32 #foreign libclang`
- `clang_getCursorResultType :: (C: CXCursor) -> CXType #foreign libclang`
- `clang_getCursorExceptionSpecificationType :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_isPODType :: (T: CXType) -> u32 #foreign libclang`
- `clang_getElementType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getNumElements :: (T: CXType) -> s64 #foreign libclang`
- `clang_getArrayElementType :: (T: CXType) -> CXType #foreign libclang`
- `clang_getArraySize :: (T: CXType) -> s64 #foreign libclang`
- `clang_Type_getNamedType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_isTransparentTagTypedef :: (T: CXType) -> u32 #foreign libclang`
- `CXTypeNullabilityKind :: enum s32`
- `clang_Type_getNullability :: (T: CXType) -> CXTypeNullabilityKind #foreign libclang`
- `CXTypeLayoutError :: enum s32`
- `clang_Type_getAlignOf :: (T: CXType) -> s64 #foreign libclang`
- `clang_Type_getClassType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_getSizeOf :: (T: CXType) -> s64 #foreign libclang`
- `clang_Type_getOffsetOf :: (T: CXType, S: *u8) -> s64 #foreign libclang`
- `clang_Type_getModifiedType :: (T: CXType) -> CXType #foreign libclang`
- `clang_Type_getValueType :: (CT: CXType) -> CXType #foreign libclang`
- `clang_Cursor_getOffsetOfField :: (C: CXCursor) -> s64 #foreign libclang`
- `clang_Cursor_isAnonymous :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isAnonymousRecordDecl :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isInlineNamespace :: (C: CXCursor) -> u32 #foreign libclang`
- `CXRefQualifierKind :: enum s32`
- `clang_Type_getNumTemplateArguments :: (T: CXType) -> s32 #foreign libclang`
- `clang_Type_getTemplateArgumentAsType :: (T: CXType, i: u32) -> CXType #foreign libclang`
- `clang_Type_getCXXRefQualifier :: (T: CXType) -> CXRefQualifierKind #foreign libclang`
- `clang_isVirtualBase :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_getOffsetOfBase :: (Parent: CXCursor, Base: CXCursor) -> s64 #foreign libclang`
- `CX_CXXAccessSpecifier :: enum s32`
- `clang_getCXXAccessSpecifier :: (unknown0: CXCursor) -> CX_CXXAccessSpecifier #foreign libclang`
- `CX_StorageClass :: enum s32`
- `CX_BinaryOperatorKind :: enum s32`
- `clang_Cursor_getBinaryOpcode :: (C: CXCursor) -> CX_BinaryOperatorKind #foreign libclang`
- `clang_Cursor_getBinaryOpcodeStr :: (Op: CX_BinaryOperatorKind) -> CXString #foreign libclang`
- `clang_Cursor_getStorageClass :: (unknown0: CXCursor) -> CX_StorageClass #foreign libclang`
- `clang_getNumOverloadedDecls :: (cursor: CXCursor) -> u32 #foreign libclang`
- `clang_getOverloadedDecl :: (cursor: CXCursor, index: u32) -> CXCursor #foreign libclang`
- `clang_getIBOutletCollectionType :: (unknown0: CXCursor) -> CXType #foreign libclang`
- `CXChildVisitResult :: enum s32`
- `CXCursorVisitor :: #type (cursor: CXCursor, parent: CXCursor, client_data: CXClientData) -> CXChildVisitResult #c_call`
- `clang_visitChildren :: (parent: CXCursor, visitor: CXCursorVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `clang_getCursorUSR :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCClass :: (class_name: *u8) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCCategory :: (class_name: *u8, category_name: *u8) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCProtocol :: (protocol_name: *u8) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCIvar :: (name: *u8, classUSR: CXString) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCMethod :: (name: *u8, isInstanceMethod: u32, classUSR: CXString) -> CXString #foreign libclang`
- `clang_constructUSR_ObjCProperty :: (property: *u8, classUSR: CXString) -> CXString #foreign libclang`
- `clang_getCursorSpelling :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getSpellingNameRange :: (unknown0: CXCursor, pieceIndex: u32, options: u32) -> CXSourceRange #foreign libclang`
- `CXPrintingPolicy :: *void`
- `CXPrintingPolicyProperty :: enum s32`
- `clang_PrintingPolicy_getProperty :: (Policy: CXPrintingPolicy, Property: CXPrintingPolicyProperty) -> u32 #foreign libclang`
- `clang_PrintingPolicy_setProperty :: (Policy: CXPrintingPolicy, Property: CXPrintingPolicyProperty, Value: u32) -> void #foreign libclang`
- `clang_getCursorPrintingPolicy :: (unknown0: CXCursor) -> CXPrintingPolicy #foreign libclang`
- `clang_PrintingPolicy_dispose :: (Policy: CXPrintingPolicy) -> void #foreign libclang`
- `clang_getCursorPrettyPrinted :: (Cursor: CXCursor, Policy: CXPrintingPolicy) -> CXString #foreign libclang`
- `clang_getTypePrettyPrinted :: (CT: CXType, cxPolicy: CXPrintingPolicy) -> CXString #foreign libclang`
- `clang_getFullyQualifiedName :: (CT: CXType, Policy: CXPrintingPolicy, WithGlobalNsPrefix: u32) -> CXString #foreign libclang`
- `clang_getCursorDisplayName :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_getCursorReferenced :: (unknown0: CXCursor) -> CXCursor #foreign libclang`
- `clang_getCursorDefinition :: (unknown0: CXCursor) -> CXCursor #foreign libclang`
- `clang_isCursorDefinition :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_getCanonicalCursor :: (unknown0: CXCursor) -> CXCursor #foreign libclang`
- `clang_Cursor_getObjCSelectorIndex :: (unknown0: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_isDynamicCall :: (C: CXCursor) -> s32 #foreign libclang`
- `clang_Cursor_getReceiverType :: (C: CXCursor) -> CXType #foreign libclang`
- `CXObjCPropertyAttrKind :: enum s32`
- `clang_Cursor_getObjCPropertyAttributes :: (C: CXCursor, reserved: u32) -> u32 #foreign libclang`
- `clang_Cursor_getObjCPropertyGetterName :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getObjCPropertySetterName :: (C: CXCursor) -> CXString #foreign libclang`
- `CXObjCDeclQualifierKind :: enum s32`
- `clang_Cursor_getObjCDeclQualifiers :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isObjCOptional :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isVariadic :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_isExternalSymbol :: (C: CXCursor, language: *CXString, definedIn: *CXString, isGenerated: *u32) -> u32 #foreign libclang`
- `clang_Cursor_getCommentRange :: (C: CXCursor) -> CXSourceRange #foreign libclang`
- `clang_Cursor_getRawCommentText :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getBriefCommentText :: (C: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getMangling :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_getCXXManglings :: (unknown0: CXCursor) -> *CXStringSet #foreign libclang`
- `clang_Cursor_getObjCManglings :: (unknown0: CXCursor) -> *CXStringSet #foreign libclang`
- `clang_Cursor_getGCCAssemblyTemplate :: (unknown0: CXCursor) -> CXString #foreign libclang`
- `clang_Cursor_isGCCAssemblyHasGoto :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyNumOutputs :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyNumInputs :: (unknown0: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyInput :: (Cursor: CXCursor, Index: u32, Constraint: *CXString, Expr: *CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyOutput :: (Cursor: CXCursor, Index: u32, Constraint: *CXString, Expr: *CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyNumClobbers :: (Cursor: CXCursor) -> u32 #foreign libclang`
- `clang_Cursor_getGCCAssemblyClobber :: (Cursor: CXCursor, Index: u32) -> CXString #foreign libclang`
- `clang_Cursor_isGCCAssemblyVolatile :: (Cursor: CXCursor) -> u32 #foreign libclang`
- `CXModule :: *void`
- `clang_Cursor_getModule :: (C: CXCursor) -> CXModule #foreign libclang`
- `clang_getModuleForFile :: (unknown0: CXTranslationUnit, unknown1: CXFile) -> CXModule #foreign libclang`
- `clang_Module_getASTFile :: (Module: CXModule) -> CXFile #foreign libclang`
- `clang_Module_getParent :: (Module: CXModule) -> CXModule #foreign libclang`
- `clang_Module_getName :: (Module: CXModule) -> CXString #foreign libclang`
- `clang_Module_getFullName :: (Module: CXModule) -> CXString #foreign libclang`
- `clang_Module_isSystem :: (Module: CXModule) -> s32 #foreign libclang`
- `clang_Module_getNumTopLevelHeaders :: (unknown0: CXTranslationUnit, Module: CXModule) -> u32 #foreign libclang`
- `clang_Module_getTopLevelHeader :: (unknown0: CXTranslationUnit, Module: CXModule, Index: u32) -> CXFile #foreign libclang`
- `clang_CXXConstructor_isConvertingConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXConstructor_isCopyConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXConstructor_isDefaultConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXConstructor_isMoveConstructor :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXField_isMutable :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isDefaulted :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isDeleted :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isPureVirtual :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isStatic :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isVirtual :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isCopyAssignmentOperator :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isMoveAssignmentOperator :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isExplicit :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXRecord_isAbstract :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_EnumDecl_isScoped :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_CXXMethod_isConst :: (C: CXCursor) -> u32 #foreign libclang`
- `clang_getTemplateCursorKind :: (C: CXCursor) -> CXCursorKind #foreign libclang`
- `clang_getSpecializedCursorTemplate :: (C: CXCursor) -> CXCursor #foreign libclang`
- `clang_getCursorReferenceNameRange :: (C: CXCursor, NameFlags: u32, PieceIndex: u32) -> CXSourceRange #foreign libclang`
- `CXNameRefFlags :: enum s32`
- `CXTokenKind :: enum s32`
- `CXToken :: struct`
- `clang_getToken :: (TU: CXTranslationUnit, Location: CXSourceLocation) -> *CXToken #foreign libclang`
- `clang_getTokenKind :: (unknown0: CXToken) -> CXTokenKind #foreign libclang`
- `clang_getTokenSpelling :: (unknown0: CXTranslationUnit, unknown1: CXToken) -> CXString #foreign libclang`
- `clang_getTokenLocation :: (unknown0: CXTranslationUnit, unknown1: CXToken) -> CXSourceLocation #foreign libclang`
- `clang_getTokenExtent :: (unknown0: CXTranslationUnit, unknown1: CXToken) -> CXSourceRange #foreign libclang`
- `clang_tokenize :: (TU: CXTranslationUnit, Range: CXSourceRange, Tokens: **CXToken, NumTokens: *u32) -> void #foreign libclang`
- `clang_annotateTokens :: (TU: CXTranslationUnit, Tokens: *CXToken, NumTokens: u32, Cursors: *CXCursor) -> void #foreign libclang`
- `clang_disposeTokens :: (TU: CXTranslationUnit, Tokens: *CXToken, NumTokens: u32) -> void #foreign libclang`
- `clang_getCursorKindSpelling :: (Kind: CXCursorKind) -> CXString #foreign libclang`
- `clang_getDefinitionSpellingAndExtent :: (unknown0: CXCursor, startBuf: **u8, endBuf: **u8, startLine: *u32, startColumn: *u32, endLine: *u32, endColumn: *u32) -> void #foreign libclang`
- `clang_enableStackTraces :: () -> void #foreign libclang`
- `clang_executeOnThread :: (fn: #type (unknown0: *void) -> void #c_call, user_data: *void, stack_size: u32) -> void #foreign libclang`
- `CXCompletionString :: *void`
- `CXCompletionResult :: struct`
- `CXCompletionChunkKind :: enum s32`
- `clang_getCompletionChunkKind :: (completion_string: CXCompletionString, chunk_number: u32) -> CXCompletionChunkKind #foreign libclang`
- `clang_getCompletionChunkText :: (completion_string: CXCompletionString, chunk_number: u32) -> CXString #foreign libclang`
- `clang_getCompletionChunkCompletionString :: (completion_string: CXCompletionString, chunk_number: u32) -> CXCompletionString #foreign libclang`
- `clang_getNumCompletionChunks :: (completion_string: CXCompletionString) -> u32 #foreign libclang`
- `clang_getCompletionPriority :: (completion_string: CXCompletionString) -> u32 #foreign libclang`
- `clang_getCompletionAvailability :: (completion_string: CXCompletionString) -> CXAvailabilityKind #foreign libclang`
- `clang_getCompletionNumAnnotations :: (completion_string: CXCompletionString) -> u32 #foreign libclang`
- `clang_getCompletionAnnotation :: (completion_string: CXCompletionString, annotation_number: u32) -> CXString #foreign libclang`
- `clang_getCompletionParent :: (completion_string: CXCompletionString, kind: *CXCursorKind) -> CXString #foreign libclang`
- `clang_getCompletionBriefComment :: (completion_string: CXCompletionString) -> CXString #foreign libclang`
- `clang_getCursorCompletionString :: (cursor: CXCursor) -> CXCompletionString #foreign libclang`
- `CXCodeCompleteResults :: struct`
- `clang_getCompletionNumFixIts :: (results: *CXCodeCompleteResults, completion_index: u32) -> u32 #foreign libclang`
- `clang_getCompletionFixIt :: (results: *CXCodeCompleteResults, completion_index: u32, fixit_index: u32, replacement_range: *CXSourceRange) -> CXString #foreign libclang`
- `CXCodeComplete_Flags :: enum_flags s32`
- `CXCompletionContext :: enum_flags s32`
- `clang_defaultCodeCompleteOptions :: () -> u32 #foreign libclang`
- `clang_codeCompleteAt :: (TU: CXTranslationUnit, complete_filename: *u8, complete_line: u32, complete_column: u32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32) -> *CXCodeCompleteResults #foreign libclang`
- `clang_sortCodeCompletionResults :: (Results: *CXCompletionResult, NumResults: u32) -> void #foreign libclang`
- `clang_disposeCodeCompleteResults :: (Results: *CXCodeCompleteResults) -> void #foreign libclang`
- `clang_codeCompleteGetNumDiagnostics :: (Results: *CXCodeCompleteResults) -> u32 #foreign libclang`
- `clang_codeCompleteGetDiagnostic :: (Results: *CXCodeCompleteResults, Index: u32) -> CXDiagnostic #foreign libclang`
- `clang_codeCompleteGetContexts :: (Results: *CXCodeCompleteResults) -> u64 #foreign libclang`
- `clang_codeCompleteGetContainerKind :: (Results: *CXCodeCompleteResults, IsIncomplete: *u32) -> CXCursorKind #foreign libclang`
- `clang_codeCompleteGetContainerUSR :: (Results: *CXCodeCompleteResults) -> CXString #foreign libclang`
- `clang_codeCompleteGetObjCSelector :: (Results: *CXCodeCompleteResults) -> CXString #foreign libclang`
- `clang_getClangVersion :: () -> CXString #foreign libclang`
- `clang_toggleCrashRecovery :: (isEnabled: u32) -> void #foreign libclang`
- `CXInclusionVisitor :: #type (included_file: CXFile, inclusion_stack: *CXSourceLocation, include_len: u32, client_data: CXClientData) -> void #c_call`
- `clang_getInclusions :: (tu: CXTranslationUnit, visitor: CXInclusionVisitor, client_data: CXClientData) -> void #foreign libclang`
- `CXEvalResultKind :: enum s32`
- `CXEvalResult :: *void`
- `clang_Cursor_Evaluate :: (C: CXCursor) -> CXEvalResult #foreign libclang`
- `clang_EvalResult_getKind :: (E: CXEvalResult) -> CXEvalResultKind #foreign libclang`
- `clang_EvalResult_getAsInt :: (E: CXEvalResult) -> s32 #foreign libclang`
- `clang_EvalResult_getAsLongLong :: (E: CXEvalResult) -> s64 #foreign libclang`
- `clang_EvalResult_isUnsignedInt :: (E: CXEvalResult) -> u32 #foreign libclang`
- `clang_EvalResult_getAsUnsigned :: (E: CXEvalResult) -> u64 #foreign libclang`
- `clang_EvalResult_getAsDouble :: (E: CXEvalResult) -> float64 #foreign libclang`
- `clang_EvalResult_getAsStr :: (E: CXEvalResult) -> *u8 #foreign libclang`
- `clang_EvalResult_dispose :: (E: CXEvalResult) -> void #foreign libclang`
- `CXVisitorResult :: enum s32`
- `CXCursorAndRangeVisitor :: struct`
- `CXResult :: enum s32`
- `clang_findReferencesInFile :: (cursor: CXCursor, file: CXFile, visitor: CXCursorAndRangeVisitor) -> CXResult #foreign libclang`
- `clang_findIncludesInFile :: (TU: CXTranslationUnit, file: CXFile, visitor: CXCursorAndRangeVisitor) -> CXResult #foreign libclang`
- `CXIdxClientFile :: *void`
- `CXIdxClientEntity :: *void`
- `CXIdxClientContainer :: *void`
- `CXIdxClientASTFile :: *void`
- `CXIdxLoc :: struct`
- `CXIdxIncludedFileInfo :: struct`
- `CXIdxImportedASTFileInfo :: struct`
- `CXIdxEntityKind :: enum s32`
- `CXIdxEntityLanguage :: enum s32`
- `CXIdxEntityCXXTemplateKind :: enum s32`
- `CXIdxAttrKind :: enum s32`
- `CXIdxAttrInfo :: struct`
- `CXIdxEntityInfo :: struct`
- `CXIdxContainerInfo :: struct`
- `CXIdxIBOutletCollectionAttrInfo :: struct`
- `CXIdxDeclInfoFlags :: enum s32`
- `CXIdxDeclInfo :: struct`
- `CXIdxObjCContainerKind :: enum s32`
- `CXIdxObjCContainerDeclInfo :: struct`
- `CXIdxBaseClassInfo :: struct`
- `CXIdxObjCProtocolRefInfo :: struct`
- `CXIdxObjCProtocolRefListInfo :: struct`
- `CXIdxObjCInterfaceDeclInfo :: struct`
- `CXIdxObjCCategoryDeclInfo :: struct`
- `CXIdxObjCPropertyDeclInfo :: struct`
- `CXIdxCXXClassDeclInfo :: struct`
- `CXIdxEntityRefKind :: enum s32`
- `CXSymbolRole :: enum s32`
- `CXIdxEntityRefInfo :: struct`
- `IndexerCallbacks :: struct`
- `clang_index_isEntityObjCContainerKind :: (unknown0: CXIdxEntityKind) -> s32 #foreign libclang`
- `clang_index_getObjCContainerDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCContainerDeclInfo #foreign libclang`
- `clang_index_getObjCInterfaceDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCInterfaceDeclInfo #foreign libclang`
- `clang_index_getObjCCategoryDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCCategoryDeclInfo #foreign libclang`
- `clang_index_getObjCProtocolRefListInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCProtocolRefListInfo #foreign libclang`
- `clang_index_getObjCPropertyDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCPropertyDeclInfo #foreign libclang`
- `clang_index_getIBOutletCollectionAttrInfo :: (unknown0: *CXIdxAttrInfo) -> *CXIdxIBOutletCollectionAttrInfo #foreign libclang`
- `clang_index_getCXXClassDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxCXXClassDeclInfo #foreign libclang`
- `clang_index_getClientContainer :: (unknown0: *CXIdxContainerInfo) -> CXIdxClientContainer #foreign libclang`
- `clang_index_setClientContainer :: (unknown0: *CXIdxContainerInfo, unknown1: CXIdxClientContainer) -> void #foreign libclang`
- `clang_index_getClientEntity :: (unknown0: *CXIdxEntityInfo) -> CXIdxClientEntity #foreign libclang`
- `clang_index_setClientEntity :: (unknown0: *CXIdxEntityInfo, unknown1: CXIdxClientEntity) -> void #foreign libclang`
- `CXIndexAction :: *void`
- `clang_IndexAction_create :: (CIdx: CXIndex) -> CXIndexAction #foreign libclang`
- `clang_IndexAction_dispose :: (unknown0: CXIndexAction) -> void #foreign libclang`
- `CXIndexOptFlags :: enum s32`
- `clang_indexSourceFile :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, out_TU: *CXTranslationUnit, TU_options: u32) -> s32 #foreign libclang`
- `clang_indexSourceFileFullArgv :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, out_TU: *CXTranslationUnit, TU_options: u32) -> s32 #foreign libclang`
- `clang_indexTranslationUnit :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, unknown1: CXTranslationUnit) -> s32 #foreign libclang`
- `clang_indexLoc_getFileLocation :: (loc: CXIdxLoc, indexFile: *CXIdxClientFile, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang`
- `clang_indexLoc_getCXSourceLocation :: (loc: CXIdxLoc) -> CXSourceLocation #foreign libclang`
- `CXFieldVisitor :: #type (C: CXCursor, client_data: CXClientData) -> CXVisitorResult #c_call`
- `clang_Type_visitFields :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `clang_visitCXXBaseClasses :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `clang_visitCXXMethods :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang`
- `CXBinaryOperatorKind :: enum s32`
- `clang_getBinaryOperatorKindSpelling :: (kind: CXBinaryOperatorKind) -> CXString #foreign libclang`
- `clang_getCursorBinaryOperatorKind :: (cursor: CXCursor) -> CXBinaryOperatorKind #foreign libclang`
- `CXUnaryOperatorKind :: enum s32`
- `clang_getUnaryOperatorKindSpelling :: (kind: CXUnaryOperatorKind) -> CXString #foreign libclang`
- `clang_getCursorUnaryOperatorKind :: (cursor: CXCursor) -> CXUnaryOperatorKind #foreign libclang`
- `CXRemapping :: *void`
- `clang_getRemappings :: (unknown0: *u8) -> CXRemapping #foreign libclang`
- `clang_getRemappingsFromFileList :: (unknown0: **u8, unknown1: u32) -> CXRemapping #foreign libclang`
- `clang_remap_getNumFiles :: (unknown0: CXRemapping) -> u32 #foreign libclang`
- `clang_remap_getFilenames :: (unknown0: CXRemapping, unknown1: u32, unknown2: *CXString, unknown3: *CXString) -> void #foreign libclang`
- `clang_remap_dispose :: (unknown0: CXRemapping) -> void #foreign libclang`

- `Library_Symbols :: struct`
- [OS == .WINDOWS] `PATH_SEPARATOR_STRING :: "\\"`
- [OS == .LINUX || OS == .MACOS] `PATH_SEPARATOR_STRING :: "/"`
- [OS == .WINDOWS] `SHARED_LIBRARY_SUFFIX :: ".dll"`
- [OS == .LINUX] `SHARED_LIBRARY_SUFFIX :: ".so"`
- [OS == .MACOS] `SHARED_LIBRARY_SUFFIX :: ".dylib"`
- [OS == .WINDOWS] `STATIC_LIBRARY_SUFFIX :: ".lib"`
- [OS == .LINUX || OS == .MACOS] `STATIC_LIBRARY_SUFFIX :: ".a"`
- `append_type_info_directives :: (builder: *String_Builder, decl: *Declaration)`
- `cache_pool: Pool`
- `contains_equivalent_function :: (declarations: [] *Declaration, fn: *Function) -> bool`
- `count := 0`
- `find_lib_for_foreign_name :: (name: string) -> *Library_Info`
- `find_or_create_extra_info :: (type: *CType, base_struct: *Declaration) -> *Struct_Extra_Info, *Struct`
- `functions_are_equivalent :: (a: *Function, b: *Function, ignore_first_argument := false) -> bool`
- `get_name_alignment :: (declarations: [] *Declaration) -> int`
- `get_name_alignment :: (fields: [] $T, get_declaration: (T) -> *Declaration, ignore_flags := false) -> int`
- `get_primitive_or_array_primitive_type_size :: (type: *CType, decl: *Declaration) -> int`
- `library_symbol_cache: [..] Library_Symbols`
- `make_valid_identifer :: (_struct: string)`
- `os_get_symbol_names_from_file :: (full_path: string) -> success: bool, [] string`
- `os_get_symbol_names_from_file_content :: (file_content: string, full_path: string) -> success: bool, [] string`
- `os_get_symbol_names_from_file_fixed_and_cached :: (full_path: string) -> success: bool, [] string`
- `os_load_library :: (paths: [] string, identifier: string, library_path: string, treat_as_system_library: bool, static_library_suffix := "", shared_library_suffix := "") -> *Library_Info`
- `os_symbol_exists_in_library :: (lib: *Library_Info, name: string) -> bool`
- `print_namespace :: (builder: *String_Builder, ns: *Namespace)`
- `print_struct :: (builder: *String_Builder, _struct: *Struct, existing_declarations: *[..] *Declaration, explicit_alignment := 0)`
- `print_typedef :: (builder: *String_Builder, def: *Typedef)`
- `raw_tokens_string_from_cursor :: (cur: CXCursor) -> string`
- `should_be_moved_to_global_scope :: (decl: *Declaration) -> bool`
- `should_be_omitted :: (decl: *Declaration, parent_decl: *Declaration) -> bool`
- `struct_extra_info_unknown := Struct_Extra_Info.{largest_natural_alignment = -1, struct_alignment = -1}`

## Public type members

#### `Declaration_Visit_Result` (enum)

- `STOP`
- `RECURSE`

#### `Strip_Flags` (enum_flags)

- `CONSTRUCTORS`
- `DESTRUCTORS`
- `FUNCTIONS_WITH_VALIST`
- `SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS`
- `INLINED_FUNCTIONS`

#### `Number_Flags` (enum_flags u8)

- `_8BIT`
- `_16BIT`
- `_32BIT`
- `_64BIT`
- `SIGNED`
- `FLOAT`
- `CHAR`

#### `C_Kind` (enum)

- `NONE`
- `DECLARATION`
- `FUNCTION`
- `ENUM`
- `TYPEDEF`
- `STRUCT`
- `BITFIELD`
- `NAMESPACE`
- `MACRO_DEFINITION`
- `TYPE_REFERENCE`
- `LITERAL`
- `UNARY_OPERATOR`
- `BINARY_OPERATOR`
- `STRUCT_LITERAL`
- `PAREN_EXPRESSION`

#### `Declaration_Flags` (enum_flags)

- `NEEDS_ARGUMENT_WRAPPER`
- `NEEDS_PRINTF_WRAPPER`
- `POINTER_TO_VALUE`
- `OMIT_FROM_OUTPUT`
- `OMIT_FROM_CHECKS`
- `NEEDS_DEFAULT_ARGUMENT_CAST`
- `IS_CONST`
- `IS_METADATA`
- `QUALIFIER_AS`
- `QUALIFIER_USING`
- `TYPE_INFO_NONE`
- `TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS`
- `NO_SPACING`
- `TEMPLATE`
- `DEPRECATED`
- `PROTECTED`
- `PRIVATE`

#### `Print_Decl_Flags` (enum_flags)

- `NO_SEMICOLON`
- `NO_INDENT`
- `IN_FUNCTION_ARGS`
- `IN_FUNCTION_RETURN`
- `IS_GLOBAL`
- `NO_ARGUMENT_DEFAULTS`
- `KEEP_POINTER_TO_VALUE`
- `C_TYPE_DETAILS`
- `NO_PATH`

#### `Arg_Print_Mode` (enum)

- `OUTER_WRAPPER`
- `FOREIGN_FUNC_DECLARATION`
- `INNER_WRAPPER_CALL`

#### `Library_Symbols` (struct)

- This type is importer-visible; its declaration precedes the module-scope transition.
- `os: Operating_System_Tag`
- `cpu: CPU_Tag`
- `full_path: string`
- `symbol_names: [] string`

#### `CXCursorSetImpl` (empty opaque struct)

```jai
CXCursorSetImpl :: struct {}
```
#### `CXModuleMapDescriptorImpl` (empty opaque struct)

```jai
CXModuleMapDescriptorImpl :: struct {}
```
#### `CXTargetInfoImpl` (empty opaque struct)

```jai
CXTargetInfoImpl :: struct {}
```
#### `CXTranslationUnitImpl` (empty opaque struct)

```jai
CXTranslationUnitImpl :: struct {}
```
#### `CXVirtualFileOverlayImpl` (empty opaque struct)

```jai
CXVirtualFileOverlayImpl :: struct {}
```
#### `Library` (struct)

- `filename:   string`
- `identifier: string`

#### `Generate_Bindings_Options` (struct)

- `os                    := TARGET_OS`
- `cpu                   := CPU`
- `library_search_paths:        [..] string`
- `libraries:                   [..] Library`
- `system_library_search_paths: [..] string`
- `system_libraries:            [..] Library`
- `libnames:             [..] string`
- `libpaths:             [..] string`
- `system_library_names: [..] string`
- `system_library_paths: [..] string`
- `static_library_suffix: string`
- `shared_library_suffix: string`
- `system_include_paths:  [..] string`
- `include_paths:         [..] string`
- `source_files:          [..] string`
- `extra_clang_arguments: [..] string`
- `path_fragments_to_treat_as_non_system_paths: [..] string`
- `system_types_to_include:                     [..] string`
- `log_ignored_system_declarations              := false`
- `flatten_namespaces:             [..] string`
- `default_namespace_flags         : Declaration_Flags`
- `strip_prefixes:                 [..] string`
- `generate_enums_from_macros_with_prefixes: [..] string`
- `convert_macro_value_to_enum_callback: #type (decl: *Declaration) -> enum_name: string`
- `strip_enum_prefixes             := true`
- `auto_detect_enum_prefixes       := true`
- `enum_prefix_delimiter           := "_"`
- `alias_original_enum_names       := true`
- `strip_enum_values_with_suffix:  string`
- `strip_enum_value_prefixes:      [..][2]string`
- `typedef_prefixes_to_unwrap:     [..] string`
- `macro_prefixes_to_unwrap:       [..] string`
- `move_struct_functions_to_global_scope := false`
- `omit_global_declarations        := false`
- `strip_flags := Strip_Flags.FUNCTIONS_WITH_VALIST | .SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS`
- `log_stripped_declarations       := true`
- `try_to_preserve_comments        := true`
- `generate_library_declarations := true`
- `generated_library_path_prefix := ""`
- `add_generator_command           := true`
- `header                          := ""`
- `footer                          := ""`
- `c_enum_emulation                      := false`
- `prepare_all_structs_for_inheritance   := false`
- `generate_compile_time_struct_checks   := true`
- `generate_printf_wrappers              := true`
- `generate_c_style_api_for_vtable       := true`
- `generate_vtable_helpers               := true`
- `generate_bitfield_helpers             := true`
- `add_overloaded_const_and_ref_comments := true`
- `add_placeholders_for_unresolved_types := true`
- `mimic_spacing_flags             := Mimic_Spacing_Flags.ALL`
- `get_func_args_for_printing: Get_Function_Arguments_For_Printing_Func = _default_get_func_args_for_printing`
- `will_print_bindings: #type ()`
- `visitor:           Declaration_Visitor`
- `visitor_with_data: Declaration_Visitor_With_Data`
- `visitor_data:      *void`
- `log_unsupported                 := false`

#### `Mimic_Spacing_Flags` (enum_flags)

- `GLOBAL`
- `STRUCT`
- `ENUM`
- `NAMESPACE`
- `VTABLE`
- `ALL :: GLOBAL | STRUCT | ENUM | NAMESPACE | VTABLE`

#### `Library_Info` (struct)

- `identifier: string`
- `full_path: string`
- `handle: HMODULE` when compiling for Windows; `handle: *void` when compiling for Linux or macOS
- `is_shared_library: bool`
- `is_system_library: bool`
- `symbol_names: [] string`
- `flags: Flags`
- `Flags :: enum_flags { LINK_ALWAYS }`

#### `Bridge_Data` (struct)

- `functions: [..] *Function`
- `needs_placement_new: bool`

#### `Cpp_Functions_Visitor_Data` (struct)

- `visitor: Declaration_Visitor`
- `visitor_with_data: Declaration_Visitor_With_Data`
- `visitor_data: *void`

#### `ComUuid` (struct)

- `location: Location`
- `uuid: string`

#### `Generator` (struct)

- This type is importer-visible. It is declared before the module's module-scope transition; its `#type_info_none` directive is part of its declaration.
- `global_scope: Block`
- `pointer_size: s64`
- `anon_declarations: [..] *Declaration`
- `type_def_void: *CType`
- `type_def_bool: *CType`
- `type_def_char: *CType`
- `type_def_u8:   *CType`
- `type_def_u16:  *CType`
- `type_def_u32:  *CType`
- `type_def_u64:  *CType`
- `type_def_u128: *CType`
- `type_def_s8:   *CType`
- `type_def_s16:  *CType`
- `type_def_s32:  *CType`
- `type_def_s64:  *CType`
- `type_def_s128: *CType`
- `type_def_float32: *CType`
- `type_def_float64: *CType`
- `type_def_pointer_to_void: *CType`
- `type_def_ptr_void: *CType`
- `type_def_objc_class: *CType`
- `type_def_objc_selector: *CType`
- `type_def_objc_block_pointer: *CType`
- `type_def_objc_id: *CType`
- `type_def_jai_string: *CType`
- `type_def_type: *CType`
- `type_def_msvc_destructor_flags: *CType`
- `declarations_by_cursor: Table(CXCursor, *Declaration, get_hash_cxcursor, compare_cxcursors)`
- `macro_token_replacements: [..] Replacement`
- `Replacement :: struct { location: Location; replacement: string; }`
- `enum_macros: [..] *Enum`
- `macros_converted_to_enum_values: Table(string, *Enum)`
- `unwrapped_typedefs: Table(string, *Typedef)`
- `extern_declarations: [..] *Declaration`
- `unique_objc_selectors: [..] string`
- `libraries: [..] *Library_Info`
- `com_uuids: [..] ComUuid`
- `extra_infos: Table(*Struct, Struct_Extra_Info)`
- `pending_bitfields: [..] *Bitfield`
- `structs_to_check: [..] *Struct`
- `flattened_global_scope_members: [] *Declaration`
- `declarations_for_global_scope: [..] *Declaration`
- `int128_was_referenced: bool`
- `generated_printf_wrapper: bool`
- `tab_depth := 0`
- `template_depth := 0`
- `counts: struct {`
- `clang_index: CXIndex`
- `translation_unit: CXTranslationUnit`
- `tokens_to_free: [..] Token_Info`
- `filenames: Table(CXFile, string)`
- `counts: struct { ignored_system_declarations: u32; stripped: u32; functions: u32; structs: u32; enums: u32; }`

#### `Token_Info` (struct)

- `token_pointer: *CXToken`
- `num_tokens: u32`

#### `CType` (struct)

- `hardcoded_jai_string: string`
- `pointer_to: *CType`
- `type_of_enum: *Enum`
- `type_of_typedef: *Typedef`
- `type_of_struct: *Struct`
- `type_of_declaration: *Declaration`
- `array_element_type: *CType`
- `array_element_count := -1`
- `array_element_count_expression: *Expression`
- `type_of_function: *Function_Type`
- `template_instantiation_params: [..] *Expression`
- `size: s64 = xx CXTypeLayoutError.Invalid`
- `alignment: s64 = -1`
- `number_flags: Number_Flags`
- `flags: Flags`
- `Flags :: enum_flags u8 { REFERENCE; CONST; VOLATILE; INSERTED; SHARED; VOID; BOOL }`
- `comment: string`

#### `Function_Type` (struct)

- `arguments: [..] *Declaration`
- `return_type: *CType`
- `flags: Flags`
- `Flags :: enum_flags { ADD_C_TYPE_DETAILS; INLINE; OBJC_METHOD; OBJC_CLASS_METHOD; VARIADIC; CPP_METHOD; CONSTRUCTOR; COPY_CONSTRUCTOR; MOVE_CONSTRUCTOR; DESTRUCTOR }`

#### `Expression` (struct)

- `kind: C_Kind`

#### `Type_Reference` (struct)

- `#as using _expression: Expression;`

- `type: *CType`

#### `Literal` (struct)

```jai
Literal :: struct {
    #as using _expression: Expression;
    kind = .LITERAL;
    literal_kind: enum int {
        NULL             :: 0;
        INTEGER          :: 1;
        UNSIGNED_INTEGER :: 2;
        FLOAT            :: 3;
        STRING           :: 4;
        RAW_CPP_TOKENS   :: 5;
        ENUM             :: 6;
        MACRO            :: 7;
        IDENTIFIER       :: 8;
    };
    union {
        int_value:    s64;
        uint_value:   u64;
        float_value:  float64;
        string_value: string;
        macro_value:  Macro_Value;
        struct { enum_type: *Enum; enum_value: *Declaration; }
        identifier: *Declaration;
    }
    raw_value: string;
    template_instantiation_params: [..] *Expression;
    location: Location;
}
```
#### `Macro_Value` (struct)

- `tokens: [] CXToken`

#### `Unary_Operator` (struct)

- `#as using _expression: Expression;`

- `op: string`
- `child: *Expression`

#### `Binary_Operator` (struct)

- `#as using _expression: Expression;`

- `op: string`
- `lhs: *Expression`
- `rhs: *Expression`

#### `Struct_Literal` (struct)

- `#as using _expression: Expression;`

- `type_name: string`
- `values: [] *Expression`

#### `Paren_Expression` (struct)

- `#as using _expression: Expression;`

- `child: *Expression`

#### `Location` (struct)

- `file: CXFile`
- `line: u32`
- `column: u32`

#### `Comment` (struct)

- `location: Location`
- `text: string`

#### `Declaration` (struct)

- `#as using _expression: Expression;`

- `location:           Location`
- `name:               string`
- `output_name:        string`
- `outputted_jai_name: string`
- `type:               *CType`
- `expression:         *Expression`
- `parent:             *Declaration`
- `should_unwrap:      bool`
- `bitfield_width           := -1`
- `bit_offset_in_parent     := -1`
- `field_alignment          := 0`
- `explicit_field_alignment := 0`
- `macro_value_result: enum {`
- `storage_class: CX_StorageClass = .None`
- `decl_flags: Declaration_Flags`
- `using_except: string`
- `comment: Comment`
- `library: *Library_Info`
- `foreign_name: string`
- `size_condition_type: *CType`

#### `Function` (struct)

- `#as using declaration: Declaration;`

- `deleting_foreign_name: string`
- `base_foreign_name: string`
- `flags: Function_Flags`
- `Function_Flags :: enum_flags { VIRTUAL; PURE_VIRTUAL }`

#### `Typedef` (struct)

- `#as using declaration: Declaration;`

- `unwrap: bool`
- `flags: Typedef_Flags`
- `Typedef_Flags :: enum_flags { ISA; DISTINCT }`

#### `Enum` (struct)

- `#as using declaration: Declaration;`

- `enumerates: [..] *Declaration`
- `flags: Enum_Flags`
- `Enum_Flags :: enum_flags { IS_ENUM_FLAGS; VALUES_IN_HEX; MIMIC_SPACING; ALIAS_ORIGINAL_NAMES }`

#### `Bitfield` (struct)

- `#as using declaration: Declaration;`

- `members: [..] *Declaration`
- `alignment_type: *CType`
- `flags: Flags`
- `Flags :: enum_flags u8 { GENERATE_HELPERS }`

#### `Bitfield_Status` (struct)

- `parent:     *Struct`
- `bitfield:   *Bitfield`
- `first_decl: *Declaration`

#### `Declaration_Overlay` (struct)

- `place: *Declaration`
- `declaration: *Declaration`

#### `Struct` (struct)

- `#as using declaration: Declaration;`

- `metadata:             [..] *Declaration`
- `parents:              [..] *Declaration`
- `virtual_functions:    [..] *Function`
- `declarations:         [..] *Declaration`
- `template_type_params: [..] *Declaration`
- `overlays:             [..] Declaration_Overlay`
- `primary_field:        *Declaration`
- `flags:                Flags`
- `Flags :: enum_flags { NONE; IS_ANON; IS_ANON_INLINE_TYPE; IS_UNION; HAS_CONSTRUCTOR; HAS_COPY_CONSTRUCTOR; HAS_MOVE_CONSTRUCTOR; HAS_DESTRUCTOR; NO_STRUCT_CHECKS; MIMIC_SPACING; OBJC_PROTOCOL; OBJC_INTERFACE; OBJC_CATEGORY; IS_RAW; IS_ABSTRACT; IS_VTABLE; MAY_HAVE_SUBCLASS }`

#### `Struct_Extra_Info` (struct)

- `vtable:                     [..] *Function`
- `vtable_struct:              *Struct`
- `vtable_pointer_declaration: *Declaration`
- `vtable_parent:              *Struct`
- `vtable_parent_info:         *Struct_Extra_Info`
- `vtable_parent_declaration:  *Declaration`
- `padding_declaration:        *Declaration`
- `largest_natural_alignment:  int`
- `struct_alignment:           int`
- `has_field:                  bool`
- `has_nonzero_sized_field:    bool`
- `inherits_from_template:     bool`
- `modified_declarations:      [..] *Declaration`
- `all_scope_declarations:     [..] *Declaration`
- `referenced_type_names:      [..] string`
- `raw_struct:                 *Struct`

#### `Namespace` (struct)

- `#as using declaration: Declaration;`

- `flags: Flags`
- `Flags :: enum_flags u8 { FLATTEN :: 0x01; IS_EMPTY :: 0x02 }`
- `declarations: [..] *Declaration`

#### `Block` (struct)

- `parent: *Block`
- `members: [..] *Declaration`

#### `Generate_Bindings_Options` (struct)

- `source_files:          [..] string`
- `extra_clang_arguments: [..] string`
- `try_to_preserve_comments: bool`

#### `Location` (struct)

- `file: CXFile`
- `line: u32`
- `column: u32`

### Unix-specific type declarations

#### `CXErrorCode` (enum)

- `Success          :: 0`
- `Failure          :: 1`
- `Crashed          :: 2`
- `InvalidArguments :: 3`
- `ASTReadError     :: 4`

#### `CXString` (struct)

- `data:          *void`
- `private_flags: u32`

#### `CXStringSet` (struct)

- `Strings: *CXString`
- `Count:   u32`

#### `CXFileUniqueID` (struct)

- `data: [3] u64`

#### `CXSourceLocation` (struct)

- `ptr_data: [2] *void`
- `int_data: u32`

#### `CXSourceRange` (struct)

- `ptr_data:       [2] *void`
- `begin_int_data: u32`
- `end_int_data:   u32`

#### `CXSourceRangeList` (struct)

- `count:  u32`
- `ranges: *CXSourceRange`

#### `CXDiagnosticSeverity` (enum)

- `Ignored :: 0`
- `Note    :: 1`
- `Warning :: 2`
- `Error   :: 3`
- `Fatal   :: 4`

#### `CXLoadDiag_Error` (enum)

- `None        :: 0`
- `Unknown     :: 1`
- `CannotLoad  :: 2`
- `InvalidFile :: 3`

#### `CXDiagnosticDisplayOptions` (enum)

- `DisplaySourceLocation :: 1`
- `DisplayColumn         :: 2`
- `DisplaySourceRanges   :: 4`
- `DisplayOption         :: 8`
- `DisplayCategoryId     :: 16`
- `DisplayCategoryName   :: 32`

#### `CXUnsavedFile` (struct)

- `Filename: *u8`
- `Contents: *u8`
- `Length:   u64`

#### `CXAvailabilityKind` (enum)

- `Available     :: 0`
- `Deprecated    :: 1`
- `NotAvailable  :: 2`
- `NotAccessible :: 3`

#### `CXVersion` (struct)

- `Major:    s32`
- `Minor:    s32`
- `Subminor: s32`

#### `CXCursor_ExceptionSpecificationKind` (enum)

- `None             :: 0`
- `DynamicNone      :: 1`
- `Dynamic          :: 2`
- `MSAny            :: 3`
- `BasicNoexcept    :: 4`
- `ComputedNoexcept :: 5`
- `Unevaluated      :: 6`
- `Uninstantiated   :: 7`
- `Unparsed         :: 8`
- `NoThrow          :: 9`

#### `CXChoice` (enum)

- `Default  :: 0`
- `Enabled  :: 1`
- `Disabled :: 2`

#### `CXGlobalOptFlags` (enum)

- `None                                :: 0`
- `ThreadBackgroundPriorityForIndexing :: 1`
- `ThreadBackgroundPriorityForEditing  :: 2`
- `ThreadBackgroundPriorityForAll      :: 3`

#### `CXIndexOptions` (struct)

- `Size:                                u32`
- `ThreadBackgroundPriorityForIndexing: u8`
- `ThreadBackgroundPriorityForEditing:  u8`
- `__bitfield:                          u16`
- `PreambleStoragePath:                 *u8`
- `InvocationEmissionPath:              *u8`

The three named boolean options are bit views, not additional aggregate fields: `set_ExcludeDeclarationsFromPCH`/`get_ExcludeDeclarationsFromPCH` use bit 0 of `__bitfield`, `set_DisplayDiagnostics`/`get_DisplayDiagnostics` use bit 1, and `set_StorePreamblesInMemory`/`get_StorePreamblesInMemory` use bit 2. The remaining bits are reserved by the associated C interface and have no named Jai accessor. The Jai record's public storage is `Size`, the two one-byte priorities, a 16-bit bitfield word, then the two path pointers. With 64-bit pointers this yields the native 24-byte record and pointer offsets 8 and 16; with 32-bit pointers the final pointer fields follow the same declared order with target-appropriate alignment. Fields have no explicit Jai initializer; ordinary omitted-field initialization applies.

#### `CXTranslationUnit_Flags` (enum_flags)

- `None                                 :: 0x0`
- `DetailedPreprocessingRecord          :: 0x1`
- `Incomplete                           :: 0x2`
- `PrecompiledPreamble                  :: 0x4`
- `CacheCompletionResults               :: 0x8`
- `ForSerialization                     :: 0x10`
- `CXXChainedPCH                        :: 0x20`
- `SkipFunctionBodies                   :: 0x40`
- `IncludeBriefCommentsInCodeCompletion :: 0x80`
- `CreatePreambleOnFirstParse           :: 0x100`
- `KeepGoing                            :: 0x200`
- `SingleFileParse                      :: 0x400`
- `LimitSkipFunctionBodiesToPreamble    :: 0x800`
- `IncludeAttributedTypes               :: 0x1000`
- `VisitImplicitAttributes              :: 0x2000`
- `IgnoreNonErrorsFromIncludedFiles     :: 0x4000`
- `RetainExcludedConditionalBlocks      :: 0x8000`

#### `CXSaveTranslationUnit_Flags` (enum_flags)

- `CXSaveTranslationUnit_None :: 0x0`

#### `CXSaveError` (enum)

- `None              :: 0`
- `Unknown           :: 1`
- `TranslationErrors :: 2`
- `InvalidTU         :: 3`

#### `CXReparse_Flags` (enum_flags)

- `CXReparse_None :: 0x0`

#### `CXTUResourceUsageKind` (enum)

- `AST                                :: 1`
- `Identifiers                        :: 2`
- `Selectors                          :: 3`
- `GlobalCompletionResults            :: 4`
- `SourceManagerContentCache          :: 5`
- `AST_SideTables                     :: 6`
- `SourceManager_Membuffer_Malloc     :: 7`
- `SourceManager_Membuffer_MMap       :: 8`
- `ExternalASTSource_Membuffer_Malloc :: 9`
- `ExternalASTSource_Membuffer_MMap   :: 10`
- `Preprocessor                       :: 11`
- `PreprocessingRecord                :: 12`
- `SourceManager_DataStructures       :: 13`
- `Preprocessor_HeaderSearch          :: 14`
- `MEMORY_IN_BYTES_BEGIN              :: 1`
- `MEMORY_IN_BYTES_END                :: 14`
- `First                              :: 1`
- `Last                               :: 14`

#### `CXTUResourceUsageEntry` (struct)

- `kind:   CXTUResourceUsageKind`
- `amount: u64`

#### `CXTUResourceUsage` (struct)

- `data:       *void`
- `numEntries: u32`
- `entries:    *CXTUResourceUsageEntry`

#### `CXCursorKind` (enum)

- `UnexposedDecl                                    :: 1`
- `StructDecl                                       :: 2`
- `UnionDecl                                        :: 3`
- `ClassDecl                                        :: 4`
- `EnumDecl                                         :: 5`
- `FieldDecl                                        :: 6`
- `EnumConstantDecl                                 :: 7`
- `FunctionDecl                                     :: 8`
- `VarDecl                                          :: 9`
- `ParmDecl                                         :: 10`
- `ObjCInterfaceDecl                                :: 11`
- `ObjCCategoryDecl                                 :: 12`
- `ObjCProtocolDecl                                 :: 13`
- `ObjCPropertyDecl                                 :: 14`
- `ObjCIvarDecl                                     :: 15`
- `ObjCInstanceMethodDecl                           :: 16`
- `ObjCClassMethodDecl                              :: 17`
- `ObjCImplementationDecl                           :: 18`
- `ObjCCategoryImplDecl                             :: 19`
- `TypedefDecl                                      :: 20`
- `CXXMethod                                        :: 21`
- `Namespace                                        :: 22`
- `LinkageSpec                                      :: 23`
- `Constructor                                      :: 24`
- `Destructor                                       :: 25`
- `ConversionFunction                               :: 26`
- `TemplateTypeParameter                            :: 27`
- `NonTypeTemplateParameter                         :: 28`
- `TemplateTemplateParameter                        :: 29`
- `FunctionTemplate                                 :: 30`
- `ClassTemplate                                    :: 31`
- `ClassTemplatePartialSpecialization               :: 32`
- `NamespaceAlias                                   :: 33`
- `UsingDirective                                   :: 34`
- `UsingDeclaration                                 :: 35`
- `TypeAliasDecl                                    :: 36`
- `ObjCSynthesizeDecl                               :: 37`
- `ObjCDynamicDecl                                  :: 38`
- `CXXAccessSpecifier                               :: 39`
- `OverloadCandidate                                :: 700`
- `ConceptDecl                                      :: 604`
- `FriendDecl                                       :: 603`
- `ObjCSuperClassRef                                :: 40`
- `ObjCProtocolRef                                  :: 41`
- `ObjCClassRef                                     :: 42`
- `TypeRef                                          :: 43`
- `CXXBaseSpecifier                                 :: 44`
- `TemplateRef                                      :: 45`
- `NamespaceRef                                     :: 46`
- `MemberRef                                        :: 47`
- `LabelRef                                         :: 48`
- `OverloadedDeclRef                                :: 49`
- `VariableRef                                      :: 50`
- `StaticAssert                                     :: 602`
- `TypeAliasTemplateDecl                            :: 601`
- `InvalidFile                                      :: 70`
- `NoDeclFound                                      :: 71`
- `NotImplemented                                   :: 72`
- `InvalidCode                                      :: 73`
- `ModuleImportDecl                                 :: 600`
- `InclusionDirective                               :: 503`
- `UnexposedExpr                                    :: 100`
- `DeclRefExpr                                      :: 101`
- `MemberRefExpr                                    :: 102`
- `CallExpr                                         :: 103`
- `ObjCMessageExpr                                  :: 104`
- `BlockExpr                                        :: 105`
- `IntegerLiteral                                   :: 106`
- `FloatingLiteral                                  :: 107`
- `ImaginaryLiteral                                 :: 108`
- `StringLiteral                                    :: 109`
- `CharacterLiteral                                 :: 110`
- `ParenExpr                                        :: 111`
- `UnaryOperator                                    :: 112`
- `ArraySubscriptExpr                               :: 113`
- `BinaryOperator                                   :: 114`
- `CompoundAssignOperator                           :: 115`
- `ConditionalOperator                              :: 116`
- `CStyleCastExpr                                   :: 117`
- `CompoundLiteralExpr                              :: 118`
- `InitListExpr                                     :: 119`
- `AddrLabelExpr                                    :: 120`
- `StmtExpr                                         :: 121`
- `GenericSelectionExpr                             :: 122`
- `GNUNullExpr                                      :: 123`
- `CXXStaticCastExpr                                :: 124`
- `CXXDynamicCastExpr                               :: 125`
- `CXXReinterpretCastExpr                           :: 126`
- `CXXConstCastExpr                                 :: 127`
- `CXXFunctionalCastExpr                            :: 128`
- `CXXTypeidExpr                                    :: 129`
- `CXXBoolLiteralExpr                               :: 130`
- `CXXNullPtrLiteralExpr                            :: 131`
- `CXXThisExpr                                      :: 132`
- `CXXThrowExpr                                     :: 133`
- `CXXNewExpr                                       :: 134`
- `CXXDeleteExpr                                    :: 135`
- `UnaryExpr                                        :: 136`
- `ObjCStringLiteral                                :: 137`
- `ObjCEncodeExpr                                   :: 138`
- `ObjCSelectorExpr                                 :: 139`
- `ObjCProtocolExpr                                 :: 140`
- `ObjCBridgedCastExpr                              :: 141`
- `PackExpansionExpr                                :: 142`
- `SizeOfPackExpr                                   :: 143`
- `LambdaExpr                                       :: 144`
- `ObjCBoolLiteralExpr                              :: 145`
- `ObjCSelfExpr                                     :: 146`
- `ArraySectionExpr                                 :: 147`
- `ObjCAvailabilityCheckExpr                        :: 148`
- `FixedPointLiteral                                :: 149`
- `OMPArrayShapingExpr                              :: 150`
- `OMPIteratorExpr                                  :: 151`
- `CXXAddrspaceCastExpr                             :: 152`
- `ConceptSpecializationExpr                        :: 153`
- `RequiresExpr                                     :: 154`
- `CXXParenListInitExpr                             :: 155`
- `PackIndexingExpr                                 :: 156`
- `MacroInstantiation                               :: 502`
- `MacroExpansion                                   :: 502`
- `UnexposedStmt                                    :: 200`
- `LabelStmt                                        :: 201`
- `CompoundStmt                                     :: 202`
- `CaseStmt                                         :: 203`
- `DefaultStmt                                      :: 204`
- `IfStmt                                           :: 205`
- `SwitchStmt                                       :: 206`
- `WhileStmt                                        :: 207`
- `DoStmt                                           :: 208`
- `ForStmt                                          :: 209`
- `GotoStmt                                         :: 210`
- `IndirectGotoStmt                                 :: 211`
- `ContinueStmt                                     :: 212`
- `BreakStmt                                        :: 213`
- `ReturnStmt                                       :: 214`
- `GCCAsmStmt                                       :: 215`
- `AsmStmt                                          :: 215`
- `ObjCAtTryStmt                                    :: 216`
- `ObjCAtCatchStmt                                  :: 217`
- `ObjCAtFinallyStmt                                :: 218`
- `ObjCAtThrowStmt                                  :: 219`
- `ObjCAtSynchronizedStmt                           :: 220`
- `ObjCAutoreleasePoolStmt                          :: 221`
- `ObjCForCollectionStmt                            :: 222`
- `CXXCatchStmt                                     :: 223`
- `CXXTryStmt                                       :: 224`
- `CXXForRangeStmt                                  :: 225`
- `SEHTryStmt                                       :: 226`
- `SEHExceptStmt                                    :: 227`
- `SEHFinallyStmt                                   :: 228`
- `MSAsmStmt                                        :: 229`
- `NullStmt                                         :: 230`
- `DeclStmt                                         :: 231`
- `OMPParallelDirective                             :: 232`
- `OMPSimdDirective                                 :: 233`
- `OMPForDirective                                  :: 234`
- `OMPSectionsDirective                             :: 235`
- `OMPSectionDirective                              :: 236`
- `OMPSingleDirective                               :: 237`
- `OMPParallelForDirective                          :: 238`
- `OMPParallelSectionsDirective                     :: 239`
- `OMPTaskDirective                                 :: 240`
- `OMPMasterDirective                               :: 241`
- `OMPCriticalDirective                             :: 242`
- `OMPTaskyieldDirective                            :: 243`
- `OMPBarrierDirective                              :: 244`
- `OMPTaskwaitDirective                             :: 245`
- `OMPFlushDirective                                :: 246`
- `SEHLeaveStmt                                     :: 247`
- `OMPOrderedDirective                              :: 248`
- `OMPAtomicDirective                               :: 249`
- `OMPForSimdDirective                              :: 250`
- `OMPParallelForSimdDirective                      :: 251`
- `OMPTargetDirective                               :: 252`
- `OMPTeamsDirective                                :: 253`
- `OMPTaskgroupDirective                            :: 254`
- `OMPCancellationPointDirective                    :: 255`
- `OMPCancelDirective                               :: 256`
- `OMPTargetDataDirective                           :: 257`
- `OMPTaskLoopDirective                             :: 258`
- `OMPTaskLoopSimdDirective                         :: 259`
- `OMPDistributeDirective                           :: 260`
- `OMPTargetEnterDataDirective                      :: 261`
- `OMPTargetExitDataDirective                       :: 262`
- `OMPTargetParallelDirective                       :: 263`
- `OMPTargetParallelForDirective                    :: 264`
- `OMPTargetUpdateDirective                         :: 265`
- `OMPDistributeParallelForDirective                :: 266`
- `OMPDistributeParallelForSimdDirective            :: 267`
- `OMPDistributeSimdDirective                       :: 268`
- `OMPTargetParallelForSimdDirective                :: 269`
- `OMPTargetSimdDirective                           :: 270`
- `OMPTeamsDistributeDirective                      :: 271`
- `OMPTeamsDistributeSimdDirective                  :: 272`
- `OMPTeamsDistributeParallelForSimdDirective       :: 273`
- `OMPTeamsDistributeParallelForDirective           :: 274`
- `OMPTargetTeamsDirective                          :: 275`
- `OMPTargetTeamsDistributeDirective                :: 276`
- `OMPTargetTeamsDistributeParallelForDirective     :: 277`
- `OMPTargetTeamsDistributeParallelForSimdDirective :: 278`
- `OMPTargetTeamsDistributeSimdDirective            :: 279`
- `BuiltinBitCastExpr                               :: 280`
- `OMPMasterTaskLoopDirective                       :: 281`
- `OMPParallelMasterTaskLoopDirective               :: 282`
- `OMPMasterTaskLoopSimdDirective                   :: 283`
- `OMPParallelMasterTaskLoopSimdDirective           :: 284`
- `OMPParallelMasterDirective                       :: 285`
- `OMPDepobjDirective                               :: 286`
- `OMPScanDirective                                 :: 287`
- `OMPTileDirective                                 :: 288`
- `OMPCanonicalLoop                                 :: 289`
- `OMPInteropDirective                              :: 290`
- `OMPDispatchDirective                             :: 291`
- `OMPMaskedDirective                               :: 292`
- `OMPUnrollDirective                               :: 293`
- `OMPMetaDirective                                 :: 294`
- `OMPGenericLoopDirective                          :: 295`
- `OMPTeamsGenericLoopDirective                     :: 296`
- `OMPTargetTeamsGenericLoopDirective               :: 297`
- `OMPParallelGenericLoopDirective                  :: 298`
- `OMPTargetParallelGenericLoopDirective            :: 299`
- `OMPParallelMaskedDirective                       :: 300`
- `OMPMaskedTaskLoopDirective                       :: 301`
- `OMPMaskedTaskLoopSimdDirective                   :: 302`
- `OMPParallelMaskedTaskLoopDirective               :: 303`
- `OMPParallelMaskedTaskLoopSimdDirective           :: 304`
- `OMPErrorDirective                                :: 305`
- `OMPScopeDirective                                :: 306`
- `OMPReverseDirective                              :: 307`
- `OMPInterchangeDirective                          :: 308`
- `OMPAssumeDirective                               :: 309`
- `OMPStripeDirective                               :: 310`
- `OpenACCComputeConstruct                          :: 320`
- `OpenACCLoopConstruct                             :: 321`
- `OpenACCCombinedConstruct                         :: 322`
- `OpenACCDataConstruct                             :: 323`
- `OpenACCEnterDataConstruct                        :: 324`
- `OpenACCExitDataConstruct                         :: 325`
- `OpenACCHostDataConstruct                         :: 326`
- `OpenACCWaitConstruct                             :: 327`
- `OpenACCInitConstruct                             :: 328`
- `OpenACCShutdownConstruct                         :: 329`
- `OpenACCSetConstruct                              :: 330`
- `OpenACCUpdateConstruct                           :: 331`
- `OpenACCAtomicConstruct                           :: 332`
- `OpenACCCacheConstruct                            :: 333`
- `MacroDefinition                                  :: 501`
- `TranslationUnit                                  :: 350`
- `PreprocessingDirective                           :: 500`
- `UnexposedAttr                                    :: 400`
- `IBActionAttr                                     :: 401`
- `IBOutletAttr                                     :: 402`
- `IBOutletCollectionAttr                           :: 403`
- `CXXFinalAttr                                     :: 404`
- `CXXOverrideAttr                                  :: 405`
- `AnnotateAttr                                     :: 406`
- `AsmLabelAttr                                     :: 407`
- `PackedAttr                                       :: 408`
- `PureAttr                                         :: 409`
- `ConstAttr                                        :: 410`
- `NoDuplicateAttr                                  :: 411`
- `CUDAConstantAttr                                 :: 412`
- `CUDADeviceAttr                                   :: 413`
- `CUDAGlobalAttr                                   :: 414`
- `CUDAHostAttr                                     :: 415`
- `CUDASharedAttr                                   :: 416`
- `VisibilityAttr                                   :: 417`
- `DLLExport                                        :: 418`
- `DLLImport                                        :: 419`
- `NSReturnsRetained                                :: 420`
- `NSReturnsNotRetained                             :: 421`
- `NSReturnsAutoreleased                            :: 422`
- `NSConsumesSelf                                   :: 423`
- `NSConsumed                                       :: 424`
- `ObjCException                                    :: 425`
- `ObjCNSObject                                     :: 426`
- `ObjCIndependentClass                             :: 427`
- `ObjCPreciseLifetime                              :: 428`
- `ObjCReturnsInnerPointer                          :: 429`
- `ObjCRequiresSuper                                :: 430`
- `ObjCRootClass                                    :: 431`
- `ObjCSubclassingRestricted                        :: 432`
- `ObjCExplicitProtocolImpl                         :: 433`
- `ObjCDesignatedInitializer                        :: 434`
- `ObjCRuntimeVisible                               :: 435`
- `ObjCBoxable                                      :: 436`
- `FlagEnum                                         :: 437`
- `ConvergentAttr                                   :: 438`
- `WarnUnusedAttr                                   :: 439`
- `WarnUnusedResultAttr                             :: 440`
- `AlignedAttr                                      :: 441`
- `FirstDecl                                        :: 1`
- `LastDecl                                         :: 39`
- `LastExtraDecl                                    :: 604`
- `FirstExtraDecl                                   :: 600`
- `FirstRef                                         :: 40`
- `LastRef                                          :: 50`
- `FirstInvalid                                     :: 70`
- `LastInvalid                                      :: 73`
- `FirstExpr                                        :: 100`
- `LastPreprocessing                                :: 503`
- `FirstPreprocessing                               :: 500`
- `LastExpr                                         :: 156`
- `FirstStmt                                        :: 200`
- `LastStmt                                         :: 333`
- `FirstAttr                                        :: 400`
- `LastAttr                                         :: 441`

#### `CXCursor` (struct)

- `kind:  CXCursorKind`
- `xdata: s32`
- `data:  [3] *void`

#### `CXLinkageKind` (enum)

- `Invalid        :: 0`
- `NoLinkage      :: 1`
- `Internal       :: 2`
- `UniqueExternal :: 3`
- `External       :: 4`

#### `CXVisibilityKind` (enum)

- `Invalid   :: 0`
- `Hidden    :: 1`
- `Protected :: 2`
- `Default   :: 3`

#### `CXPlatformAvailability` (struct)

- `Platform:    CXString`
- `Introduced:  CXVersion`
- `Deprecated:  CXVersion`
- `Obsoleted:   CXVersion`
- `Unavailable: s32`
- `Message:     CXString`

#### `CXLanguageKind` (enum)

- `Invalid   :: 0`
- `C         :: 1`
- `ObjC      :: 2`
- `CPlusPlus :: 3`

#### `CXTLSKind` (enum)

- `None    :: 0`
- `Dynamic :: 1`
- `Static  :: 2`

#### `CXTypeKind` (enum)

- `Invalid                                              :: 0`
- `Unexposed                                            :: 1`
- `Void                                                 :: 2`
- `Bool                                                 :: 3`
- `Char_U                                               :: 4`
- `UChar                                                :: 5`
- `Char16                                               :: 6`
- `Char32                                               :: 7`
- `UShort                                               :: 8`
- `UInt                                                 :: 9`
- `ULong                                                :: 10`
- `ULongLong                                            :: 11`
- `UInt128                                              :: 12`
- `Char_S                                               :: 13`
- `SChar                                                :: 14`
- `WChar                                                :: 15`
- `Short                                                :: 16`
- `Int                                                  :: 17`
- `Long                                                 :: 18`
- `LongLong                                             :: 19`
- `Int128                                               :: 20`
- `Float                                                :: 21`
- `Double                                               :: 22`
- `LongDouble                                           :: 23`
- `NullPtr                                              :: 24`
- `Overload                                             :: 25`
- `Dependent                                            :: 26`
- `ObjCId                                               :: 27`
- `ObjCClass                                            :: 28`
- `ObjCSel                                              :: 29`
- `Float128                                             :: 30`
- `Half                                                 :: 31`
- `Float16                                              :: 32`
- `ShortAccum                                           :: 33`
- `Accum                                                :: 34`
- `LongAccum                                            :: 35`
- `UShortAccum                                          :: 36`
- `UAccum                                               :: 37`
- `ULongAccum                                           :: 38`
- `BFloat16                                             :: 39`
- `Ibm128                                               :: 40`
- `HLSLInlineSpirv                                      :: 181`
- `HLSLAttributedResource                               :: 180`
- `Complex                                              :: 100`
- `Pointer                                              :: 101`
- `BlockPointer                                         :: 102`
- `LValueReference                                      :: 103`
- `RValueReference                                      :: 104`
- `Record                                               :: 105`
- `Enum                                                 :: 106`
- `Typedef                                              :: 107`
- `ObjCInterface                                        :: 108`
- `ObjCObjectPointer                                    :: 109`
- `FunctionNoProto                                      :: 110`
- `FunctionProto                                        :: 111`
- `ConstantArray                                        :: 112`
- `Vector                                               :: 113`
- `IncompleteArray                                      :: 114`
- `VariableArray                                        :: 115`
- `DependentSizedArray                                  :: 116`
- `MemberPointer                                        :: 117`
- `Auto                                                 :: 118`
- `Elaborated                                           :: 119`
- `Pipe                                                 :: 120`
- `OCLImage1dRO                                         :: 121`
- `OCLImage1dArrayRO                                    :: 122`
- `OCLImage1dBufferRO                                   :: 123`
- `OCLImage2dRO                                         :: 124`
- `OCLImage2dArrayRO                                    :: 125`
- `OCLImage2dDepthRO                                    :: 126`
- `OCLImage2dArrayDepthRO                               :: 127`
- `OCLImage2dMSAARO                                     :: 128`
- `OCLImage2dArrayMSAARO                                :: 129`
- `OCLImage2dMSAADepthRO                                :: 130`
- `OCLImage2dArrayMSAADepthRO                           :: 131`
- `OCLImage3dRO                                         :: 132`
- `OCLImage1dWO                                         :: 133`
- `OCLImage1dArrayWO                                    :: 134`
- `OCLImage1dBufferWO                                   :: 135`
- `OCLImage2dWO                                         :: 136`
- `OCLImage2dArrayWO                                    :: 137`
- `OCLImage2dDepthWO                                    :: 138`
- `OCLImage2dArrayDepthWO                               :: 139`
- `OCLImage2dMSAAWO                                     :: 140`
- `OCLImage2dArrayMSAAWO                                :: 141`
- `OCLImage2dMSAADepthWO                                :: 142`
- `OCLImage2dArrayMSAADepthWO                           :: 143`
- `OCLImage3dWO                                         :: 144`
- `OCLImage1dRW                                         :: 145`
- `OCLImage1dArrayRW                                    :: 146`
- `OCLImage1dBufferRW                                   :: 147`
- `OCLImage2dRW                                         :: 148`
- `OCLImage2dArrayRW                                    :: 149`
- `OCLImage2dDepthRW                                    :: 150`
- `OCLImage2dArrayDepthRW                               :: 151`
- `OCLImage2dMSAARW                                     :: 152`
- `OCLImage2dArrayMSAARW                                :: 153`
- `OCLImage2dMSAADepthRW                                :: 154`
- `OCLImage2dArrayMSAADepthRW                           :: 155`
- `OCLImage3dRW                                         :: 156`
- `OCLSampler                                           :: 157`
- `OCLEvent                                             :: 158`
- `OCLQueue                                             :: 159`
- `OCLReserveID                                         :: 160`
- `ObjCObject                                           :: 161`
- `ObjCTypeParam                                        :: 162`
- `Attributed                                           :: 163`
- `OCLIntelSubgroupAVCMcePayload                        :: 164`
- `OCLIntelSubgroupAVCImePayload                        :: 165`
- `OCLIntelSubgroupAVCRefPayload                        :: 166`
- `OCLIntelSubgroupAVCSicPayload                        :: 167`
- `OCLIntelSubgroupAVCMceResult                         :: 168`
- `OCLIntelSubgroupAVCImeResult                         :: 169`
- `OCLIntelSubgroupAVCRefResult                         :: 170`
- `OCLIntelSubgroupAVCSicResult                         :: 171`
- `OCLIntelSubgroupAVCImeResultSingleReferenceStreamout :: 172`
- `OCLIntelSubgroupAVCImeResultDualReferenceStreamout   :: 173`
- `OCLIntelSubgroupAVCImeSingleReferenceStreamin        :: 174`
- `OCLIntelSubgroupAVCImeDualReferenceStreamin          :: 175`
- `OCLIntelSubgroupAVCImeResultSingleRefStreamout       :: 172`
- `OCLIntelSubgroupAVCImeResultDualRefStreamout         :: 173`
- `OCLIntelSubgroupAVCImeSingleRefStreamin              :: 174`
- `OCLIntelSubgroupAVCImeDualRefStreamin                :: 175`
- `ExtVector                                            :: 176`
- `Atomic                                               :: 177`
- `BTFTagAttributed                                     :: 178`
- `HLSLResource                                         :: 179`
- `FirstBuiltin                                         :: 2`
- `LastBuiltin                                          :: 40`

#### `CXCallingConv` (enum)

- `Default            :: 0`
- `C                  :: 1`
- `X86StdCall         :: 2`
- `X86FastCall        :: 3`
- `X86ThisCall        :: 4`
- `X86Pascal          :: 5`
- `AAPCS              :: 6`
- `AAPCS_VFP          :: 7`
- `X86RegCall         :: 8`
- `IntelOclBicc       :: 9`
- `Win64              :: 10`
- `X86_64Win64        :: 10`
- `X86_64SysV         :: 11`
- `X86VectorCall      :: 12`
- `Swift              :: 13`
- `PreserveMost       :: 14`
- `PreserveAll        :: 15`
- `AArch64VectorCall  :: 16`
- `SwiftAsync         :: 17`
- `AArch64SVEPCS      :: 18`
- `M68kRTD            :: 19`
- `PreserveNone       :: 20`
- `RISCVVectorCall    :: 21`
- `RISCVVLSCall_32    :: 22`
- `RISCVVLSCall_64    :: 23`
- `RISCVVLSCall_128   :: 24`
- `RISCVVLSCall_256   :: 25`
- `RISCVVLSCall_512   :: 26`
- `RISCVVLSCall_1024  :: 27`
- `RISCVVLSCall_2048  :: 28`
- `RISCVVLSCall_4096  :: 29`
- `RISCVVLSCall_8192  :: 30`
- `RISCVVLSCall_16384 :: 31`
- `RISCVVLSCall_32768 :: 32`
- `RISCVVLSCall_65536 :: 33`
- `Invalid            :: 100`
- `Unexposed          :: 200`

#### `CXType` (struct)

- `kind: CXTypeKind`
- `data: [2] *void`

#### `CXTemplateArgumentKind` (enum)

- `Null              :: 0`
- `Type              :: 1`
- `Declaration       :: 2`
- `NullPtr           :: 3`
- `Integral          :: 4`
- `Template          :: 5`
- `TemplateExpansion :: 6`
- `Expression        :: 7`
- `Pack              :: 8`
- `Invalid           :: 9`

#### `CXTypeNullabilityKind` (enum)

- `NonNull        :: 0`
- `Nullable       :: 1`
- `Unspecified    :: 2`
- `Invalid        :: 3`
- `NullableResult :: 4`

#### `CXTypeLayoutError` (enum)

- `Invalid          :: -1`
- `Incomplete       :: -2`
- `Dependent        :: -3`
- `NotConstantSize  :: -4`
- `InvalidFieldName :: -5`
- `Undeduced        :: -6`

#### `CXRefQualifierKind` (enum)

- `None   :: 0`
- `LValue :: 1`
- `RValue :: 2`

#### `CX_CXXAccessSpecifier` (enum)

- `CXXInvalidAccessSpecifier :: 0`
- `CXXPublic                 :: 1`
- `CXXProtected              :: 2`
- `CXXPrivate                :: 3`

#### `CX_StorageClass` (enum)

- `Invalid              :: 0`
- `None                 :: 1`
- `Extern               :: 2`
- `Static               :: 3`
- `PrivateExtern        :: 4`
- `OpenCLWorkGroupLocal :: 5`
- `Auto                 :: 6`
- `Register             :: 7`

#### `CX_BinaryOperatorKind` (enum)

- `Invalid   :: 0`
- `PtrMemD   :: 1`
- `PtrMemI   :: 2`
- `Mul       :: 3`
- `Div       :: 4`
- `Rem       :: 5`
- `Add       :: 6`
- `Sub       :: 7`
- `Shl       :: 8`
- `Shr       :: 9`
- `Cmp       :: 10`
- `LT        :: 11`
- `GT        :: 12`
- `LE        :: 13`
- `GE        :: 14`
- `EQ        :: 15`
- `NE        :: 16`
- `And       :: 17`
- `Xor       :: 18`
- `Or        :: 19`
- `LAnd      :: 20`
- `LOr       :: 21`
- `Assign    :: 22`
- `MulAssign :: 23`
- `DivAssign :: 24`
- `RemAssign :: 25`
- `AddAssign :: 26`
- `SubAssign :: 27`
- `ShlAssign :: 28`
- `ShrAssign :: 29`
- `AndAssign :: 30`
- `XorAssign :: 31`
- `OrAssign  :: 32`
- `Comma     :: 33`
- `LAST      :: 33`

#### `CXChildVisitResult` (enum)

- `Break    :: 0`
- `Continue :: 1`
- `Recurse  :: 2`

#### `CXPrintingPolicyProperty` (enum)

- `Indentation                           :: 0`
- `SuppressSpecifiers                    :: 1`
- `SuppressTagKeyword                    :: 2`
- `IncludeTagDefinition                  :: 3`
- `SuppressScope                         :: 4`
- `SuppressUnwrittenScope                :: 5`
- `SuppressInitializers                  :: 6`
- `ConstantArraySizeAsWritten            :: 7`
- `AnonymousTagLocations                 :: 8`
- `SuppressStrongLifetime                :: 9`
- `SuppressLifetimeQualifiers            :: 10`
- `SuppressTemplateArgsInCXXConstructors :: 11`
- `Bool                                  :: 12`
- `Restrict                              :: 13`
- `Alignof                               :: 14`
- `UnderscoreAlignof                     :: 15`
- `UseVoidForZeroParams                  :: 16`
- `TerseOutput                           :: 17`
- `PolishForDeclaration                  :: 18`
- `Half                                  :: 19`
- `MSWChar                               :: 20`
- `IncludeNewlines                       :: 21`
- `MSVCFormatting                        :: 22`
- `ConstantsAsWritten                    :: 23`
- `SuppressImplicitBase                  :: 24`
- `FullyQualifiedName                    :: 25`
- `LastProperty                          :: 25`

#### `CXObjCPropertyAttrKind` (enum)

- `noattr            :: 0`
- `readonly          :: 1`
- `getter            :: 2`
- `assign            :: 4`
- `readwrite         :: 8`
- `retain            :: 16`
- `copy              :: 32`
- `nonatomic         :: 64`
- `setter            :: 128`
- `atomic            :: 256`
- `weak              :: 512`
- `strong            :: 1024`
- `unsafe_unretained :: 2048`
- `class             :: 4096`

#### `CXObjCDeclQualifierKind` (enum)

- `None   :: 0`
- `In     :: 1`
- `Inout  :: 2`
- `Out    :: 4`
- `Bycopy :: 8`
- `Byref  :: 16`
- `Oneway :: 32`

#### `CXNameRefFlags` (enum)

- `WantQualifier    :: 1`
- `WantTemplateArgs :: 2`
- `WantSinglePiece  :: 4`

#### `CXTokenKind` (enum)

- `Punctuation :: 0`
- `Keyword     :: 1`
- `Identifier  :: 2`
- `Literal     :: 3`
- `Comment     :: 4`

#### `CXToken` (struct)

- `int_data: [4] u32`
- `ptr_data: *void`

#### `CXCompletionResult` (struct)

- `CursorKind:       CXCursorKind`
- `CompletionString: CXCompletionString`

#### `CXCompletionChunkKind` (enum)

- `Optional         :: 0`
- `TypedText        :: 1`
- `Text             :: 2`
- `Placeholder      :: 3`
- `Informative      :: 4`
- `CurrentParameter :: 5`
- `LeftParen        :: 6`
- `RightParen       :: 7`
- `LeftBracket      :: 8`
- `RightBracket     :: 9`
- `LeftBrace        :: 10`
- `RightBrace       :: 11`
- `LeftAngle        :: 12`
- `RightAngle       :: 13`
- `Comma            :: 14`
- `ResultType       :: 15`
- `Colon            :: 16`
- `SemiColon        :: 17`
- `Equal            :: 18`
- `HorizontalSpace  :: 19`
- `VerticalSpace    :: 20`

#### `CXCodeCompleteResults` (struct)

- `Results:    *CXCompletionResult`
- `NumResults: u32`

#### `CXCodeComplete_Flags` (enum_flags)

- `IncludeMacros                :: 0x1`
- `IncludeCodePatterns          :: 0x2`
- `IncludeBriefComments         :: 0x4`
- `SkipPreamble                 :: 0x8`
- `IncludeCompletionsWithFixIts :: 0x10`

#### `CXCompletionContext` (enum_flags)

- `Unexposed           :: 0x0`
- `AnyType             :: 0x1`
- `AnyValue            :: 0x2`
- `ObjCObjectValue     :: 0x4`
- `ObjCSelectorValue   :: 0x8`
- `CXXClassTypeValue   :: 0x10`
- `DotMemberAccess     :: 0x20`
- `ArrowMemberAccess   :: 0x40`
- `ObjCPropertyAccess  :: 0x80`
- `EnumTag             :: 0x100`
- `UnionTag            :: 0x200`
- `StructTag           :: 0x400`
- `ClassTag            :: 0x800`
- `Namespace           :: 0x1000`
- `NestedNameSpecifier :: 0x2000`
- `ObjCInterface       :: 0x4000`
- `ObjCProtocol        :: 0x8000`
- `ObjCCategory        :: 0x10000`
- `ObjCInstanceMessage :: 0x20000`
- `ObjCClassMessage    :: 0x40000`
- `ObjCSelectorName    :: 0x80000`
- `MacroName           :: 0x100000`
- `NaturalLanguage     :: 0x200000`
- `IncludedFile        :: 0x400000`
- `Unknown             :: 0x7fffff`

#### `CXEvalResultKind` (enum)

- `Int            :: 1`
- `Float          :: 2`
- `ObjCStrLiteral :: 3`
- `StrLiteral     :: 4`
- `CFStr          :: 5`
- `Other          :: 6`
- `UnExposed      :: 0`

#### `CXVisitorResult` (enum)

- `Break    :: 0`
- `Continue :: 1`

#### `CXCursorAndRangeVisitor` (struct)

- `_context: *void`
- `visit:    #type (_context: *void, unknown0: CXCursor, unknown1: CXSourceRange) -> CXVisitorResult #c_call`

#### `CXResult` (enum)

- `Success    :: 0`
- `Invalid    :: 1`
- `VisitBreak :: 2`

#### `CXIdxLoc` (struct)

- `ptr_data: [2] *void`
- `int_data: u32`

#### `CXIdxIncludedFileInfo` (struct)

- `hashLoc:        CXIdxLoc`
- `filename:       *u8`
- `file:           CXFile`
- `isImport:       s32`
- `isAngled:       s32`
- `isModuleImport: s32`

#### `CXIdxImportedASTFileInfo` (struct)

- `file:       CXFile`
- `module:     CXModule`
- `loc:        CXIdxLoc`
- `isImplicit: s32`

#### `CXIdxEntityKind` (enum)

- `Unexposed             :: 0`
- `Typedef               :: 1`
- `Function              :: 2`
- `Variable              :: 3`
- `Field                 :: 4`
- `EnumConstant          :: 5`
- `ObjCClass             :: 6`
- `ObjCProtocol          :: 7`
- `ObjCCategory          :: 8`
- `ObjCInstanceMethod    :: 9`
- `ObjCClassMethod       :: 10`
- `ObjCProperty          :: 11`
- `ObjCIvar              :: 12`
- `Enum                  :: 13`
- `Struct                :: 14`
- `Union                 :: 15`
- `CXXClass              :: 16`
- `CXXNamespace          :: 17`
- `CXXNamespaceAlias     :: 18`
- `CXXStaticVariable     :: 19`
- `CXXStaticMethod       :: 20`
- `CXXInstanceMethod     :: 21`
- `CXXConstructor        :: 22`
- `CXXDestructor         :: 23`
- `CXXConversionFunction :: 24`
- `CXXTypeAlias          :: 25`
- `CXXInterface          :: 26`
- `CXXConcept            :: 27`

#### `CXIdxEntityLanguage` (enum)

- `None  :: 0`
- `C     :: 1`
- `ObjC  :: 2`
- `CXX   :: 3`
- `Swift :: 4`

#### `CXIdxEntityCXXTemplateKind` (enum)

- `NonTemplate                   :: 0`
- `Template                      :: 1`
- `TemplatePartialSpecialization :: 2`
- `TemplateSpecialization        :: 3`

#### `CXIdxAttrKind` (enum)

- `Unexposed          :: 0`
- `IBAction           :: 1`
- `IBOutlet           :: 2`
- `IBOutletCollection :: 3`

#### `CXIdxAttrInfo` (struct)

- `kind:   CXIdxAttrKind`
- `cursor: CXCursor`
- `loc:    CXIdxLoc`

#### `CXIdxEntityInfo` (struct)

- `kind:          CXIdxEntityKind`
- `templateKind:  CXIdxEntityCXXTemplateKind`
- `lang:          CXIdxEntityLanguage`
- `name:          *u8`
- `USR:           *u8`
- `cursor:        CXCursor`
- `attributes:    **CXIdxAttrInfo`
- `numAttributes: u32`

#### `CXIdxContainerInfo` (struct)

- `cursor: CXCursor`

#### `CXIdxIBOutletCollectionAttrInfo` (struct)

- `attrInfo:    *CXIdxAttrInfo`
- `objcClass:   *CXIdxEntityInfo`
- `classCursor: CXCursor`
- `classLoc:    CXIdxLoc`

#### `CXIdxDeclInfoFlags` (enum)

- `CXIdxDeclFlag_Skipped :: 1`

#### `CXIdxDeclInfo` (struct)

- `entityInfo:        *CXIdxEntityInfo`
- `cursor:            CXCursor`
- `loc:               CXIdxLoc`
- `semanticContainer: *CXIdxContainerInfo`
- `lexicalContainer:  *CXIdxContainerInfo`
- `isRedeclaration:   s32`
- `isDefinition:      s32`
- `isContainer:       s32`
- `declAsContainer:   *CXIdxContainerInfo`
- `isImplicit:        s32`
- `attributes:        **CXIdxAttrInfo`
- `numAttributes:     u32`
- `flags:             u32`

#### `CXIdxObjCContainerKind` (enum)

- `ForwardRef     :: 0`
- `Interface      :: 1`
- `Implementation :: 2`

#### `CXIdxObjCContainerDeclInfo` (struct)

- `declInfo: *CXIdxDeclInfo`
- `kind:     CXIdxObjCContainerKind`

#### `CXIdxBaseClassInfo` (struct)

- `base:   *CXIdxEntityInfo`
- `cursor: CXCursor`
- `loc:    CXIdxLoc`

#### `CXIdxObjCProtocolRefInfo` (struct)

- `protocol: *CXIdxEntityInfo`
- `cursor:   CXCursor`
- `loc:      CXIdxLoc`

#### `CXIdxObjCProtocolRefListInfo` (struct)

- `protocols:    **CXIdxObjCProtocolRefInfo`
- `numProtocols: u32`

#### `CXIdxObjCInterfaceDeclInfo` (struct)

- `containerInfo: *CXIdxObjCContainerDeclInfo`
- `superInfo:     *CXIdxBaseClassInfo`
- `protocols:     *CXIdxObjCProtocolRefListInfo`

#### `CXIdxObjCCategoryDeclInfo` (struct)

- `containerInfo: *CXIdxObjCContainerDeclInfo`
- `objcClass:     *CXIdxEntityInfo`
- `classCursor:   CXCursor`
- `classLoc:      CXIdxLoc`
- `protocols:     *CXIdxObjCProtocolRefListInfo`

#### `CXIdxObjCPropertyDeclInfo` (struct)

- `declInfo: *CXIdxDeclInfo`
- `getter:   *CXIdxEntityInfo`
- `setter:   *CXIdxEntityInfo`

#### `CXIdxCXXClassDeclInfo` (struct)

- `declInfo: *CXIdxDeclInfo`
- `bases:    **CXIdxBaseClassInfo`
- `numBases: u32`

#### `CXIdxEntityRefKind` (enum)

- `Direct   :: 1`
- `Implicit :: 2`

#### `CXSymbolRole` (enum)

- `None        :: 0`
- `Declaration :: 1`
- `Definition  :: 2`
- `Reference   :: 4`
- `Read        :: 8`
- `Write       :: 16`
- `Call        :: 32`
- `Dynamic     :: 64`
- `AddressOf   :: 128`
- `Implicit    :: 256`

#### `CXIdxEntityRefInfo` (struct)

- `kind:             CXIdxEntityRefKind`
- `cursor:           CXCursor`
- `loc:              CXIdxLoc`
- `referencedEntity: *CXIdxEntityInfo`
- `parentEntity:     *CXIdxEntityInfo`
- `container:        *CXIdxContainerInfo`
- `role:             CXSymbolRole`

#### `IndexerCallbacks` (struct)

- `abortQuery:             #type (client_data: CXClientData, reserved: *void) -> s32 #c_call`
- `diagnostic:             #type (client_data: CXClientData, unknown0: CXDiagnosticSet, reserved: *void) -> void #c_call`
- `enteredMainFile:        #type (client_data: CXClientData, mainFile: CXFile, reserved: *void) -> CXIdxClientFile #c_call`
- `ppIncludedFile:         #type (client_data: CXClientData, unknown0: *CXIdxIncludedFileInfo) -> CXIdxClientFile #c_call`
- `importedASTFile:        #type (client_data: CXClientData, unknown0: *CXIdxImportedASTFileInfo) -> CXIdxClientASTFile #c_call`
- `startedTranslationUnit: #type (client_data: CXClientData, reserved: *void) -> CXIdxClientContainer #c_call`
- `indexDeclaration:       #type (client_data: CXClientData, unknown0: *CXIdxDeclInfo) -> void #c_call`
- `indexEntityReference:   #type (client_data: CXClientData, unknown0: *CXIdxEntityRefInfo) -> void #c_call`

#### `CXIndexOptFlags` (enum)

- `None                                :: 0`
- `SuppressRedundantRefs               :: 1`
- `IndexFunctionLocalSymbols           :: 2`
- `IndexImplicitTemplateInstantiations :: 4`
- `SuppressWarnings                    :: 8`
- `SkipParsedBodiesInSession           :: 16`

#### `CXBinaryOperatorKind` (enum)

- `Invalid   :: 0`
- `PtrMemD   :: 1`
- `PtrMemI   :: 2`
- `Mul       :: 3`
- `Div       :: 4`
- `Rem       :: 5`
- `Add       :: 6`
- `Sub       :: 7`
- `Shl       :: 8`
- `Shr       :: 9`
- `Cmp       :: 10`
- `LT        :: 11`
- `GT        :: 12`
- `LE        :: 13`
- `GE        :: 14`
- `EQ        :: 15`
- `NE        :: 16`
- `And       :: 17`
- `Xor       :: 18`
- `Or        :: 19`
- `LAnd      :: 20`
- `LOr       :: 21`
- `Assign    :: 22`
- `MulAssign :: 23`
- `DivAssign :: 24`
- `RemAssign :: 25`
- `AddAssign :: 26`
- `SubAssign :: 27`
- `ShlAssign :: 28`
- `ShrAssign :: 29`
- `AndAssign :: 30`
- `XorAssign :: 31`
- `OrAssign  :: 32`
- `Comma     :: 33`
- `Last      :: 33`

#### `CXUnaryOperatorKind` (enum)

- `Invalid   :: 0`
- `PostInc   :: 1`
- `PostDec   :: 2`
- `PreInc    :: 3`
- `PreDec    :: 4`
- `AddrOf    :: 5`
- `Deref     :: 6`
- `Plus      :: 7`
- `Minus     :: 8`
- `Not       :: 9`
- `LNot      :: 10`
- `Real      :: 11`
- `Imag      :: 12`
- `Extension :: 13`
- `Coawait   :: 14`

### Windows-specific type declarations

#### `CXErrorCode` (enum)

- `Success          :: 0`
- `Failure          :: 1`
- `Crashed          :: 2`
- `InvalidArguments :: 3`
- `ASTReadError     :: 4`

#### `CXString` (struct)

- `data:          *void`
- `private_flags: u32`

#### `CXStringSet` (struct)

- `Strings: *CXString`
- `Count:   u32`

#### `CXFileUniqueID` (struct)

- `data: [3] u64`

#### `CXSourceLocation` (struct)

- `ptr_data: [2] *void`
- `int_data: u32`

#### `CXSourceRange` (struct)

- `ptr_data:       [2] *void`
- `begin_int_data: u32`
- `end_int_data:   u32`

#### `CXSourceRangeList` (struct)

- `count:  u32`
- `ranges: *CXSourceRange`

#### `CXDiagnosticSeverity` (enum)

- `Ignored :: 0`
- `Note    :: 1`
- `Warning :: 2`
- `Error   :: 3`
- `Fatal   :: 4`

#### `CXLoadDiag_Error` (enum)

- `None        :: 0`
- `Unknown     :: 1`
- `CannotLoad  :: 2`
- `InvalidFile :: 3`

#### `CXDiagnosticDisplayOptions` (enum)

- `DisplaySourceLocation :: 1`
- `DisplayColumn         :: 2`
- `DisplaySourceRanges   :: 4`
- `DisplayOption         :: 8`
- `DisplayCategoryId     :: 16`
- `DisplayCategoryName   :: 32`

#### `CXUnsavedFile` (struct)

- `Filename: *u8`
- `Contents: *u8`
- `Length:   u32`

#### `CXAvailabilityKind` (enum)

- `Available     :: 0`
- `Deprecated    :: 1`
- `NotAvailable  :: 2`
- `NotAccessible :: 3`

#### `CXVersion` (struct)

- `Major:    s32`
- `Minor:    s32`
- `Subminor: s32`

#### `CXCursor_ExceptionSpecificationKind` (enum)

- `None             :: 0`
- `DynamicNone      :: 1`
- `Dynamic          :: 2`
- `MSAny            :: 3`
- `BasicNoexcept    :: 4`
- `ComputedNoexcept :: 5`
- `Unevaluated      :: 6`
- `Uninstantiated   :: 7`
- `Unparsed         :: 8`
- `NoThrow          :: 9`

#### `CXChoice` (enum)

- `Default  :: 0`
- `Enabled  :: 1`
- `Disabled :: 2`

#### `CXGlobalOptFlags` (enum)

- `None                                :: 0`
- `ThreadBackgroundPriorityForIndexing :: 1`
- `ThreadBackgroundPriorityForEditing  :: 2`
- `ThreadBackgroundPriorityForAll      :: 3`

#### `CXIndexOptions` (struct)

- `Size:                                u32`
- `ThreadBackgroundPriorityForIndexing: u8`
- `ThreadBackgroundPriorityForEditing:  u8`
- `__bitfield:                          u16`
- `PreambleStoragePath:                 *u8`
- `InvocationEmissionPath:              *u8`

The three named boolean options are bit views, not additional aggregate fields: `set_ExcludeDeclarationsFromPCH`/`get_ExcludeDeclarationsFromPCH` use bit 0 of `__bitfield`, `set_DisplayDiagnostics`/`get_DisplayDiagnostics` use bit 1, and `set_StorePreamblesInMemory`/`get_StorePreamblesInMemory` use bit 2. The remaining bits are reserved by the associated C interface and have no named Jai accessor. The Jai record's public storage is `Size`, the two one-byte priorities, a 16-bit bitfield word, then the two path pointers. With 64-bit pointers this yields the native 24-byte record and pointer offsets 8 and 16; with 32-bit pointers the final pointer fields follow the same declared order with target-appropriate alignment. Fields have no explicit Jai initializer; ordinary omitted-field initialization applies.

#### `CXTranslationUnit_Flags` (enum_flags)

- `None                                 :: 0x0`
- `DetailedPreprocessingRecord          :: 0x1`
- `Incomplete                           :: 0x2`
- `PrecompiledPreamble                  :: 0x4`
- `CacheCompletionResults               :: 0x8`
- `ForSerialization                     :: 0x10`
- `CXXChainedPCH                        :: 0x20`
- `SkipFunctionBodies                   :: 0x40`
- `IncludeBriefCommentsInCodeCompletion :: 0x80`
- `CreatePreambleOnFirstParse           :: 0x100`
- `KeepGoing                            :: 0x200`
- `SingleFileParse                      :: 0x400`
- `LimitSkipFunctionBodiesToPreamble    :: 0x800`
- `IncludeAttributedTypes               :: 0x1000`
- `VisitImplicitAttributes              :: 0x2000`
- `IgnoreNonErrorsFromIncludedFiles     :: 0x4000`
- `RetainExcludedConditionalBlocks      :: 0x8000`

#### `CXSaveTranslationUnit_Flags` (enum_flags)

- `CXSaveTranslationUnit_None :: 0x0`

#### `CXSaveError` (enum)

- `None              :: 0`
- `Unknown           :: 1`
- `TranslationErrors :: 2`
- `InvalidTU         :: 3`

#### `CXReparse_Flags` (enum_flags)

- `CXReparse_None :: 0x0`

#### `CXTUResourceUsageKind` (enum)

- `AST                                :: 1`
- `Identifiers                        :: 2`
- `Selectors                          :: 3`
- `GlobalCompletionResults            :: 4`
- `SourceManagerContentCache          :: 5`
- `AST_SideTables                     :: 6`
- `SourceManager_Membuffer_Malloc     :: 7`
- `SourceManager_Membuffer_MMap       :: 8`
- `ExternalASTSource_Membuffer_Malloc :: 9`
- `ExternalASTSource_Membuffer_MMap   :: 10`
- `Preprocessor                       :: 11`
- `PreprocessingRecord                :: 12`
- `SourceManager_DataStructures       :: 13`
- `Preprocessor_HeaderSearch          :: 14`
- `MEMORY_IN_BYTES_BEGIN              :: 1`
- `MEMORY_IN_BYTES_END                :: 14`
- `First                              :: 1`
- `Last                               :: 14`

#### `CXTUResourceUsageEntry` (struct)

- `kind:   CXTUResourceUsageKind`
- `amount: u32`

#### `CXTUResourceUsage` (struct)

- `data:       *void`
- `numEntries: u32`
- `entries:    *CXTUResourceUsageEntry`

#### `CXCursorKind` (enum)

- `UnexposedDecl                                    :: 1`
- `StructDecl                                       :: 2`
- `UnionDecl                                        :: 3`
- `ClassDecl                                        :: 4`
- `EnumDecl                                         :: 5`
- `FieldDecl                                        :: 6`
- `EnumConstantDecl                                 :: 7`
- `FunctionDecl                                     :: 8`
- `VarDecl                                          :: 9`
- `ParmDecl                                         :: 10`
- `ObjCInterfaceDecl                                :: 11`
- `ObjCCategoryDecl                                 :: 12`
- `ObjCProtocolDecl                                 :: 13`
- `ObjCPropertyDecl                                 :: 14`
- `ObjCIvarDecl                                     :: 15`
- `ObjCInstanceMethodDecl                           :: 16`
- `ObjCClassMethodDecl                              :: 17`
- `ObjCImplementationDecl                           :: 18`
- `ObjCCategoryImplDecl                             :: 19`
- `TypedefDecl                                      :: 20`
- `CXXMethod                                        :: 21`
- `Namespace                                        :: 22`
- `LinkageSpec                                      :: 23`
- `Constructor                                      :: 24`
- `Destructor                                       :: 25`
- `ConversionFunction                               :: 26`
- `TemplateTypeParameter                            :: 27`
- `NonTypeTemplateParameter                         :: 28`
- `TemplateTemplateParameter                        :: 29`
- `FunctionTemplate                                 :: 30`
- `ClassTemplate                                    :: 31`
- `ClassTemplatePartialSpecialization               :: 32`
- `NamespaceAlias                                   :: 33`
- `UsingDirective                                   :: 34`
- `UsingDeclaration                                 :: 35`
- `TypeAliasDecl                                    :: 36`
- `ObjCSynthesizeDecl                               :: 37`
- `ObjCDynamicDecl                                  :: 38`
- `CXXAccessSpecifier                               :: 39`
- `OverloadCandidate                                :: 700`
- `ConceptDecl                                      :: 604`
- `FriendDecl                                       :: 603`
- `ObjCSuperClassRef                                :: 40`
- `ObjCProtocolRef                                  :: 41`
- `ObjCClassRef                                     :: 42`
- `TypeRef                                          :: 43`
- `CXXBaseSpecifier                                 :: 44`
- `TemplateRef                                      :: 45`
- `NamespaceRef                                     :: 46`
- `MemberRef                                        :: 47`
- `LabelRef                                         :: 48`
- `OverloadedDeclRef                                :: 49`
- `VariableRef                                      :: 50`
- `StaticAssert                                     :: 602`
- `TypeAliasTemplateDecl                            :: 601`
- `InvalidFile                                      :: 70`
- `NoDeclFound                                      :: 71`
- `NotImplemented                                   :: 72`
- `InvalidCode                                      :: 73`
- `ModuleImportDecl                                 :: 600`
- `InclusionDirective                               :: 503`
- `UnexposedExpr                                    :: 100`
- `DeclRefExpr                                      :: 101`
- `MemberRefExpr                                    :: 102`
- `CallExpr                                         :: 103`
- `ObjCMessageExpr                                  :: 104`
- `BlockExpr                                        :: 105`
- `IntegerLiteral                                   :: 106`
- `FloatingLiteral                                  :: 107`
- `ImaginaryLiteral                                 :: 108`
- `StringLiteral                                    :: 109`
- `CharacterLiteral                                 :: 110`
- `ParenExpr                                        :: 111`
- `UnaryOperator                                    :: 112`
- `ArraySubscriptExpr                               :: 113`
- `BinaryOperator                                   :: 114`
- `CompoundAssignOperator                           :: 115`
- `ConditionalOperator                              :: 116`
- `CStyleCastExpr                                   :: 117`
- `CompoundLiteralExpr                              :: 118`
- `InitListExpr                                     :: 119`
- `AddrLabelExpr                                    :: 120`
- `StmtExpr                                         :: 121`
- `GenericSelectionExpr                             :: 122`
- `GNUNullExpr                                      :: 123`
- `CXXStaticCastExpr                                :: 124`
- `CXXDynamicCastExpr                               :: 125`
- `CXXReinterpretCastExpr                           :: 126`
- `CXXConstCastExpr                                 :: 127`
- `CXXFunctionalCastExpr                            :: 128`
- `CXXTypeidExpr                                    :: 129`
- `CXXBoolLiteralExpr                               :: 130`
- `CXXNullPtrLiteralExpr                            :: 131`
- `CXXThisExpr                                      :: 132`
- `CXXThrowExpr                                     :: 133`
- `CXXNewExpr                                       :: 134`
- `CXXDeleteExpr                                    :: 135`
- `UnaryExpr                                        :: 136`
- `ObjCStringLiteral                                :: 137`
- `ObjCEncodeExpr                                   :: 138`
- `ObjCSelectorExpr                                 :: 139`
- `ObjCProtocolExpr                                 :: 140`
- `ObjCBridgedCastExpr                              :: 141`
- `PackExpansionExpr                                :: 142`
- `SizeOfPackExpr                                   :: 143`
- `LambdaExpr                                       :: 144`
- `ObjCBoolLiteralExpr                              :: 145`
- `ObjCSelfExpr                                     :: 146`
- `ArraySectionExpr                                 :: 147`
- `ObjCAvailabilityCheckExpr                        :: 148`
- `FixedPointLiteral                                :: 149`
- `OMPArrayShapingExpr                              :: 150`
- `OMPIteratorExpr                                  :: 151`
- `CXXAddrspaceCastExpr                             :: 152`
- `ConceptSpecializationExpr                        :: 153`
- `RequiresExpr                                     :: 154`
- `CXXParenListInitExpr                             :: 155`
- `PackIndexingExpr                                 :: 156`
- `MacroInstantiation                               :: 502`
- `MacroExpansion                                   :: 502`
- `UnexposedStmt                                    :: 200`
- `LabelStmt                                        :: 201`
- `CompoundStmt                                     :: 202`
- `CaseStmt                                         :: 203`
- `DefaultStmt                                      :: 204`
- `IfStmt                                           :: 205`
- `SwitchStmt                                       :: 206`
- `WhileStmt                                        :: 207`
- `DoStmt                                           :: 208`
- `ForStmt                                          :: 209`
- `GotoStmt                                         :: 210`
- `IndirectGotoStmt                                 :: 211`
- `ContinueStmt                                     :: 212`
- `BreakStmt                                        :: 213`
- `ReturnStmt                                       :: 214`
- `GCCAsmStmt                                       :: 215`
- `AsmStmt                                          :: 215`
- `ObjCAtTryStmt                                    :: 216`
- `ObjCAtCatchStmt                                  :: 217`
- `ObjCAtFinallyStmt                                :: 218`
- `ObjCAtThrowStmt                                  :: 219`
- `ObjCAtSynchronizedStmt                           :: 220`
- `ObjCAutoreleasePoolStmt                          :: 221`
- `ObjCForCollectionStmt                            :: 222`
- `CXXCatchStmt                                     :: 223`
- `CXXTryStmt                                       :: 224`
- `CXXForRangeStmt                                  :: 225`
- `SEHTryStmt                                       :: 226`
- `SEHExceptStmt                                    :: 227`
- `SEHFinallyStmt                                   :: 228`
- `MSAsmStmt                                        :: 229`
- `NullStmt                                         :: 230`
- `DeclStmt                                         :: 231`
- `OMPParallelDirective                             :: 232`
- `OMPSimdDirective                                 :: 233`
- `OMPForDirective                                  :: 234`
- `OMPSectionsDirective                             :: 235`
- `OMPSectionDirective                              :: 236`
- `OMPSingleDirective                               :: 237`
- `OMPParallelForDirective                          :: 238`
- `OMPParallelSectionsDirective                     :: 239`
- `OMPTaskDirective                                 :: 240`
- `OMPMasterDirective                               :: 241`
- `OMPCriticalDirective                             :: 242`
- `OMPTaskyieldDirective                            :: 243`
- `OMPBarrierDirective                              :: 244`
- `OMPTaskwaitDirective                             :: 245`
- `OMPFlushDirective                                :: 246`
- `SEHLeaveStmt                                     :: 247`
- `OMPOrderedDirective                              :: 248`
- `OMPAtomicDirective                               :: 249`
- `OMPForSimdDirective                              :: 250`
- `OMPParallelForSimdDirective                      :: 251`
- `OMPTargetDirective                               :: 252`
- `OMPTeamsDirective                                :: 253`
- `OMPTaskgroupDirective                            :: 254`
- `OMPCancellationPointDirective                    :: 255`
- `OMPCancelDirective                               :: 256`
- `OMPTargetDataDirective                           :: 257`
- `OMPTaskLoopDirective                             :: 258`
- `OMPTaskLoopSimdDirective                         :: 259`
- `OMPDistributeDirective                           :: 260`
- `OMPTargetEnterDataDirective                      :: 261`
- `OMPTargetExitDataDirective                       :: 262`
- `OMPTargetParallelDirective                       :: 263`
- `OMPTargetParallelForDirective                    :: 264`
- `OMPTargetUpdateDirective                         :: 265`
- `OMPDistributeParallelForDirective                :: 266`
- `OMPDistributeParallelForSimdDirective            :: 267`
- `OMPDistributeSimdDirective                       :: 268`
- `OMPTargetParallelForSimdDirective                :: 269`
- `OMPTargetSimdDirective                           :: 270`
- `OMPTeamsDistributeDirective                      :: 271`
- `OMPTeamsDistributeSimdDirective                  :: 272`
- `OMPTeamsDistributeParallelForSimdDirective       :: 273`
- `OMPTeamsDistributeParallelForDirective           :: 274`
- `OMPTargetTeamsDirective                          :: 275`
- `OMPTargetTeamsDistributeDirective                :: 276`
- `OMPTargetTeamsDistributeParallelForDirective     :: 277`
- `OMPTargetTeamsDistributeParallelForSimdDirective :: 278`
- `OMPTargetTeamsDistributeSimdDirective            :: 279`
- `BuiltinBitCastExpr                               :: 280`
- `OMPMasterTaskLoopDirective                       :: 281`
- `OMPParallelMasterTaskLoopDirective               :: 282`
- `OMPMasterTaskLoopSimdDirective                   :: 283`
- `OMPParallelMasterTaskLoopSimdDirective           :: 284`
- `OMPParallelMasterDirective                       :: 285`
- `OMPDepobjDirective                               :: 286`
- `OMPScanDirective                                 :: 287`
- `OMPTileDirective                                 :: 288`
- `OMPCanonicalLoop                                 :: 289`
- `OMPInteropDirective                              :: 290`
- `OMPDispatchDirective                             :: 291`
- `OMPMaskedDirective                               :: 292`
- `OMPUnrollDirective                               :: 293`
- `OMPMetaDirective                                 :: 294`
- `OMPGenericLoopDirective                          :: 295`
- `OMPTeamsGenericLoopDirective                     :: 296`
- `OMPTargetTeamsGenericLoopDirective               :: 297`
- `OMPParallelGenericLoopDirective                  :: 298`
- `OMPTargetParallelGenericLoopDirective            :: 299`
- `OMPParallelMaskedDirective                       :: 300`
- `OMPMaskedTaskLoopDirective                       :: 301`
- `OMPMaskedTaskLoopSimdDirective                   :: 302`
- `OMPParallelMaskedTaskLoopDirective               :: 303`
- `OMPParallelMaskedTaskLoopSimdDirective           :: 304`
- `OMPErrorDirective                                :: 305`
- `OMPScopeDirective                                :: 306`
- `OMPReverseDirective                              :: 307`
- `OMPInterchangeDirective                          :: 308`
- `OMPAssumeDirective                               :: 309`
- `OMPStripeDirective                               :: 310`
- `OpenACCComputeConstruct                          :: 320`
- `OpenACCLoopConstruct                             :: 321`
- `OpenACCCombinedConstruct                         :: 322`
- `OpenACCDataConstruct                             :: 323`
- `OpenACCEnterDataConstruct                        :: 324`
- `OpenACCExitDataConstruct                         :: 325`
- `OpenACCHostDataConstruct                         :: 326`
- `OpenACCWaitConstruct                             :: 327`
- `OpenACCInitConstruct                             :: 328`
- `OpenACCShutdownConstruct                         :: 329`
- `OpenACCSetConstruct                              :: 330`
- `OpenACCUpdateConstruct                           :: 331`
- `OpenACCAtomicConstruct                           :: 332`
- `OpenACCCacheConstruct                            :: 333`
- `MacroDefinition                                  :: 501`
- `TranslationUnit                                  :: 350`
- `PreprocessingDirective                           :: 500`
- `UnexposedAttr                                    :: 400`
- `IBActionAttr                                     :: 401`
- `IBOutletAttr                                     :: 402`
- `IBOutletCollectionAttr                           :: 403`
- `CXXFinalAttr                                     :: 404`
- `CXXOverrideAttr                                  :: 405`
- `AnnotateAttr                                     :: 406`
- `AsmLabelAttr                                     :: 407`
- `PackedAttr                                       :: 408`
- `PureAttr                                         :: 409`
- `ConstAttr                                        :: 410`
- `NoDuplicateAttr                                  :: 411`
- `CUDAConstantAttr                                 :: 412`
- `CUDADeviceAttr                                   :: 413`
- `CUDAGlobalAttr                                   :: 414`
- `CUDAHostAttr                                     :: 415`
- `CUDASharedAttr                                   :: 416`
- `VisibilityAttr                                   :: 417`
- `DLLExport                                        :: 418`
- `DLLImport                                        :: 419`
- `NSReturnsRetained                                :: 420`
- `NSReturnsNotRetained                             :: 421`
- `NSReturnsAutoreleased                            :: 422`
- `NSConsumesSelf                                   :: 423`
- `NSConsumed                                       :: 424`
- `ObjCException                                    :: 425`
- `ObjCNSObject                                     :: 426`
- `ObjCIndependentClass                             :: 427`
- `ObjCPreciseLifetime                              :: 428`
- `ObjCReturnsInnerPointer                          :: 429`
- `ObjCRequiresSuper                                :: 430`
- `ObjCRootClass                                    :: 431`
- `ObjCSubclassingRestricted                        :: 432`
- `ObjCExplicitProtocolImpl                         :: 433`
- `ObjCDesignatedInitializer                        :: 434`
- `ObjCRuntimeVisible                               :: 435`
- `ObjCBoxable                                      :: 436`
- `FlagEnum                                         :: 437`
- `ConvergentAttr                                   :: 438`
- `WarnUnusedAttr                                   :: 439`
- `WarnUnusedResultAttr                             :: 440`
- `AlignedAttr                                      :: 441`
- `FirstDecl                                        :: 1`
- `LastDecl                                         :: 39`
- `LastExtraDecl                                    :: 604`
- `FirstExtraDecl                                   :: 600`
- `FirstRef                                         :: 40`
- `LastRef                                          :: 50`
- `FirstInvalid                                     :: 70`
- `LastInvalid                                      :: 73`
- `FirstExpr                                        :: 100`
- `LastPreprocessing                                :: 503`
- `FirstPreprocessing                               :: 500`
- `LastExpr                                         :: 156`
- `FirstStmt                                        :: 200`
- `LastStmt                                         :: 333`
- `FirstAttr                                        :: 400`
- `LastAttr                                         :: 441`

#### `CXCursor` (struct)

- `kind:  CXCursorKind`
- `xdata: s32`
- `data:  [3] *void`

#### `CXLinkageKind` (enum)

- `Invalid        :: 0`
- `NoLinkage      :: 1`
- `Internal       :: 2`
- `UniqueExternal :: 3`
- `External       :: 4`

#### `CXVisibilityKind` (enum)

- `Invalid   :: 0`
- `Hidden    :: 1`
- `Protected :: 2`
- `Default   :: 3`

#### `CXPlatformAvailability` (struct)

- `Platform:    CXString`
- `Introduced:  CXVersion`
- `Deprecated:  CXVersion`
- `Obsoleted:   CXVersion`
- `Unavailable: s32`
- `Message:     CXString`

#### `CXLanguageKind` (enum)

- `Invalid   :: 0`
- `C         :: 1`
- `ObjC      :: 2`
- `CPlusPlus :: 3`

#### `CXTLSKind` (enum)

- `None    :: 0`
- `Dynamic :: 1`
- `Static  :: 2`

#### `CXTypeKind` (enum)

- `Invalid                                              :: 0`
- `Unexposed                                            :: 1`
- `Void                                                 :: 2`
- `Bool                                                 :: 3`
- `Char_U                                               :: 4`
- `UChar                                                :: 5`
- `Char16                                               :: 6`
- `Char32                                               :: 7`
- `UShort                                               :: 8`
- `UInt                                                 :: 9`
- `ULong                                                :: 10`
- `ULongLong                                            :: 11`
- `UInt128                                              :: 12`
- `Char_S                                               :: 13`
- `SChar                                                :: 14`
- `WChar                                                :: 15`
- `Short                                                :: 16`
- `Int                                                  :: 17`
- `Long                                                 :: 18`
- `LongLong                                             :: 19`
- `Int128                                               :: 20`
- `Float                                                :: 21`
- `Double                                               :: 22`
- `LongDouble                                           :: 23`
- `NullPtr                                              :: 24`
- `Overload                                             :: 25`
- `Dependent                                            :: 26`
- `ObjCId                                               :: 27`
- `ObjCClass                                            :: 28`
- `ObjCSel                                              :: 29`
- `Float128                                             :: 30`
- `Half                                                 :: 31`
- `Float16                                              :: 32`
- `ShortAccum                                           :: 33`
- `Accum                                                :: 34`
- `LongAccum                                            :: 35`
- `UShortAccum                                          :: 36`
- `UAccum                                               :: 37`
- `ULongAccum                                           :: 38`
- `BFloat16                                             :: 39`
- `Ibm128                                               :: 40`
- `HLSLInlineSpirv                                      :: 181`
- `HLSLAttributedResource                               :: 180`
- `Complex                                              :: 100`
- `Pointer                                              :: 101`
- `BlockPointer                                         :: 102`
- `LValueReference                                      :: 103`
- `RValueReference                                      :: 104`
- `Record                                               :: 105`
- `Enum                                                 :: 106`
- `Typedef                                              :: 107`
- `ObjCInterface                                        :: 108`
- `ObjCObjectPointer                                    :: 109`
- `FunctionNoProto                                      :: 110`
- `FunctionProto                                        :: 111`
- `ConstantArray                                        :: 112`
- `Vector                                               :: 113`
- `IncompleteArray                                      :: 114`
- `VariableArray                                        :: 115`
- `DependentSizedArray                                  :: 116`
- `MemberPointer                                        :: 117`
- `Auto                                                 :: 118`
- `Elaborated                                           :: 119`
- `Pipe                                                 :: 120`
- `OCLImage1dRO                                         :: 121`
- `OCLImage1dArrayRO                                    :: 122`
- `OCLImage1dBufferRO                                   :: 123`
- `OCLImage2dRO                                         :: 124`
- `OCLImage2dArrayRO                                    :: 125`
- `OCLImage2dDepthRO                                    :: 126`
- `OCLImage2dArrayDepthRO                               :: 127`
- `OCLImage2dMSAARO                                     :: 128`
- `OCLImage2dArrayMSAARO                                :: 129`
- `OCLImage2dMSAADepthRO                                :: 130`
- `OCLImage2dArrayMSAADepthRO                           :: 131`
- `OCLImage3dRO                                         :: 132`
- `OCLImage1dWO                                         :: 133`
- `OCLImage1dArrayWO                                    :: 134`
- `OCLImage1dBufferWO                                   :: 135`
- `OCLImage2dWO                                         :: 136`
- `OCLImage2dArrayWO                                    :: 137`
- `OCLImage2dDepthWO                                    :: 138`
- `OCLImage2dArrayDepthWO                               :: 139`
- `OCLImage2dMSAAWO                                     :: 140`
- `OCLImage2dArrayMSAAWO                                :: 141`
- `OCLImage2dMSAADepthWO                                :: 142`
- `OCLImage2dArrayMSAADepthWO                           :: 143`
- `OCLImage3dWO                                         :: 144`
- `OCLImage1dRW                                         :: 145`
- `OCLImage1dArrayRW                                    :: 146`
- `OCLImage1dBufferRW                                   :: 147`
- `OCLImage2dRW                                         :: 148`
- `OCLImage2dArrayRW                                    :: 149`
- `OCLImage2dDepthRW                                    :: 150`
- `OCLImage2dArrayDepthRW                               :: 151`
- `OCLImage2dMSAARW                                     :: 152`
- `OCLImage2dArrayMSAARW                                :: 153`
- `OCLImage2dMSAADepthRW                                :: 154`
- `OCLImage2dArrayMSAADepthRW                           :: 155`
- `OCLImage3dRW                                         :: 156`
- `OCLSampler                                           :: 157`
- `OCLEvent                                             :: 158`
- `OCLQueue                                             :: 159`
- `OCLReserveID                                         :: 160`
- `ObjCObject                                           :: 161`
- `ObjCTypeParam                                        :: 162`
- `Attributed                                           :: 163`
- `OCLIntelSubgroupAVCMcePayload                        :: 164`
- `OCLIntelSubgroupAVCImePayload                        :: 165`
- `OCLIntelSubgroupAVCRefPayload                        :: 166`
- `OCLIntelSubgroupAVCSicPayload                        :: 167`
- `OCLIntelSubgroupAVCMceResult                         :: 168`
- `OCLIntelSubgroupAVCImeResult                         :: 169`
- `OCLIntelSubgroupAVCRefResult                         :: 170`
- `OCLIntelSubgroupAVCSicResult                         :: 171`
- `OCLIntelSubgroupAVCImeResultSingleReferenceStreamout :: 172`
- `OCLIntelSubgroupAVCImeResultDualReferenceStreamout   :: 173`
- `OCLIntelSubgroupAVCImeSingleReferenceStreamin        :: 174`
- `OCLIntelSubgroupAVCImeDualReferenceStreamin          :: 175`
- `OCLIntelSubgroupAVCImeResultSingleRefStreamout       :: 172`
- `OCLIntelSubgroupAVCImeResultDualRefStreamout         :: 173`
- `OCLIntelSubgroupAVCImeSingleRefStreamin              :: 174`
- `OCLIntelSubgroupAVCImeDualRefStreamin                :: 175`
- `ExtVector                                            :: 176`
- `Atomic                                               :: 177`
- `BTFTagAttributed                                     :: 178`
- `HLSLResource                                         :: 179`
- `FirstBuiltin                                         :: 2`
- `LastBuiltin                                          :: 40`

#### `CXCallingConv` (enum)

- `Default            :: 0`
- `C                  :: 1`
- `X86StdCall         :: 2`
- `X86FastCall        :: 3`
- `X86ThisCall        :: 4`
- `X86Pascal          :: 5`
- `AAPCS              :: 6`
- `AAPCS_VFP          :: 7`
- `X86RegCall         :: 8`
- `IntelOclBicc       :: 9`
- `Win64              :: 10`
- `X86_64Win64        :: 10`
- `X86_64SysV         :: 11`
- `X86VectorCall      :: 12`
- `Swift              :: 13`
- `PreserveMost       :: 14`
- `PreserveAll        :: 15`
- `AArch64VectorCall  :: 16`
- `SwiftAsync         :: 17`
- `AArch64SVEPCS      :: 18`
- `M68kRTD            :: 19`
- `PreserveNone       :: 20`
- `RISCVVectorCall    :: 21`
- `RISCVVLSCall_32    :: 22`
- `RISCVVLSCall_64    :: 23`
- `RISCVVLSCall_128   :: 24`
- `RISCVVLSCall_256   :: 25`
- `RISCVVLSCall_512   :: 26`
- `RISCVVLSCall_1024  :: 27`
- `RISCVVLSCall_2048  :: 28`
- `RISCVVLSCall_4096  :: 29`
- `RISCVVLSCall_8192  :: 30`
- `RISCVVLSCall_16384 :: 31`
- `RISCVVLSCall_32768 :: 32`
- `RISCVVLSCall_65536 :: 33`
- `Invalid            :: 100`
- `Unexposed          :: 200`

#### `CXType` (struct)

- `kind: CXTypeKind`
- `data: [2] *void`

#### `CXTemplateArgumentKind` (enum)

- `Null              :: 0`
- `Type              :: 1`
- `Declaration       :: 2`
- `NullPtr           :: 3`
- `Integral          :: 4`
- `Template          :: 5`
- `TemplateExpansion :: 6`
- `Expression        :: 7`
- `Pack              :: 8`
- `Invalid           :: 9`

#### `CXTypeNullabilityKind` (enum)

- `NonNull        :: 0`
- `Nullable       :: 1`
- `Unspecified    :: 2`
- `Invalid        :: 3`
- `NullableResult :: 4`

#### `CXTypeLayoutError` (enum)

- `Invalid          :: -1`
- `Incomplete       :: -2`
- `Dependent        :: -3`
- `NotConstantSize  :: -4`
- `InvalidFieldName :: -5`
- `Undeduced        :: -6`

#### `CXRefQualifierKind` (enum)

- `None   :: 0`
- `LValue :: 1`
- `RValue :: 2`

#### `CX_CXXAccessSpecifier` (enum)

- `CXXInvalidAccessSpecifier :: 0`
- `CXXPublic                 :: 1`
- `CXXProtected              :: 2`
- `CXXPrivate                :: 3`

#### `CX_StorageClass` (enum)

- `Invalid              :: 0`
- `None                 :: 1`
- `Extern               :: 2`
- `Static               :: 3`
- `PrivateExtern        :: 4`
- `OpenCLWorkGroupLocal :: 5`
- `Auto                 :: 6`
- `Register             :: 7`

#### `CX_BinaryOperatorKind` (enum)

- `Invalid   :: 0`
- `PtrMemD   :: 1`
- `PtrMemI   :: 2`
- `Mul       :: 3`
- `Div       :: 4`
- `Rem       :: 5`
- `Add       :: 6`
- `Sub       :: 7`
- `Shl       :: 8`
- `Shr       :: 9`
- `Cmp       :: 10`
- `LT        :: 11`
- `GT        :: 12`
- `LE        :: 13`
- `GE        :: 14`
- `EQ        :: 15`
- `NE        :: 16`
- `And       :: 17`
- `Xor       :: 18`
- `Or        :: 19`
- `LAnd      :: 20`
- `LOr       :: 21`
- `Assign    :: 22`
- `MulAssign :: 23`
- `DivAssign :: 24`
- `RemAssign :: 25`
- `AddAssign :: 26`
- `SubAssign :: 27`
- `ShlAssign :: 28`
- `ShrAssign :: 29`
- `AndAssign :: 30`
- `XorAssign :: 31`
- `OrAssign  :: 32`
- `Comma     :: 33`
- `LAST      :: 33`

#### `CXChildVisitResult` (enum)

- `Break    :: 0`
- `Continue :: 1`
- `Recurse  :: 2`

#### `CXPrintingPolicyProperty` (enum)

- `Indentation                           :: 0`
- `SuppressSpecifiers                    :: 1`
- `SuppressTagKeyword                    :: 2`
- `IncludeTagDefinition                  :: 3`
- `SuppressScope                         :: 4`
- `SuppressUnwrittenScope                :: 5`
- `SuppressInitializers                  :: 6`
- `ConstantArraySizeAsWritten            :: 7`
- `AnonymousTagLocations                 :: 8`
- `SuppressStrongLifetime                :: 9`
- `SuppressLifetimeQualifiers            :: 10`
- `SuppressTemplateArgsInCXXConstructors :: 11`
- `Bool                                  :: 12`
- `Restrict                              :: 13`
- `Alignof                               :: 14`
- `UnderscoreAlignof                     :: 15`
- `UseVoidForZeroParams                  :: 16`
- `TerseOutput                           :: 17`
- `PolishForDeclaration                  :: 18`
- `Half                                  :: 19`
- `MSWChar                               :: 20`
- `IncludeNewlines                       :: 21`
- `MSVCFormatting                        :: 22`
- `ConstantsAsWritten                    :: 23`
- `SuppressImplicitBase                  :: 24`
- `FullyQualifiedName                    :: 25`
- `LastProperty                          :: 25`

#### `CXObjCPropertyAttrKind` (enum)

- `noattr            :: 0`
- `readonly          :: 1`
- `getter            :: 2`
- `assign            :: 4`
- `readwrite         :: 8`
- `retain            :: 16`
- `copy              :: 32`
- `nonatomic         :: 64`
- `setter            :: 128`
- `atomic            :: 256`
- `weak              :: 512`
- `strong            :: 1024`
- `unsafe_unretained :: 2048`
- `class             :: 4096`

#### `CXObjCDeclQualifierKind` (enum)

- `None   :: 0`
- `In     :: 1`
- `Inout  :: 2`
- `Out    :: 4`
- `Bycopy :: 8`
- `Byref  :: 16`
- `Oneway :: 32`

#### `CXNameRefFlags` (enum)

- `WantQualifier    :: 1`
- `WantTemplateArgs :: 2`
- `WantSinglePiece  :: 4`

#### `CXTokenKind` (enum)

- `Punctuation :: 0`
- `Keyword     :: 1`
- `Identifier  :: 2`
- `Literal     :: 3`
- `Comment     :: 4`

#### `CXToken` (struct)

- `int_data: [4] u32`
- `ptr_data: *void`

#### `CXCompletionResult` (struct)

- `CursorKind:       CXCursorKind`
- `CompletionString: CXCompletionString`

#### `CXCompletionChunkKind` (enum)

- `Optional         :: 0`
- `TypedText        :: 1`
- `Text             :: 2`
- `Placeholder      :: 3`
- `Informative      :: 4`
- `CurrentParameter :: 5`
- `LeftParen        :: 6`
- `RightParen       :: 7`
- `LeftBracket      :: 8`
- `RightBracket     :: 9`
- `LeftBrace        :: 10`
- `RightBrace       :: 11`
- `LeftAngle        :: 12`
- `RightAngle       :: 13`
- `Comma            :: 14`
- `ResultType       :: 15`
- `Colon            :: 16`
- `SemiColon        :: 17`
- `Equal            :: 18`
- `HorizontalSpace  :: 19`
- `VerticalSpace    :: 20`

#### `CXCodeCompleteResults` (struct)

- `Results:    *CXCompletionResult`
- `NumResults: u32`

#### `CXCodeComplete_Flags` (enum_flags)

- `IncludeMacros                :: 0x1`
- `IncludeCodePatterns          :: 0x2`
- `IncludeBriefComments         :: 0x4`
- `SkipPreamble                 :: 0x8`
- `IncludeCompletionsWithFixIts :: 0x10`

#### `CXCompletionContext` (enum_flags)

- `Unexposed           :: 0x0`
- `AnyType             :: 0x1`
- `AnyValue            :: 0x2`
- `ObjCObjectValue     :: 0x4`
- `ObjCSelectorValue   :: 0x8`
- `CXXClassTypeValue   :: 0x10`
- `DotMemberAccess     :: 0x20`
- `ArrowMemberAccess   :: 0x40`
- `ObjCPropertyAccess  :: 0x80`
- `EnumTag             :: 0x100`
- `UnionTag            :: 0x200`
- `StructTag           :: 0x400`
- `ClassTag            :: 0x800`
- `Namespace           :: 0x1000`
- `NestedNameSpecifier :: 0x2000`
- `ObjCInterface       :: 0x4000`
- `ObjCProtocol        :: 0x8000`
- `ObjCCategory        :: 0x10000`
- `ObjCInstanceMessage :: 0x20000`
- `ObjCClassMessage    :: 0x40000`
- `ObjCSelectorName    :: 0x80000`
- `MacroName           :: 0x100000`
- `NaturalLanguage     :: 0x200000`
- `IncludedFile        :: 0x400000`
- `Unknown             :: 0x7fffff`

#### `CXEvalResultKind` (enum)

- `Int            :: 1`
- `Float          :: 2`
- `ObjCStrLiteral :: 3`
- `StrLiteral     :: 4`
- `CFStr          :: 5`
- `Other          :: 6`
- `UnExposed      :: 0`

#### `CXVisitorResult` (enum)

- `Break    :: 0`
- `Continue :: 1`

#### `CXCursorAndRangeVisitor` (struct)

- `_context: *void`
- `visit:    #type (_context: *void, unknown0: CXCursor, unknown1: CXSourceRange) -> CXVisitorResult #c_call`

#### `CXResult` (enum)

- `Success    :: 0`
- `Invalid    :: 1`
- `VisitBreak :: 2`

#### `CXIdxLoc` (struct)

- `ptr_data: [2] *void`
- `int_data: u32`

#### `CXIdxIncludedFileInfo` (struct)

- `hashLoc:        CXIdxLoc`
- `filename:       *u8`
- `file:           CXFile`
- `isImport:       s32`
- `isAngled:       s32`
- `isModuleImport: s32`

#### `CXIdxImportedASTFileInfo` (struct)

- `file:       CXFile`
- `module:     CXModule`
- `loc:        CXIdxLoc`
- `isImplicit: s32`

#### `CXIdxEntityKind` (enum)

- `Unexposed             :: 0`
- `Typedef               :: 1`
- `Function              :: 2`
- `Variable              :: 3`
- `Field                 :: 4`
- `EnumConstant          :: 5`
- `ObjCClass             :: 6`
- `ObjCProtocol          :: 7`
- `ObjCCategory          :: 8`
- `ObjCInstanceMethod    :: 9`
- `ObjCClassMethod       :: 10`
- `ObjCProperty          :: 11`
- `ObjCIvar              :: 12`
- `Enum                  :: 13`
- `Struct                :: 14`
- `Union                 :: 15`
- `CXXClass              :: 16`
- `CXXNamespace          :: 17`
- `CXXNamespaceAlias     :: 18`
- `CXXStaticVariable     :: 19`
- `CXXStaticMethod       :: 20`
- `CXXInstanceMethod     :: 21`
- `CXXConstructor        :: 22`
- `CXXDestructor         :: 23`
- `CXXConversionFunction :: 24`
- `CXXTypeAlias          :: 25`
- `CXXInterface          :: 26`
- `CXXConcept            :: 27`

#### `CXIdxEntityLanguage` (enum)

- `None  :: 0`
- `C     :: 1`
- `ObjC  :: 2`
- `CXX   :: 3`
- `Swift :: 4`

#### `CXIdxEntityCXXTemplateKind` (enum)

- `NonTemplate                   :: 0`
- `Template                      :: 1`
- `TemplatePartialSpecialization :: 2`
- `TemplateSpecialization        :: 3`

#### `CXIdxAttrKind` (enum)

- `Unexposed          :: 0`
- `IBAction           :: 1`
- `IBOutlet           :: 2`
- `IBOutletCollection :: 3`

#### `CXIdxAttrInfo` (struct)

- `kind:   CXIdxAttrKind`
- `cursor: CXCursor`
- `loc:    CXIdxLoc`

#### `CXIdxEntityInfo` (struct)

- `kind:          CXIdxEntityKind`
- `templateKind:  CXIdxEntityCXXTemplateKind`
- `lang:          CXIdxEntityLanguage`
- `name:          *u8`
- `USR:           *u8`
- `cursor:        CXCursor`
- `attributes:    **CXIdxAttrInfo`
- `numAttributes: u32`

#### `CXIdxContainerInfo` (struct)

- `cursor: CXCursor`

#### `CXIdxIBOutletCollectionAttrInfo` (struct)

- `attrInfo:    *CXIdxAttrInfo`
- `objcClass:   *CXIdxEntityInfo`
- `classCursor: CXCursor`
- `classLoc:    CXIdxLoc`

#### `CXIdxDeclInfoFlags` (enum)

- `CXIdxDeclFlag_Skipped :: 1`

#### `CXIdxDeclInfo` (struct)

- `entityInfo:        *CXIdxEntityInfo`
- `cursor:            CXCursor`
- `loc:               CXIdxLoc`
- `semanticContainer: *CXIdxContainerInfo`
- `lexicalContainer:  *CXIdxContainerInfo`
- `isRedeclaration:   s32`
- `isDefinition:      s32`
- `isContainer:       s32`
- `declAsContainer:   *CXIdxContainerInfo`
- `isImplicit:        s32`
- `attributes:        **CXIdxAttrInfo`
- `numAttributes:     u32`
- `flags:             u32`

#### `CXIdxObjCContainerKind` (enum)

- `ForwardRef     :: 0`
- `Interface      :: 1`
- `Implementation :: 2`

#### `CXIdxObjCContainerDeclInfo` (struct)

- `declInfo: *CXIdxDeclInfo`
- `kind:     CXIdxObjCContainerKind`

#### `CXIdxBaseClassInfo` (struct)

- `base:   *CXIdxEntityInfo`
- `cursor: CXCursor`
- `loc:    CXIdxLoc`

#### `CXIdxObjCProtocolRefInfo` (struct)

- `protocol: *CXIdxEntityInfo`
- `cursor:   CXCursor`
- `loc:      CXIdxLoc`

#### `CXIdxObjCProtocolRefListInfo` (struct)

- `protocols:    **CXIdxObjCProtocolRefInfo`
- `numProtocols: u32`

#### `CXIdxObjCInterfaceDeclInfo` (struct)

- `containerInfo: *CXIdxObjCContainerDeclInfo`
- `superInfo:     *CXIdxBaseClassInfo`
- `protocols:     *CXIdxObjCProtocolRefListInfo`

#### `CXIdxObjCCategoryDeclInfo` (struct)

- `containerInfo: *CXIdxObjCContainerDeclInfo`
- `objcClass:     *CXIdxEntityInfo`
- `classCursor:   CXCursor`
- `classLoc:      CXIdxLoc`
- `protocols:     *CXIdxObjCProtocolRefListInfo`

#### `CXIdxObjCPropertyDeclInfo` (struct)

- `declInfo: *CXIdxDeclInfo`
- `getter:   *CXIdxEntityInfo`
- `setter:   *CXIdxEntityInfo`

#### `CXIdxCXXClassDeclInfo` (struct)

- `declInfo: *CXIdxDeclInfo`
- `bases:    **CXIdxBaseClassInfo`
- `numBases: u32`

#### `CXIdxEntityRefKind` (enum)

- `Direct   :: 1`
- `Implicit :: 2`

#### `CXSymbolRole` (enum)

- `None        :: 0`
- `Declaration :: 1`
- `Definition  :: 2`
- `Reference   :: 4`
- `Read        :: 8`
- `Write       :: 16`
- `Call        :: 32`
- `Dynamic     :: 64`
- `AddressOf   :: 128`
- `Implicit    :: 256`

#### `CXIdxEntityRefInfo` (struct)

- `kind:             CXIdxEntityRefKind`
- `cursor:           CXCursor`
- `loc:              CXIdxLoc`
- `referencedEntity: *CXIdxEntityInfo`
- `parentEntity:     *CXIdxEntityInfo`
- `container:        *CXIdxContainerInfo`
- `role:             CXSymbolRole`

#### `IndexerCallbacks` (struct)

- `abortQuery:             #type (client_data: CXClientData, reserved: *void) -> s32 #c_call`
- `diagnostic:             #type (client_data: CXClientData, unknown0: CXDiagnosticSet, reserved: *void) -> void #c_call`
- `enteredMainFile:        #type (client_data: CXClientData, mainFile: CXFile, reserved: *void) -> CXIdxClientFile #c_call`
- `ppIncludedFile:         #type (client_data: CXClientData, unknown0: *CXIdxIncludedFileInfo) -> CXIdxClientFile #c_call`
- `importedASTFile:        #type (client_data: CXClientData, unknown0: *CXIdxImportedASTFileInfo) -> CXIdxClientASTFile #c_call`
- `startedTranslationUnit: #type (client_data: CXClientData, reserved: *void) -> CXIdxClientContainer #c_call`
- `indexDeclaration:       #type (client_data: CXClientData, unknown0: *CXIdxDeclInfo) -> void #c_call`
- `indexEntityReference:   #type (client_data: CXClientData, unknown0: *CXIdxEntityRefInfo) -> void #c_call`

#### `CXIndexOptFlags` (enum)

- `None                                :: 0`
- `SuppressRedundantRefs               :: 1`
- `IndexFunctionLocalSymbols           :: 2`
- `IndexImplicitTemplateInstantiations :: 4`
- `SuppressWarnings                    :: 8`
- `SkipParsedBodiesInSession           :: 16`

#### `CXBinaryOperatorKind` (enum)

- `Invalid   :: 0`
- `PtrMemD   :: 1`
- `PtrMemI   :: 2`
- `Mul       :: 3`
- `Div       :: 4`
- `Rem       :: 5`
- `Add       :: 6`
- `Sub       :: 7`
- `Shl       :: 8`
- `Shr       :: 9`
- `Cmp       :: 10`
- `LT        :: 11`
- `GT        :: 12`
- `LE        :: 13`
- `GE        :: 14`
- `EQ        :: 15`
- `NE        :: 16`
- `And       :: 17`
- `Xor       :: 18`
- `Or        :: 19`
- `LAnd      :: 20`
- `LOr       :: 21`
- `Assign    :: 22`
- `MulAssign :: 23`
- `DivAssign :: 24`
- `RemAssign :: 25`
- `AddAssign :: 26`
- `SubAssign :: 27`
- `ShlAssign :: 28`
- `ShrAssign :: 29`
- `AndAssign :: 30`
- `XorAssign :: 31`
- `OrAssign  :: 32`
- `Comma     :: 33`
- `Last      :: 33`

#### `CXUnaryOperatorKind` (enum)

- `Invalid   :: 0`
- `PostInc   :: 1`
- `PostDec   :: 2`
- `PreInc    :: 3`
- `PreDec    :: 4`
- `AddrOf    :: 5`
- `Deref     :: 6`
- `Plus      :: 7`
- `Minus     :: 8`
- `Not       :: 9`
- `LNot      :: 10`
- `Real      :: 11`
- `Imag      :: 12`
- `Extension :: 13`
- `Coawait   :: 14`

## Runtime requirements

`generate_bindings` calls into `libgenerator_platform` (library loading, symbols, environment), libclang and a C compiler. A release archive ships the first in `native-libs/<platform>/` and, on Linux and Windows, libclang; on macOS libclang comes from the Xcode toolchain. When one is missing the error names it and the places searched. Details and the reasons: [releases](../../../tools/releases.md#bindings_generator).

Language: the first source's extension picks C or C++ (`.h` is C). With no `-x` in `extra_clang_arguments`, a header that fails to parse as C is parsed again as C++, and that parse is used when it succeeds (a class in a `.h` works); when both fail, the C errors are reported. An explicit `-x` in `extra_clang_arguments` is always used as given and never retried. The language is the one the macro and expression re-parses use too (`gr_parse` in `internal/developer-tools/generator-run.jai`).

## Behavioral contract and constraints

This package generates Jai declarations/wrappers from C and C++ headers using libclang. Its public records and procedures describe generator inputs, parsing/configuration, and emitted binding output; errors arise from header parsing, unsupported constructs, or output generation and are reported by generator diagnostics/results. Generated declarations inherit the source ABI and native library's ownership rules. The generator itself depends on libclang; the precise accepted C++ language/version matrix is unspecified.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: `cpp_library`, `libclang`.

## Public behavior details

`generate_bridge(options, filename)` writes C++ adapter source to `filename` and returns whether generation succeeded. The caller supplies any required C++ declarations through the configured header text and is responsible for compiling and linking the generated source with the C++ library. This API does not build a library or define a generated library filename or `cpp_library` identity. It emits adapters for supported, visible, non-deprecated, non-operator C++ functions represented by the generated declarations when a C++ linkage name and mapped external library are available; it does not promise an adapter for every declaration in the input. The C-linkage adapter name follows the generated binding declaration's output name, including its visible parent-scope path, and is the name the Jai foreign declaration must link against. Whitespace, comments, and other source formatting are not compatibility guarantees. Aggregate-by-value results are represented by a leading out pointer; C++ reference results are exposed as pointers. Functions requiring a printf-style adapter replace the original variadic tail with a signed 64-bit message byte count and a character pointer to already formatted data. Unsupported declarations can be omitted; successful generation alone does not guarantee that the output compiles or links in an arbitrary build environment.

The comparison helpers expose deliberately limited equivalence:

- `is_same_type` erases typedef aliases and ignores top-level `const`/`volatile` qualification. Pointer and reference types are treated as the same pointer-shaped category. Nominal enum and record types compare by represented declaration identity; function types compare return and parameter types, without requiring matching calling-convention metadata. Scalar types compare their represented numeric category and size. This is not a general C++ semantic-equivalence query.
- `is_same_expression` recognizes identical expression objects and deeply compares literal expressions. Distinct non-literal expressions are not considered equal merely because their syntax is alike.
- `is_same_literal` compares signed and unsigned integral values within their respective categories; floating values use ordinary equality (`+0` and `-0` compare equal, while distinct NaNs do not); strings compare contents; raw-token literals compare token text; enum literals require the same enum declaration and member name; identifier literals require the same declaration. Macro-value literals are not considered equal unless they are the same object. Locations and source spelling do not change these rules.
- `is_same_location` requires the same `CXFile` handle and equal line and column. Equal paths represented by distinct handles are not guaranteed to compare equal, including across translation units.

The libclang `clang_getFileTime` binding returns the target branch's `time_t`. On supported POSIX branches this is the POSIX module's signed 64-bit `time_t`; the Windows branch uses a module-local signed 64-bit alias. `time_t` is not exported as a Bindings_Generator-owned public type. The storage width is 64 bits in the declared branches, while nominal type identity follows the branch's declaration.


`init_generator` installs the supplied pointer in the module's active `context.generator` slot and initializes its canonical primitive types. `run_generator` also installs that pointer and sets the active options before using the context-dependent APIs. Procedures such as `find_lib_for_foreign_name` rely on this active slot and scan `Generator.libraries` in order, returning the first library where the symbol is found, or null. The scan does not exclude `LINK_ALWAYS` libraries. `deinit_generator` disposes the generator's token sets, translation unit, and clang index; it does not clear or restore the active slot. The caller owns the `Generator` record and must keep it valid while context-dependent operations run. Any nested activation/restoration policy belongs to the caller's ordinary active-context management; the module does not provide a separate generator stack or lifecycle registry.

`os_get_symbol_names_from_file_fixed_and_cached` caches successful results by exact `(OS, CPU, full_path)` values. A cache hit returns the cached symbol-name slice; a successful miss also stores copied path/name strings in the module cache. No public invalidation procedure is provided. On Apple targets the function removes one leading underscore from each discovered symbol name before caching/returning it; `fix_macos_foreign_name` applies the same one-prefix removal to a supplied foreign name on macOS and iOS. Treat the returned names as borrowed cache data on cache hits; mutation can affect subsequent results. `get_libc_paths` returns include-directory roots, not linkable library files: Linux supplies generic and platform-specific include roots, Android supplies NDK include roots, Apple targets return the SDK `usr/include` root in both outputs, and unsupported targets return empty paths with `success = false`.

`ensure_local_libclang_exists(source_path)` compares the source library's modification time and file size with a file of the same basename in the current working directory. It copies the source over that local file if the local file is absent, older, or has a different size; otherwise it leaves the local file in place. A missing source triggers an assertion. The no-argument overload selects the bundled libclang file for Windows, macOS, or Linux and asserts for other operating systems. It does not return a status or validate libclang compatibility.

The cursor-based model constructors use `Generator.declarations_by_cursor` to preserve declaration identity: record, enum, namespace, typedef, function, and Objective-C interface/type-parameter cursors are registered by canonical cursor rather than creating a second declaration for a redeclaration. Repeated `create_function` calls return null after refreshing the existing declaration's source metadata; the existing entry remains canonical. `find_referenced_declaration` only looks up the cursor returned by libclang for the reference; it returns null when no entry is present and does not construct a declaration on demand. `create_declaration` dispatches supported cursor categories and adds access flags to a non-null result; `create_declaration_inner` performs the dispatch without that final access-flag step. `ignore_non_structs` suppresses function/variable/macro work while retaining namespaces and their nested type declarations. Function templates, conversion functions, unsupported template specializations, and unexposed expressions are not converted and may produce null. `resolving_type_only` on record, enum, and Objective-C constructors returns the registered type shell without populating its ordinary member/constant list during that call.

`create_type` uses the canonical primitive type objects held by the active `Generator`. Pointer types are represented as pointer-to-type records; lvalue references retain a reference flag. For an elaborated/template type, `cursor_is_accurate` controls whether the associated cursor's children can supply the template references; when false, `substitute_children` is the caller-supplied child view, and a missing usable view can make the type unrepresentable. `type_arg_index` selects the relevant Objective-C type-parameter reference among cursor children. `create_elaborated_type`'s integer result is the number of supplied child cursors consumed while recovering the represented type/template arguments (or namespace references plus the type reference for a non-unexposed named type); unsupported forms return a null type and zero consumed children.

`create_function_type` records variadic, inline, constructor/destructor, C++ method, and Objective-C method categories from cursor/type metadata. A non-static C++ method receives a leading pointer-shaped `this` argument; a const method marks that pointer const. Only parameter declaration children become ordinary arguments; missing names are assigned `unknown0`, `unknown1`, and so on. A no-prototype case has no separate synthesis rule: the argument list reflects the parameter children libclang supplies, plus the implicit or target-specific arguments described here. The result type and each parameter must be representable or the operation can return null. `create_function` records virtual/pure-virtual and deprecation flags from libclang cursor queries. Constructor and destructor declarations receive normalized output names; Objective-C methods use their selector as foreign name and remove the colon suffix from the output identifier. Other functions require a nonempty clang mangling; when unavailable, this helper returns null. `create_variable_declaration` reports any required wrapper flags separately in its second result; a const lvalue-reference parameter may set the parent function's argument-wrapper flag, except when that function is virtual. A pointer parameter initialized with a struct literal also requires a wrapper and is marked pointer-to-value. Fixed-size arrays and bitfields do not receive an initializer expression from this routine. Its `is_function_argument` option enables the function-parameter array normalization behavior. `create_objc_type_param` returns the canonical declaration with its type set to the generator's `Type` type; unsupported cursor forms are outside this contract.

`create_typedef` registers by canonical cursor and builds its underlying type. If the underlying type cannot be represented it returns null. When the configured typedef-unwrapping rule matches a named underlying type, it records the typedef in `unwrapped_typedefs` and returns null because callers should print the underlying name instead. When a typedef supplies the name for an otherwise anonymous record or enum, that type receives the typedef's public name and the alias declaration is marked omitted. `create_enum` uses Clang's declared underlying integer type; enumerator values are stored as signed values for signed underlying types and unsigned values otherwise. `create_struct` preserves the canonical record identity; unsupported specializations return null. Namespace construction preserves nested declaration order, and duplicate equivalent functions are kept in the model but marked omitted from output. Objective-C interface, protocol, and category cursors are represented using the public `Struct` model; type-only resolution returns the canonical shell, and full resolution adds the parent/protocol relationships and method children libclang exposes. Protocols have an `ID` parent entry, root interfaces expose an `ID` parent entry, and category cursors attach to their target class. Objective-C ivars are not emitted as ordinary struct fields. `create_objc_type_param` requires a `TemplateTypeParameter` cursor.

`create_literal(raw_tokens, location)` creates a raw-token literal whose text is kept as supplied; `create_literal(Macro_Value, location)` creates a macro literal backed by that token view. The cursor overload accepts libclang-evaluated signed/unsigned integers, floating values, and string values; it copies evaluated string contents. An unexposed string evaluation is retried through its parent cursor. A reference expression is represented only when it resolves to a const declaration; an unresolved or non-const reference returns null, as does a static const declaration without an initializer expression. Other unevaluable cursor kinds return null. `create_macro_definition` excludes common include guards, configured ignored names, empty definitions, and function-like macros; accepted definitions keep the replacement tokens in the generator's token-lifetime list for later use. The caller must keep the generator's translation unit alive until those token views are no longer used and call `deinit_generator` before disposing that translation unit.

`create_expression_from_cursor` returns `skip = true` only for cursor kinds outside libclang's expression-kind range. For expression cursors, unsupported/unrepresentable cases return a null expression with `skip = false`; callers must not treat that as an ignorable non-expression. Its supported forms include evaluated literals/references/null, unary and binary operators, transparent unexposed or C-style cast wrappers, and supported constructor-style aggregate initializers. Several C++/Objective-C forms including member access, array subscripting, conditional expressions, compound assignment, and Objective-C message expressions remain unsupported and return null. Dependent `TemplateRef` expression arguments are also unsupported. `convert_children_to_expression` converts only the final child expression and returns null when there is no usable child.

`is_token` compares a token's libclang spelling with the requested string and therefore requires the active generator translation unit. `count_asterisks` counts consecutive leading token spellings equal to `*` and stops at the first different spelling. `find_closing_paren_token_index` returns the first closing-parenthesis index encountered when the scan's nesting counter is already zero; nested parentheses encountered after the start are accounted for. The caller passes the first token inside the opener: for `(`, `name`, `)` and `start_index = 1`, it returns index 2. Passing index 0 skips that opener's matching close and searches for a subsequent zero-depth close. It returns -1 if none occurs. `macro_values_are_equal` requires equal token counts and equal corresponding token spellings; it does not compare token kinds or source spacing. `macro_literals_are_equal` is false unless both operands are macro literals, then applies that token-spelling comparison.

`find_and_check_macro_identifier` checks built-in `void`/`int` spellings first, then the supplied enum's own values, macro-converted enum values, unwrapped typedef names, and finally global-scope declarations in stored order. Its omit option bypasses omit filtering for enum/global-scope lookup. A matching macro in `NEEDS_CHECKING` is validated during lookup; lookup succeeds only if the resulting state is `KEEP`. `NEEDS_CHECKING` is therefore pending work, not an automatic lookup rejection. `DISCARD` is rejected, and `ignore_omit_flag` does not bypass that conversion decision. Unsupported identifiers return `found = false`. `find_or_create_enum_for_macro` reuses an enum with matching source or output name from the macro-enum list first, then from global scope; otherwise it creates an empty enum and copies the first declaration's location/comment metadata. It does not choose or infer an underlying integer width.

`find_or_create_extra_info` returns the cached per-record metadata when present. A template-parameter-like parent shape yields the shared unknown-metadata sentinel and null generated record; a non-record parent reports an error and returns null outputs. Record metadata can include derived alignment/field presence and generated virtual-function information, so the call may add declarations to the represented record; exact private metadata organization is not an API guarantee. `find_first_member_offset` returns Clang's bit offset for the first field found in the supplied outer record, descending through anonymous nested records; it returns -1 when no field is found. Other negative values can be Clang's layout-error sentinels. `can_omit_type` returns true only when `NO_ARGUMENT_DEFAULTS` is absent, the declaration has an expression, and the type being printed is bool. Other print flags do not broaden that rule.

Name normalization helpers have these observable rules:

- `avoid_cpp_keywords` changes only the exact identifier `this`, returning `_this`; all other inputs are returned unchanged.
- `RESERVED_NAMES` contains `cast`, `context`, `false`, `is_constant`, `null`, `remove`, `true`, `type_info`, `xx`, `interface`, `Any`, `bool`, `float`, `float32`, `float64`, `int`, `s16`, `s32`, `s64`, `s8`, `string`, `u16`, `u32`, `u64`, `u8`, `v128`, and `void`. `RESERVED_TYPE_NAMES` contains only `Type`. `avoid_reserved_names` prefixes a matched reserved name with `_`; the type-only list is checked only when `avoid_type_names` is true. For names starting with `operator`, it instead applies the Jai-compatible operator normalization described below. Otherwise it returns the input spelling unchanged. Returned string ownership is not separately promised.
- `replace_operator_name` removes spaces and replaces operator spellings with stable identifier fragments: `<=>` becomes `_wtf`, `->*` `_wtf2`, `->` `_arrow`, `++` `_plus_plus`, and `--` `_minus_minus`; in ordinary mode it additionally maps `==`, `!=`, `>=`, `<=`, `<<`, `>>`, `&&`, `||`, `+`, `-`, `*`, `/`, `%`, `^`, `&`, `|`, `~`, `<`, `>`, `[]`, `!`, and `=` respectively to `_equals`, `_not_equals`, `_greater_equals`, `_less_equals`, `_lshift`, `_rshift`, `_and_and`, `_or_or`, `_plus`, `_minus`, `_times`, `_divide`, `_modulo`, `_xor`, `_and`, `_or`, `_tilde`, `_less`, `_greater`, `_brackets`, `_not`, and `_assign`. Both modes replace comma with `_comma`, `()` with `_parens`, and `""` with `_literal`. With `keep_operators_allowed_in_jai = true`, `operator!` and `operator=` become `operator_not` and `operator_assign`; the ordinary individual-operator substitutions are skipped. These substitutions are name mapping behavior, not a general parser for arbitrary operator spellings.
- `make_valid_identifer` mutates the bytes addressed by its string descriptor in place, replacing each `-` and `.` byte with `_`. It does not return a replacement string and does not otherwise validate or repair identifier syntax; empty strings and other bytes remain as supplied.
- `convert_objc_selector_name_to_function_identifier` mutates only the string count, truncating at the first colon if one exists. It leaves the bytes unchanged and leaves strings with no colon intact. `print_objc_selector_identifier` writes each input byte in order, substituting `_` for each colon; it does not truncate after the first selector component.
- `get_enum_value_name` first suppresses a value whose spelling ends with the configured nonempty suffix, returning the empty string. Otherwise it removes the first configured `(enum name, value prefix)` pair whose enum name exactly matches and whose prefix begins the value spelling; then, if enabled, it removes the enum name prefix and one following underscore. It returns a slice of the input spelling for prefix removal, or the original input when unchanged; it does not normalize reserved words here.
- `compute_enum_names` mutates enumerator `output_name` fields, skipping entries marked `OMIT_FROM_OUTPUT`. If automatic prefix detection is enabled, the enum has more than one value, and `already_stripped` is false, it computes a common prefix over the current output names. When enum-prefix stripping is enabled and a value's original name already begins with the enum name, detection is suppressed. A configured delimiter shortens the removable prefix to the last occurrence of that delimiter, including the delimiter. The chosen prefix is removed from each included output name, leading underscores are trimmed, and reserved-name handling is applied. After shortening, empty names and names beginning with a digit receive a leading underscore. For anonymous enums, a detected prefix (with trailing underscores removed) supplies the output name; otherwise an empty name becomes `anon_enum`. `already_stripped` disables only automatic common-prefix detection; it does not disable empty/digit repair or anonymous-name fallback. Omitted values are neither considered for nor changed by prefix processing. Collision handling for enumerator names and a guarantee that transformed names remain unique are not established.

Type and declaration helpers expose copy-on-write and eligibility behavior. `shared_type` sets the `SHARED` flag on the supplied `CType` object and returns that same pointer. `add_type_flags` mutates and returns the same object when it is not shared; for a shared input it returns a separate shallow copy with the existing `SHARED` bit cleared before OR-ing the requested flags, leaving the original object unchanged. If the caller includes `SHARED` in the requested flags, the result has that bit set again. Child pointers and other referenced records are not deep-copied. The API provides no separate ownership-transfer or release contract for these model objects. `append_calling_convention_directives` emits nothing for Objective-C methods; otherwise it emits `#cpp_method` for C++ methods, or `#c_call` when `assume_foreign` is false. For a non-void struct result it also emits `#cpp_return_type_is_non_pod` when the target-specific record flags indicate a non-POD return: constructor presence on Windows, or move/copy constructor or destructor presence on other supported targets. It does not emit a variadic directive. `append_type_info_directives` emits `#type_info_none` when that declaration flag is set; otherwise it emits `#type_info_procedures_are_void_pointers` when that flag is set. If both flags are present, the first takes precedence. `check_if_foreign_function_should_be_omitted` checks the enabled strip policies in this order: variadic-list parameters, missing foreign-library symbols, then inline functions. Missing-library classification exempts virtual functions and Objective-C methods. On the first matching policy it marks the function omitted, increments the active generator's stripped count once, and logs the reason only when `log_stripped_declarations` is enabled. A function exempted by the missing-library check can still be omitted by a later enabled policy only when that earlier policy did not match.

`should_be_moved_to_global_scope` returns true for declarations classified as extern/elsewhere and for functions when `move_struct_functions_to_global_scope` is enabled or the function is an operator. Other declarations return false. `get_name_alignment` considers only declaration, typedef, macro-definition, and bitfield members, and excludes members moved to global scope. Its result is the maximum byte count of `output_name`, plus six for an included `using` qualifier or four for an `as` qualifier when `ignore_flags` is false; it returns zero when no member qualifies. The slice overload applies the same rule through its supplied declaration selector. `append_indentation` appends `TAB_STRING` once per active generator `tab_depth` level; `TAB_STRING` is one space. These context-dependent helpers require the corresponding active generator/options state.

`maybe_add_spacing` always updates `last_location` to the supplied location. It appends a newline for an ordinary element and, when `NO_SPACING` is set, appends none unless source-based extra spacing is warranted. Extra spacing is warranted only when the previous location has a file and the new location is in another file or is on a line other than the same or immediately following line. When the selected `mimic` value is true this adds one extra newline (two for ordinary entries, one for `NO_SPACING`); when false it adds no extra blank line. The `Mimic_Spacing_Flags` overload selects that boolean by intersecting the requested declaration category with the active `mimic_spacing_flags` option. Unknown previous locations do not cause extra blank lines; same-line and consecutive-line locations do not either.

Comment helpers preserve a distinction between leading and trailing comments. `prepend_comment` replaces `comment.text` with the supplied text followed by one space and the previous text when both are nonempty; if there was no previous text it assigns the supplied string directly. It leaves the comment location unchanged. The module does not promise uniform allocation/ownership for the resulting text. `fill_comment` reads the raw comment attached to a cursor, trims surrounding whitespace, and leaves the existing `Comment` untouched when there is no comment. When text exists it replaces `comment.text` with a copy; it sets `comment.location` to the comment's expansion location only when that comment starts on a line before the supplied declaration location. `fill_location_and_comment_if_needed` refreshes both using the cursor when the location is unset or the cursor is its canonical definition; otherwise it leaves both as-is. A missing raw comment leaves existing comment contents intact even when the location is refreshed. `is_prefix_comment` is true when comment text has a nonzero recorded line or contains a newline. `append_prefix_comment` emits only prefix comments, splitting on newlines, trimming spaces/tabs/CR around each line, writing each line with a newline, and optionally appending current indentation after each line. `append_postfix_comment` emits only nonempty non-prefix comments, preceded by one space and with line endings normalized. Exact allocator ownership of comment strings and platform-specific libclang comment-location edge cases are not further specified.

`get_name_for_cursor` returns a copied cursor spelling, except spellings containing ` at ` are normalized to the empty string to handle libclang's anonymous-type descriptions. `debug_print_cursor` visits and logs descendants of the supplied cursor, including each descendant's spelling and kind with indentation by nesting depth; it does not log the supplied cursor itself. It relies on libclang cursor traversal and the active Jai logging context; output formatting beyond the reported spelling/kind and nesting is diagnostic, not generated binding text.

## Model integration clarifications

These clarifications describe observable behavior and public record relationships. They do not specify a private implementation or promise complete C++ ABI coverage.

### Function equivalence, naming, and omission

`functions_are_equivalent(a, b, ignore_first_argument)` requires equal `Function.name` and equal argument counts. Compared parameter types must satisfy the public `is_same_type` relation. When `ignore_first_argument` is true, the first parameter's type is excluded from comparison, but the argument counts must still match. Return types, parameter names/default expressions, output names, foreign names, parent identities, function flags, and calling conventions are not additional equivalence requirements. The relation inherits `is_same_type`'s documented limits; it is not full C++ signature equivalence. `contains_equivalent_function` considers function declarations in its supplied slice, including functions already marked omitted; other declaration kinds do not match.

Constructor `Function.name` spellings are `Constructor`, `CopyConstructor`, or `MoveConstructor`, according to the represented category. Destructor spelling is `Destructor`. These are case-sensitive. `output_name` subsequently follows the configured prefix stripping and reserved-name normalization, so it need not equal the normalized source name. Constructor/destructor strip options mark these functions omitted. Objective-C selector behavior remains as documented above.

`create_omit_cpp_functions_visitor` returns a callback/data pair preserving both supplied callbacks and the supplied opaque data pointer. Its omission test applies to a `.FUNCTION` whose `foreign_name` differs from `name`; it is not restricted to method/constructor cursor categories. Ordinary mangled C++ global functions therefore qualify. Such a function is marked `OMIT_FROM_OUTPUT`, the result is `STOP`, and neither wrapped callback is invoked for that declaration. A matching foreign/source name does not satisfy this omission test, regardless of other C++ category flags.

For declarations that pass the omission test, the plain callback is invoked before the callback with data. A plain callback returning `STOP` prevents the data callback; a `STOP` from either is propagated. Otherwise the wrapper returns `RECURSE`. Callback return behavior is observable visitor composition, not a requested implementation sequence for unrelated model construction. Null callbacks are allowed. The public API supplies no separate release function or allocation-lifetime guarantee for the returned visitor data.

### Active generator and lifetime

The public context addition declares `generator: *Generator`. For a single module instance, importer code can rely on this declared field type and on `init_generator`/`run_generator` installing the supplied pointer. No rule is specified for unifying distinct `Generator` nominal types across simultaneous parameterized module instances. A generic internal protocol may preserve pointer identity, but that implementation choice does not imply cross-instance public type equivalence.

`deinit_generator` disposes registered native token sets, the translation unit, and the clang index. It leaves the active context pointer installed. It does not promise to free all model records/copied strings, reset every model table, support repeated disposal, or make libclang-backed cursors/token views usable afterward. Conversely, the absence of aggregate deallocation is not a guarantee that every reference is independently valid after teardown. Keep the owning session and native resources alive through model construction, metadata consumption, and printing. A transaction ledger is a caller/integration ownership choice, not an additional public model API.

### Type and expression representation limits

| Question | Specified behavior |
| --- | --- |
| `wchar_t` | The conversion classifies `WChar` with signed integer types and selects the canonical signed integer by Clang-reported byte width. This is the module's mapping; it is not a claim that every target's native `wchar_t` is signed. No additional wchar-specific slot or flag is declared. |
| Extended floating types | `Float`, `Double`, and `LongDouble` map by native size: four bytes to canonical float32, eight to canonical float64, sixteen to null. Other sizes are unsupported and may diagnose. Half and C++ member-pointer types return null; the module guarantees no additional encoding or bridge calling convention for them. |
| Block pointers | `Generator.type_def_objc_block_pointer` is already a public canonical slot. `BlockPointer` conversion returns it; initialization gives it the named opaque record representation `Block`. This is not an ordinary function-pointer signature and does not promise block invocation or capture-layout support. |
| Integers wider than 64 bits | Canonical 128-bit integer `CType` slots exist, but `Literal.int_value` and `uint_value` and the evaluation accessors expose only 64-bit payloads. No exact extra literal/enum-value payload is established. Do not claim lossless wider evaluated values from this schema; rejecting an unrepresentable value is an explicit implementation limit, not a newly proven historical rejection rule. |
| Evaluated strings | The evaluated-string accessor supplies a C-string pointer, and the model stores a Jai string. The module guarantees no complete wide-string encoding, embedded-NUL extent preservation, or a safe inferred length beyond the C-string terminator. Copying evaluated contents is established; complete support for those additional literal domains is not. |
| Instantiated aggregate literal names | `Struct_Literal` exposes only `type_name: string` and `values: [] *Expression` beyond its base. Jai printing uses the supplied type-name text when a type prefix is needed, followed by aggregate-literal syntax; it can omit a matching target record name. The C++ expression helper also uses the supplied text directly and does not translate generic arguments or establish valid C++ aggregate syntax for every such value. There is no guaranteed neutral instantiated-name grammar or hidden CType/argument field. |
| Non-type template arguments | `CType.template_instantiation_params` contains expression records and can represent a supported non-type argument through the ordinary expression model. A blanket assertion that every non-type argument lacks representation is too strong. Dependent or otherwise unsupported expressions remain subject to the documented expression-conversion limits. |
| Formal template parameters/defaults | Type and template-template formals use a constant declaration of canonical `Type`; non-type formals use their converted native type. `Declaration.expression` can hold a successfully converted default. The module guarantees no general parameter-pack expansion, full template-template identity/arity constraints, or preservation of every dependent default. A diagnostic/comment for an unresolved default is not an equivalent emitted default. |

### Shared record metadata and vtables

`Generator.extra_infos` is the public cache keyed by canonical `*Struct`, with `Struct_Extra_Info` values. `find_or_create_extra_info` is the public operation that supplies a record's derived metadata on demand, including an uncached supported polymorphic record; requiring a different owner to prepopulate every such entry is not the public behavior. The producer and printer must agree on this one model.

`Struct_Extra_Info.modified_declarations` is the member list consumed for generated record output. `vtable_struct` represents a generated vtable record; `vtable_pointer_declaration` represents a record's own vtable-pointer member when needed. `vtable_parent`, `vtable_parent_info`, and `vtable_parent_declaration` identify inherited vtable relationships. `padding_declaration` and `raw_struct` can represent layout padding and a variant needed for inheritance. These are public typed pointers to actual shared model objects, not unrelated printer-only descriptions. `Struct.metadata` is a separate list for metadata declarations; it must not be confused with the extra-info cache.

For a supported generated vtable, the public model exposes ordered function entries and typed generated member declarations. Omitted virtual entries retain a pointer-sized placeholder instead of collapsing subsequent member offsets. An inherited override considered equivalent with the first parameter ignored does not add a new inherited slot. Public behavior distinguishes Windows same-name overload slot ordering from the non-Windows model. Slot order matters; this is not a complete portable recipe for C++ vtables.

General virtual-base displacement, all multiple-inheritance adjustment thunks, or every target's destructor-slot ABI are unspecified. Constructor/destructor category and platform-specific foreign-symbol metadata do not determine those layouts. In particular, native record size and field offsets alone do not determine virtual-base displacement or vtable entries. Supporting those cases requires independent ABI knowledge; a precise diagnosed limitation must remain visible until then. Do not invent a second metadata registry, guessed ABI fields, or a success result that conceals absent polymorphic metadata.

### Narrow helper contracts: literals, virtual calls, templates, and token text

The canonical signed and unsigned 128-bit `CType` slots are initialized as the
hardcoded Jai type names `S128` and `U128`, each with size 16. Native integer
types of size 16 map to these slots. `Literal` itself stores only `int_value:
s64` and `uint_value: u64`; the libclang evaluation path obtains only
those 64-bit payloads. It does not guarantee lossless evaluated literal values
outside those ranges or a two-limb representation. Ordinary integer output
prints the represented signed or unsigned 64-bit payload as decimal unless
hex output is selected. The signed/unsigned hex conversion helpers cover only
1, 2, 4, and 8-byte targets; a 16-byte target has no supported hex rendering
contract and reaches the helper's assertion path. No 128-bit hex spelling or
round-trip guarantee is specified.

For an ordinary signed `Literal` with a 16-byte signed target and a represented
payload such as `-42`, the printer's decimal path writes the signed decimal
payload directly; it does not synthesize `cast(S128)(-42)` or construct
`S128 {low, high}`. The public `S128` and `U128` layouts are limb structs, so a
scalar cast is not the documented conversion mechanism. This describes the
output path only; acceptance of the emitted decimal in every
target context and lossless 128-bit literal support are unspecified. The
model's payload remains limited to `s64`/`u64`.

For `Literal.literal_kind == .RAW_CPP_TOKENS`, `raw_value: string` is the
authoritative text. `string_value` is the distinct union arm for ordinary
string literals and is not the raw-token source. Literal printing appends
`raw_value` unchanged; it does not parse, qualify, escape, or validate those
bytes as Jai syntax. The caller is responsible for supplying valid output
text.

`raw_tokens_string_from_cursor(cur)` tokenizes the cursor extent and
concatenates token spellings without inserting whitespace. It does not retain
original spaces, comments, or formatting. When range tokenization fails, the
fallback may return only the token at the cursor location, with the spelling
available from that fallback path; if neither tokenization route yields a
token, the result is empty. Therefore the result is token-spelling text, not
a source-slice or a stable formatting representation.

For a record with virtual slots, `Struct_Extra_Info.vtable_struct` is the
typed record used for generated slot layout. The vtable pointer declaration
is inserted at the initial storage position of the record or generated raw
layout record when that record owns the vtable; when a vtable parent supplies
the pointer, the child does not add a second one. The generated vtable record
preserves slot order and pointer-sized placeholders for omitted slots so
later slots retain their positions. `print_struct_vtable` emits that record;
when enabled, it also emits inline `<Record>_<slot>` wrappers which call
`obj.vtable.<slot>(obj, ...)`, and an optional `vtable(obj)` accessor returning
the typed pointer. `print_function` gives direct virtual foreign declarations
the `virtual_` prefix (and `_CFormat` where applicable), retaining the
represented foreign library and symbol. This direct declaration is distinct
from invoking the slot through the vtable. Slot order and inheritance remain
subject to the target/ABI limits documented above.

The two public vtable-output options are independent. The typed vtable record
is emitted in all four settings; `generate_c_style_api_for_vtable` controls the
`<Record>_<slot>` inline wrappers, and `generate_vtable_helpers` controls the
typed `vtable(obj)` accessor:

| C-style slot wrappers | `vtable` accessor | Additional output |
| --- | --- | --- |
| off | off | Neither helper form |
| on | off | `<Record>_<slot>` wrappers |
| off | on | `vtable(obj)` accessor |
| on | on | Both helper forms |

Wrapper suffixes are taken from the generated vtable member's `output_name`
(for example `slot_0`), not re-derived from the original `Function.output_name`.
Variadic slot wrappers are skipped with a diagnostic; the typed vtable record
remains emitted.

Objective-C method output is a generated typed wrapper, not a Jai compiler
intrinsic. It declares a selector symbol and casts the selected Objective-C
runtime entry point to a function pointer whose parameters are the receiver,
selector, and represented method arguments. The wrapper uses `objc_msgSend`
for ordinary returns, `objc_msgSend_stret` for represented struct returns,
and `objc_msgSend_fpret` for represented floating-point returns. This records
the generator's emitted ABI selection; it does not certify that the selection
is correct for every Apple architecture or runtime version. No separate
compiler intrinsic spelling is required by this contract.

The wrapper takes its receiver from `Function_Type.arguments[0]`: that
argument's name and type become the receiver parameter and call operand, and
remaining represented arguments follow the selector in the runtime call. The
generator explicitly synthesizes `this` for C++ methods, but has no analogous
Objective-C receiver insertion in `create_function_type`; that constructor
collects Objective-C `ParmDecl` children. The Objective-C print path therefore
assumes the first represented argument is already the receiver. It applies the
same rule to instance and class methods; it does not synthesize a class object
from the enclosing interface. If the incoming canonical argument list omits a
receiver, a correct receiver signature is not specified (an
empty list cannot satisfy its first-argument access). Whether a particular
libclang cursor supplies an implicit receiver as a parameter is unspecified. Selector storage is emitted as a `SEL_<foreign declaration name>` foreign
selector symbol; no separate registration/lifetime manager is generated.

`Struct.template_type_params` is emitted in list order as the Jai generic
parameter list on the record declaration, using each parameter's represented
name, type, and printable default expression. A generated raw-layout variant,
when required, carries the same parameters and its using member supplies the
corresponding parameter references. Template records disable ordinary field
offset checks because a generic declaration does not determine all
instantiated layouts. The module explicitly leaves some template padding and
inheritance cases incomplete; an emitted generic record is not a guarantee
that every specialization has a validated native size, alignment, or offset.
### How to extend this contract

Keep constructors, the canonical cache, and printers consistent with the existing public record types. Define the supported domain for new ABI/type/template cases before claiming them equivalent. Configuration affecting these rules includes target OS/CPU, constructor/destructor strip flags, output-name normalization, `generate_c_style_api_for_vtable`, `generate_vtable_helpers`, and inheritance/layout options. Dependencies are the active `Generator`, configured options, libclang metadata, shared model constructors, and the public declaration/type printers. Allocation registries, transaction ledgers, and private session protocols remain implementation choices.

## Shared emission and parser helper contracts

This section clarifies callable helpers of the public module for independent implementation. It specifies observable results, mutations, and unsupported domains; it does not prescribe implementation bodies. Dependencies throughout are the canonical model, active generator/options, configured argument callback, libclang cursor/token views, and the existing printers. Native token/session lifetimes described above continue to apply.

### Option normalization and public generation workflows

`normalize_options` accepts zero, one, or multiple ordinary/system `Library` entries. It does not require one explicit library, invent a library for a header-only input, or discard earlier canonical entries. A `Library` has `filename` then `identifier`; an empty identifier requests loader derivation. The normalizer does not perform library loading or symbol discovery.

The returned options have these library/input adjustments:

- Legacy `libnames` append ordinary entries with that name as `filename` and an empty identifier; `system_library_names` append corresponding system entries. Legacy `libpaths` and `system_library_paths` append to their canonical search-path collections. Canonical entries precede appended legacy entries. Each nonempty legacy collection produces its deprecation warning and is reset in the returned options. No automatic deduplication is promised.
- If the resulting ordinary library search list is empty, it becomes `.`. Zero libraries still remain zero libraries. Platform-added system search paths can precede appended legacy system paths; legacy arrays do not replace explicit/default canonical paths.
- Empty system include paths request the platform's default include paths. Non-Windows empty static suffix becomes `.a`; Android also supplies `.so` for an empty shared suffix. Other empty suffixes remain available for loader defaults, rather than every platform suffix being written into normalized options.
- For macOS/iOS, an initially empty canonical system-library search list requests the respective SDK's `usr/lib`. In that same empty-list case, absent explicit `-target` requests the corresponding CPU target triple. Android requests its NDK system-library roots when needed and supplies a target triple unless `-target` is already present. Explicit native arguments are preserved; the module does not guarantee every equivalent combined argument spelling as suppressing those additions.
- Configured path fragments treated as non-system paths are lowercased for lookup. The normalization operation copies selected mutable arrays, not the entire transitive options graph; callers must not assume arbitrary original array/string backing is immutable or independently owned afterward.

Unavailable optional console-platform support returns failure. The normalizer does not otherwise reject ordinary nondefault visitors, macro transforms, naming/flattening rules, output options, vtable/bitfield controls, or supported model transformations merely because they differ from defaults. Downstream representation/native limits remain separate. It does not replace an explicitly null `get_func_args_for_printing` callback: the declared options default is `_default_get_func_args_for_printing`, but callers that overwrite it with null have no promised automatic fallback or successful printing result. Optional visitor, macro-conversion, and will-print callbacks remain optional as declared.

Direct `run_generator` consumes the supplied canonical options. It does **not** invoke `normalize_options`, translate the deprecated library/path arrays, add all normalizer defaults, or validate every option. A caller using direct init/run with legacy configuration must explicitly normalize and check its result first. Both public high-level generation workflows do that normalization before creating their canonical generator. No second parser/model path is part of either workflow's contract.

Library loading honors each configured search list and explicit suffix choices. Within a search location, a usable shared library/symbol view is preferred to a static fallback; Apple text stubs are an available shared-symbol source. Native-host loading and cross-generation symbol inspection are distinct domains. Empty identifiers are derived from the loader's extension-stripped filename spelling by lowercasing and the documented identifier-byte repair; this is not a promise of comprehensive path sanitization. Successful `Library_Info` records retain system/shared classification and generated path information. Symbol ownership uses the active generator's loaded-library order and returns the first match. Neither multiple matching libraries nor duplicate identifiers establish a new conflict-resolution policy. Cross-platform SDK, target suffix, and symbol-parser support is separate and not covered here.

`Library_Info.full_path` is the spelling used for a generated Jai library declaration, **not a promise of the absolute physical filename opened for symbol inspection**. For an ordinary library found through a search directory it contains the directory prefix plus the loader's extension-stripped base spelling; a system library uses the base spelling. Platform suffix/version stripping follows the loader's supported filename rules. Generated output must not replace that logical spelling with a resolved absolute, suffixed file merely because both refer to the same physical library. The physical inspection/load path, generated library name, and public identifier are separate values.

The emitted directive is `#library` with optional `,system`, `,no_dll` when dynamic availability is absent, and `,link_always` when requested. A shared library omits `,no_dll`; it does not add a `,shared` modifier. `,shared` is not accepted Jai syntax. `is_shared_library` therefore affects availability modifiers rather than prescribing that token or universal loader preference.

On macOS, generated bindings use relative extensionless library spellings, and the corresponding Windows static form adds `,no_dll`. For example, a directive whose literal is `./liboriginal_resolved` resolves to `liboriginal_resolved.dylib` next to the consumer when the working directory is that directory. This follows the documented relative logical-name shape without assuming absolute-path or explicit-suffix handling. Arbitrary working-directory/source-directory relocation, an absolute logical stem, and an absolute already-suffixed literal are unspecified. The generator's `full_path` field name alone implies no special handling.

`generate_bindings(options, log_results = false)` returns a `String_Builder` plus success. It normalizes, initializes one canonical generator, runs it, and invokes `will_print_bindings` once when configured after successful run preparation and before emitting binding declarations. It then emits the generated-file banner (optionally carrying a reduced generator command), caller header, visible flattened declarations with delayed global declarations, file-scope support, configured library declarations and generated checks/helpers, and caller footer. Generated command text avoids unnecessarily exposing the absolute generator executable path; it is not a promise to reproduce every compiler invocation byte-for-byte. The logical library spellings in generated ordinary declarations receive `generated_library_path_prefix`; system spellings do not. System/shared/link-always flags affect generated library directives as described above. Basic support is added when generated checks, 128-bit helpers, or formatting wrappers require it.

The returned builder's emitted text is the output artifact; the high-level call disposes registered native token sets and its translation unit/index, and releases its temporary model allocation pool before returning. This does not unload native libraries universally or release every module-level cache. Retaining model pointers obtained in visitors or the will-print callback beyond generation is unsupported. No valid persistent active generator is promised after that cleanup. Caller-owned callback objects/replacements are not implicitly transferred into a new permanent registry. `log_results` controls the generator result report, not all diagnostic/progress logging.

The filename overload defaults `log_results` to true, obtains the generated builder, and writes its contents to exactly the supplied filename only when generation succeeded. Failed generation or failed output writing returns false; successful file output returns true. It does not choose a target-dependent filename, create a compiled library, or automatically generate a bridge. The builder overload has no file-writing failure domain of its own. The high-level calls do not install a universal diagnostic-error accumulator: a diagnostic from a skipped/unsupported model or printer path is not, by itself, a documented false-return condition. Conversely, a true result is not proof that all declarations were represented or that emitted bindings compile/link. Native preparation failure and actual output failure remain explicit failure domains; silently claiming equivalence for an unsupported representation is not justified by this narrow boolean.

`generate_bridge(options, filename)` likewise normalizes and uses canonical init/run/deinit. It writes caller header first, adds the placement-new include when collected constructors require it, emits C-linkage adapters, and appends caller footer. It does not automatically include every configured source header: required C++ declarations belong in the caller's header text. It does not invoke `will_print_bindings`, emit the Jai banner/library declarations, produce a library binary, or use an automatically selected bridge output path. Private/protected/deprecated/operator functions are outside its emitted adapter selection, as are functions lacking required mapped library metadata. Successful native preparation and successful writing of the supplied output filename determine the return result; external C++ compilation/linking is caller work.

Bridge names use the visible-parent underscore composition and collision repair described for `append_cpp_wrapper`. Each generation call owns its own canonical model lifetime. Bridge generation does not automatically rewrite a later separate `generate_bindings` call's foreign symbols or declare a bridge-library identity. Jai declarations intended to call compiled adapters must be configured/generated against those actual exported names and ABI signatures by the caller's bridge workflow. The existing adapter-name rule establishes which symbol to link, not an automatic cross-call mapping registry.

### Macro-enum underlying type boundary

Creating a macro enum through `find_or_create_enum_for_macro` does not infer an underlying integer type. The run retains macro expression records and enum-typed member declarations; it does not implement a native integer-width/signedness join over the group, choose the first macro's SDK type, or validate lossless promotion of every mixed-sign value. A freshly generated macro enum can therefore have no explicit underlying CType and print ordinary Jai enum syntax without an explicit integer type. Reusing an existing enum retains that enum's represented underlying type; there is no automatic widening guarantee.

Equal-width grouping does not imply a required first-member type rule, and combining a negative signed value with a large unsigned value has no lossless join/ABI contract. Visitors can explicitly set a suitable enum type using the public model. Without additional policy, mixed-width/signedness overflow, noninteger values, and values beyond the literal payload domain remain explicit representation/emission limits. The native parser's success and the runner's true return do not guarantee that such a generated enum is valid or ABI-equivalent. An implementation must not silently truncate, invent a native type, or erase an accepted expression to claim success in that broader domain.

### `run_generator` completion and observable stage boundaries

`run_generator(generator, options)` installs the actual supplied generator and options as the active context and performs native parsing plus model preparation. It does not itself initialize all canonical primitive slots, emit the binding file, or dispose native resources; callers use the separate lifecycle operations for those responsibilities.

A failed configured library load, nonzero native translation-unit parse result, or native `Error`/`Fatal` diagnostic makes the boolean result false. Native warnings and lesser diagnostics alone do not. Configured ordinary libraries precede configured system libraries in the loaded library list; failed loads are diagnosed without discarding successfully loaded entries. Parsing uses configured include paths, system include paths, extra native arguments, source-file includes, optional comment parsing, and detailed preprocessing records. When preparation has failed in these domains, post-parse declaration visitors/transforms do not run.

For a successfully parsed input, user-visible ordering obligations are:

- Native cursor intake constructs canonical declarations and source metadata before macro validation and transformations.
- Global macros are validated before user declaration visitors see them. Rejected macros are removed from global scope; accepted macros can be marked for unwrapping or converted to configured enum groups. Converted enum entries retain their source expression/location/comment metadata. Macro-enum naming and visible namespace merging occur before user visitors.
- The configured `visitor_with_data` pass precedes the configured plain `visitor` pass. Each pass visits the current global declarations through ordinary declaration traversal. This ordering differs from the separately documented omission-wrapper callback composition.
- Redundant typedef/macro omission occurs after user visitors, so it reflects visitor-renamed declarations. Namespace emptiness and the final flattened global-scope member view also reflect visitor edits and omission/global-placement policies.

These are externally observable stage dependencies, not a required private algorithm. Derived `Struct_Extra_Info` construction remains on demand through the canonical metadata operation, including when printing requests it; `run_generator` does not promise an eager metadata pass before visitors. A visitor can request metadata itself. The public contract provides no general invalidation/rebuild rule when a visitor changes a model after its derived metadata has already been cached.

For `generate_enums_from_macros_with_prefixes`, a matching prefix supplies the enum source name after removing **all trailing underscores**, not the literal prefix including its delimiter and not the macro's full name. For example, prefix `HTTP_STATUS_` or `HTTP_STATUS___` selects source name `HTTP_STATUS`; a prefix with no trailing underscore keeps its full spelling. An empty result does not select an enum. Enum output-name normalization and subsequent enum/member naming policies still apply separately. A callback-selected enum name is its returned nonempty string; this prefix-trimming rule does not automatically apply to a callback return value.

The run's automatic macro-unwrapping policy is `macro_prefixes_to_unwrap`: a matching accepted macro gets `should_unwrap = true` and `OMIT_FROM_OUTPUT`. Lookup can still resolve such an omitted macro and substitute its converted value text rather than its declaration output name. The macro is not simply deleted as an invalid macro. No additional built-in macro-name suffix, typedef-prefix, or identifier-category rule automatically sets `should_unwrap`. Typedef unwrapping is a separate type policy; redundant/no-op macro omission and rejected macro validation are separate operations. Marking a macro for unwrapping does not by itself exclude it from configured macro-to-enum conversion. Explicit user mutation of public fields remains distinct from automatic run policy.

The boolean is not a universal count of every later model diagnostic. Unsupported constructor results, discarded macros, and diagnostics emitted during transforms do not independently establish a false return rule. A true result means the run completed its native preparation and passes; it does not certify complete representation of every native declaration, ABI correctness, successful emission, or successful file writing. A failed run can retain loaded libraries/native handles and partial state for lifecycle cleanup; atomic rollback and safe repeated runs on an uncleared generator are not promised. Invalid/null native handles following parser failure are not an established usable domain.

Native declaration traversal receives the context captured for that traversal. `handle_toplevel_cursors` activates that captured context for its callback and restores the incoming callback context afterward. Native child enumeration similarly uses its captured context for context-dependent operations. This guarantees the intended active generator/options during synchronous native callbacks; it is not a global mutable-context registry or a guarantee for arbitrary asynchronous callbacks after the captured stack/session has ended. Direct user visitor/callback calls run under the active run context; the API does not promise a separate snapshot/restore around every user callback. `run_generator` itself leaves its installed active generator/options in place. Caller `push_context` or equivalent scoped activation controls restoration of an outer session.

### Paths, names, and diagnostics

`append_parent_path` produces outer-to-inner `output_name` components, each followed by the supplied separator. Null contributes nothing and returns false. A declaration equal to the current scope or one of its ancestors contributes nothing. Flattened namespaces are skipped when `skip_flattened_namespaces` is true; disabling that option retains their names for native C++ qualification. Other declaration categories are not filtered merely by kind. An empty `output_name` emits no component, but a processed non-flattened declaration still makes the result true. Thus the boolean is not an exact byte-count test. The diagnostic `location` does not guarantee an escaping, collision, or anonymous-name repair policy.

`append_parent_field_path` is for anonymous record containment below the current scope, not arbitrary namespace/type qualification. Its supported input ancestors are anonymous structs. Components use the actual declaration referring to each anonymous struct in the parent's derived member/overlay model, followed by the separator. An unnamed containing field contributes no component; processed anonymous containment still returns true. Missing containing declarations are diagnostic/assertion failures, not invented field paths.

`stripped_name` removes the first configured prefix that matches the original name. After removal, leading underscores are trimmed only while another valid identifier-start byte follows; an otherwise invalid initial byte gets a leading underscore. With no matching prefix or an empty input the spelling is unchanged. It does not apply general reserved-name normalization itself.

`avoid_name_clash` mutates `output_name`, applies reserved-name normalization, and consults the supplied declaration slice with the documented function-equivalence/overload policy. The exact output `_` is exempt and is not inserted. A function matching a global struct's output name gets a trailing underscore even when no declaration slice is supplied. Colliding functions acquire `ADD_C_TYPE_DETAILS` on both signatures. With `try_parent_prefix`, names can acquire ancestor output-name prefixes separated by underscores; remaining conflicts use positive numeric suffixes `_1`, `_2`, and so on. Operator names use the existing safe operator-name normalization before numeric suffixing. `insert` controls unique pointer insertion into the supplied slice; it does not disable name repair. There is no general deduplication of equivalent wrapper implementations hidden in this helper.

`should_be_omitted` tests the declaration's omission flag and the supplied parent's ancestor chain. It does not infer ancestors from a different scope or increment strip counts. `should_unwrap_typedef` uses exact built-in unwrap names, built-in unwrap prefixes, the `__..._t` spelling convention, and configured `typedef_prefixes_to_unwrap`. Its cursor overload uses cursor spelling. It is a name policy, not an automatic release or recursive type rewrite.

`omit_unnecessary_typedefs_and_macros` marks a typedef omitted when its printed underlying type equals its output name, and a macro omitted when its printed expression equals its output name. Namespaces and structs return `RECURSE`; other kinds return `STOP`. It does not increment the stripped counter or provide additional semantics through `user_data`.

`as_str(CXCursor)` returns copied cursor spelling; `all_info` adds cursor kind and type to a diagnostic description. `as_str(CXType)` returns copied native type spelling. `to_string(Location)` formats cached filename, line, and column separated by colons. Filename lookup uses the module's lowercased filename cache. Cursor/location `log` overloads prefix message text with the native filename/line/column, while passing the caller-supplied `loc` to ordinary logging as `Log_Info.location`. They preserve ordinary flags/user flags; `log_error` adds `ERROR`. Native declaration location in the text does not replace caller location in logging metadata. No special unset-location placeholder or safe null-file behavior is specified.

### Array conversion and type queries

`maybe_convert_fixed_array_type` implements parameter normalization. A fixed array, including one behind a typedef, returns a new pointer to the original supplied type marked `INSERTED`, with `changed = true`. An explicit dimension expression also qualifies. An incomplete outer array can retain its shape while its element type undergoes this conversion. `va_list` is exempt. Ineligible inputs return the original pointer and false. The helper does not mutate a shared canonical input or rewrite declaration flags.

`maybe_convert_pointer_type_to_fixed_array_type` recovers a fixed array when an eligible pointer cursor's final-child expression is a positive signed or unsigned integer literal. The result has the pointer's pointee as its element type, the recovered count, and product size. Ineligible inputs, nonliteral counts, and nonpositive counts preserve the original pointer. It creates a separate type rather than changing the original pointer object; arbitrary original qualifiers/alignment are not promised to be copied. Out-of-range counts and arithmetic overflow are unsupported.

Eligibility is evaluated only on function-argument declarations by the declaration-construction path. The helper itself has no `.INSERTED`-flag exclusion: it tests for a non-null pointer type and a final-child expression that is a positive signed or unsigned integer literal. It also has no pointer-to-array exclusion; the recovered element type is exactly the pointer's immediate pointee, even when that pointee is itself an array type. The subsequent fixed-array parameter normalization is a separate step.

`type_has_name` is narrowly false for a directly represented struct with empty source `name`; it is true for other valid CType categories, including numeric and enum types. It is not a general test for printable nominal syntax. `get_type_name` returns the selected represented declaration's source or output name; primitive and hardcoded text alone do not supply a declaration name. `strip_pointers` removes pointer layers, not typedefs or array layers. `try_find_struct_in_type` follows direct struct, array-element, and pointer links; it does not promise typedef, enum, or function-signature traversal. `type_to_string` composes the Jai type printer with the supplied scope and flags.

`size_from_number_flags` recognizes the 8-, 16-, 32-, and 64-bit flags as one, two, four, and eight bytes; with multiple such bits, the smallest recognized width wins. No recognized width returns zero. The existence of canonical 128-bit types does not add a 128-bit branch to this helper's behavior.

### Bitfield records and accessors

`maybe_add_to_bitfield` mutates `Bitfield_Status` and preserves each original field pointer in the group's `members`. A non-bitfield declaration (`bitfield_width == -1`) or unsupported primitive storage size terminates the active group and returns that declaration unchanged. A zero-width field returns null without itself clearing the active group. The first positive-width field of a new group returns the new `Bitfield`; additional fields merged into that group return null. The group records the first field's source name/location/bit offset, the status parent, the storage output name `__bitfield`, and the configured helper-generation flag. Non-Windows grouping does not require equal primitive types; Windows grouping uses represented underlying storage compatibility. Neither grouping nor a zero-width field independently fabricates padding that is absent from native offsets.

`finish_bitfield` derives the occupied byte span from the group's first offset through its last member's end, and records the largest member storage type as the alignment type when needed. A span fitting the alignment storage can use that storage directly; non-Windows smaller spans can use a smaller unsigned scalar rounded to a supported width. A larger span uses a byte array with byte alignment and retains separate alignment information. This updates the existing group's type/alignment metadata. The supplied `index` has no observable naming effect in this helper. Empty groups and invalid spans are unsupported inputs.

`maybe_unwrap_bitfield_declaration` returns a one-member bitfield's original member pointer after assigning the group's storage type to that member. Other inputs return the original declaration pointer. This is a visible member-type mutation, not a new declaration or copy.

`append_bitfield_helpers` emits getters and setters for named group members. Names are `get_`/`set_` followed by anonymous-container field prefixes joined by underscores and then the member output name. The setter takes `fields: *Scope` and a value; the getter takes `fields: Scope` by value and returns the value. One-bit values use bool; wider values use the member's represented type. Setters update only the member's bit range, preserving unrelated bits; byte-array groups can span elements. Getters recover the masked bit range. The emitter does not apply an additional narrow signed-field sign-extension operation; full C/C++ signed-bitfield equivalence must not be inferred merely from a signed return type. Unnamed members produce no accessors.

### Wrappers and printing callbacks

`_default_get_func_args_for_printing` returns `Function_Type.arguments` for all modes. `print_function_type_to_builder` invokes the configured callback in `FOREIGN_FUNC_DECLARATION` mode when supplied a `Function`; a standalone function type uses its arguments directly. The callback result supplies the actual foreign declaration parameters. A wrapper uses the callback's `OUTER_WRAPPER` result for its exposed parameter list and `INNER_WRAPPER_CALL` result for argument names/ordering in the call. `POINTER_TO_VALUE` affects foreign parameter flags and inner-call addressing; these callback outputs must not be ignored or replaced with an independent model.

`print_function` emits the foreign declaration and, when required, an argument or formatting wrapper. Non-formatting variadic functions requiring argument wrappers diagnose unsupported automatic wrapping. A formatting wrapper uses a `_CFormat` foreign declaration, exposes a string plus variadic Any arguments, and marks that formatting support was generated. The exact virtual foreign-declaration prefix is `virtual_`. Thus a function whose output name is `step` has a virtual foreign declaration named `virtual_step`, or `virtual_step_CFormat` when the formatting suffix applies. Objective-C methods use the represented selector and intrinsic message-send path; this does not guarantee ABI correctness on every Apple target.

For an ordinary argument wrapper without formatting, the exposed wrapper and foreign declaration use the same base output name; the API does not prescribe a `_raw`, `_foreign`, or similar additional rename. Formatting uses the base name for the exposed wrapper and base plus `_CFormat` for its inner foreign declaration. `OUTER_WRAPPER` callback declaration output names are authoritative for exposed parameters; `FOREIGN_FUNC_DECLARATION` names are authoritative for foreign parameters; `INNER_WRAPPER_CALL` names supply the actual inner call expressions, with represented pointer-to-value addressing. The default callback returns the original argument objects and therefore does not invent parameter renames. A formatting wrapper additionally exposes the generated variadic argument name `__args`; its format-string argument name comes from the callback. These rules do not promise that every combination of virtual/formatting/custom callback flags produces a valid collision-free overload set; unsupported combinations need explicit verification rather than invented hidden names.

For the C++ bridge, generated parameter base names are `out_return_value`, `message_count`, and `message_data` where the corresponding result/formatting transformation is required. They undergo ordinary collision repair against bridge arguments, so suffixes can change their final output names. Other bridge parameters retain their represented output names with the documented C++ keyword normalization. No stronger universal exposed/foreign parameter-name identity is guaranteed when callbacks or collision repair change names.

`append_cpp_wrapper` emits a C++ bridge and mutates `func.output_name` to include visible parent prefixes, plus `_message` for formatting bridges. `existing_declarations` participates in collision repair with overloads disabled and records the function pointer; it is not a cache that silently suppresses equivalent output. By-value struct results use an explicit output pointer and a void bridge return; reference returns become pointer-shaped results. Method/destructor bridges consume the leading instance parameter as receiver; constructors use placement construction. The bridge's formatting variant accepts message count/data. Its ordinary argument list comes from the represented function arguments rather than the Jai printing callback. Default expressions that cannot form a valid native default sequence are not emitted as active defaults. General unsupported native expressions must remain visible failures/limitations.

The bridge's transformed C ABI must be paired with a Jai foreign declaration whose signature describes that ABI. `generate_bindings` independently prints the represented C++ declaration; it does not rewrite its aggregate-by-value result to match the bridge's output-pointer form. A caller that consumes a bridge must arrange a matching Jai declaration separately. The C++ bridge path also does not invoke `get_func_args_for_printing`; callback counts or argument reordering from that callback apply to ordinary binding/wrapper printing, not bridge generation.

The C++ bridge collector skips declarations recognized as C++ operators before calling `append_cpp_wrapper`; consequently it does not emit an operator bridge or specify an operator bridge symbol spelling. `append_cpp_wrapper` itself composes the existing `func.output_name` with visible parent names and does not call `replace_operator_name`. Elsewhere, declaration collision handling calls `avoid_reserved_names`, which uses `replace_operator_name(..., keep_operators_allowed_in_jai = true)` for operator declarations; this Jai declaration-name handling does not imply a C bridge export.

`print_function` emits a virtual function declaration with the `virtual_` prefix and the represented function type. It routes that declaration through ordinary foreign-linkage emission; when no library is available, the linkage helper reports an error and uses the `__UnknownLib` placeholder. The separate generated vtable record contains ordered function-pointer slots. Optional C-style helpers, when enabled, dispatch through the instance's vtable pointer and the matching slot; they are not the `virtual_` declaration. A missing library does not guarantee that the direct declaration names an available callable implementation.

`append_elsewhere` emits `#foreign` for functions and `#elsewhere` otherwise, using the declaration's library identifier. A different nonempty foreign name is emitted as a quoted symbol name. A missing library produces a diagnostic and the visible `__UnknownLib` placeholder; that output is not a successful resolved linkage contract.

`append_template_instantiation_params` emits nothing for an empty list. Otherwise it emits parenthesized comma-separated expression arguments, using a corresponding formal declaration's type as target type when available. `append_cpp_type_modifiers` emits const only with `C_TYPE_DETAILS`; a reference marker is independent of that flag. It does not independently emit volatile or reconstruct arbitrary qualifier placement.

`print_enum` prefixes `using` when `c_enum_emulation` is enabled and emits the enum output name plus its body. `print_enum_body` emits enum/enum_flags with its represented underlying type, omits flagged/empty-name enumerators, uses enum target type and optional hex values, and increments the generator's enum count. `ALIAS_ORIGINAL_NAMES` emits original-name aliases only for changed, nonomitted, nonreserved names. `print_namespace` emits the namespace as a struct, skips omitted and empty nested namespaces, and queues declarations requiring global placement. `print_typedef` emits an ordinary alias or the represented distinct/isa modifier; simultaneous distinct and isa flags are invalid. `print_struct` consumes canonical `modified_declarations` and extra-info vtable/layout metadata as specified above. These are printing effects, not new model-construction registries.

#### Incomplete native records

A Jai empty record declaration such as `Opaque :: struct {};` is valid syntax and can provide a nominal pointee type for a native incomplete struct or union. It is only an opaque pointer marker: the empty Jai record says nothing about the native record's size, alignment, fields, or by-value calling convention. A generated binding must not claim by-value ABI support for such a record; by-value parameters, results, fields, and arrays require a diagnosed unsupported case unless layout information is available from elsewhere. The public `print_struct` signature does not provide a separate native-completeness flag or a graph-wide usage contract, so whether the printer itself can reject every by-value use is unspecified.

For a canonical native `CType`, the exact Clang layout result `size == -2` and `alignment == -2` is `CXTypeLayoutError.Incomplete`; it is distinct from invalid and dependent-layout errors. This sentinel, together with a canonical struct/union model that has no member declarations, no template type or instantiation parameters, no parents, virtual functions, or overlays, is the supported incomplete-record recognition rule. Keep all other negative/mismatched layout results and models with declarations or template/dependent metadata on the diagnostic path. If the model did not obtain the values from the canonical native layout query, the sentinel alone is not proof. An incomplete record may be emitted as the named empty Jai marker for pointer references only. Use by value in a parameter, result, field, or array has no native layout and must be rejected by generation/validation rather than emitted as ABI-compatible. Explicit nonzero alignment and visitor-supplied fields, parents, vtable entries, or overlays must likewise be rejected for this record; they would assert layout/content absent from the native declaration.

`print_declaration_to_builder` dispatches the represented declaration kind through the canonical function, enum, typedef, struct, namespace, macro, bitfield, or ordinary declaration printer. It honors indentation, comment, alignment, qualifier, default-expression, and semicolon flags. Function/struct emission increments the corresponding generator count. A helper-enabled multi-member bitfield is queued for accessor emission; a single-member group has the identity/type mutation described above. `size_condition_type` wraps the represented declaration in a compile-time zero-size condition.

For an ordinary declaration, `POINTER_TO_VALUE` exposes the pointee unless `KEEP_POINTER_TO_VALUE` requests its foreign representation. `NO_ARGUMENT_DEFAULTS` suppresses the initializer and prevents bool type inference. Constants outside argument lists use constant declaration syntax; arguments use argument/default syntax even when const. `NEEDS_DEFAULT_ARGUMENT_CAST` adds a represented-type cast unless expression printing reports that it already emitted enum conversion. Foreign linkage is appended for a nonempty foreign name when the declaration has no expression or is const. Explicit alignment and positive bitfield-width annotations are printed. Unsupported declaration kinds diagnose; omission filtering belongs to callers rather than a universal early-return promise here.

`append_cpp_type` uses native declaration names with C++ path separators, retaining flattened namespaces unless `NO_PATH` suppresses qualification. Represented reference parameters use reference syntax; reference returns are pointer-shaped for bridges. `INSERTED` pointer layers suppress their extra native marker. Native array types print as element pointers in this helper rather than reconstructing postfix C array declarators. These supported transformations do not establish arbitrary declarator, method-pointer, or generic ABI completeness. `print_type_to_builder` uses Jai output names, array dimensions/incomplete-array pointer form, represented aliases and template expressions, and standalone function-type syntax; it can delegate anonymous aggregates to their canonical printers. A configured va_list strip policy can replace a function type containing va_list with an explicit opaque-pointer diagnostic representation.

### Macro and expression APIs

`get_scoped_token_string` is a macro exposing the token spelling for the caller's expansion scope, with native string disposal tied to that scope. The returned view must not be treated as permanent owned storage. `raw_string_from_macro_tokens` copies spellings into a string separated by one space between tokens, independent of original spacing. `expect_tokens` compares consecutive token spellings at a valid nonnegative start index; insufficient remaining tokens return false. Negative indices are unsupported.

`check_macro_value` requires a macro literal declaration in `NEEDS_CHECKING` and changes its state to `KEEP` or `DISCARD`. Its supported domain is resolvable object-like expression macros, not a general C preprocessor evaluator. Unresolved/self identifiers, unsupported literal conversion, braces/comma/continuation syntax, detected prefix address-of, identifier-member syntax, empty-call syntax, and unsupported adjacent identifier/string concatenation cause discard. A converted value identical to the macro's name is also discarded. Diagnostics for unsupported forms follow `log_unsupported`. The helper does not guarantee general recursive-macro termination or full C expression typing.

`string_from_macro_value` converts a previously checked macro into Jai expression text. It preserves source token separation where extents are nonadjacent, converts supported literal spellings, resolves identifiers to output names even when later user omission hides them, qualifies cross-enum members, and removes one enclosing parenthesis pair only when it encloses the entire expression. Comments have normalized line endings. It relies on prior validation; it is not a safe fallback parser for unknown identifiers.

`maybe_append_cast` recognizes supported parenthesized type-identifier casts with optional pointer stars and emits Jai cast text. It reports success plus the final consumed token index; an unsupported shape reports false and -1. A function-pointer cast uses the opaque pointer representation. A cast matching the supplied enum's underlying type can be consumed without emitted text. Applicable negative/complement numeric cases use truncating casts. No general C declarator parser or equivalent bridge ABI follows from these recognized forms.

`parse_call_expression` supports constructor-style/aggregate initializer conversion, not arbitrary runtime function calls. An initial TypeRef is consumed when present. With `expect_type_ref`, native null can produce a null literal, ordinary literal initializer children can be accepted without a TypeRef, and a native unary sizeof-like expression may be evaluated as a literal. Unsupported member/global references, nested complex call/lambda/namespace/template-reference shapes, and unsupported unary-address forms return null. A supported aggregate has the public cursor-derived `type_name` and converted child values; any unrepresentable child fails conversion. No hidden CType field or dropped template argument is authorized. Empty/malformed native child lists are unsupported.

Dependent array bounds are represented by `CType.array_element_count_expression` when an appropriate native child expression is available and convertible. There is no complete macro spelling/expansion extent recovery contract, and it does not require reconstruction of a missing/ambiguous native bound from unrelated declarator text. A precisely diagnosed unsupported macro extent is therefore an explicit accepted-domain limit; it must not be described as a rejection rule. Ordinary convertible dependent expressions remain supported and must retain the correct bound for their own array layer.

Runtime C `VariableArray` is not an implemented conversion category in the type constructor. No obligation to represent its non-const runtime bound through `create_reference_expression` follows: that helper's const-reference eligibility remains in force. A native VariableArray can return null with an unsupported-kind diagnostic. It must not be silently claimed equivalent to an incomplete array or a guessed fixed count. This limit is distinct from supported `DependentSizedArray` expressions referring to convertible template/dependent constants. General evaluation of runtime bound expressions or a new public runtime-expression record is not specified.

`convert_children_to_expression` uses the final native child and returns null for no usable child. `fill_template_arguments` appends represented type-reference or supported expression arguments and reports the supplied-child consumption count; failure is false with zero count, not an atomic-rollback guarantee for a caller's already mutable array. `fill_template_arguments_for_expression` requires an identifier literal, appends supported references/expressions, and rejects ambiguous nested TemplateRef grouping. Its public unsupported-child placeholder is diagnostic representation, not proof that the argument was converted equivalently. General packs/dependent defaults remain limited as documented above.

`handle_toplevel_cursors` uses its supplied context and native system-header classification. Configured path fragments and included system names can exempt declarations. Kept ignored-system declarations are marked omitted and counted; linkage/unexposed container cursors recurse. Equivalent global functions are marked omitted. Equal macro redeclarations are not duplicated; a differing macro redeclaration replaces the prior global entry with a warning. Canonical declaration constructors remain the single source of represented declarations.

System-header exemption is checked for the cursor delivered to `handle_toplevel_cursors`, using that cursor's spelling for `system_types_to_include`. Namespace construction consumes its own children with the inherited `ignore_non_structs` value; it does not independently reapply that include-name list to each child. Consequently, listing a nested type's name alone does not rescue it from an ignored enclosing system namespace. Listing the namespace itself, or exempting its source path, permits ordinary namespace construction including its children. This is not a recursively applied name allowlist. Native linkage/unexposed containers that return `Recurse` instead deliver their children through the callback, where those children's own filtering is applied.

Filtering affects construction as well as output: an ignored cursor is passed to `create_declaration` with `ignore_non_structs = true`. That mode suppresses function, variable/field, and macro-definition construction, while keeping supported type declarations and namespaces; namespaces propagate it to their namespace children. A kept ignored declaration is additionally marked `OMIT_FROM_OUTPUT` and counted. There is no requirement to create a hidden function/variable model first and merely omit it afterward. Conversely, this mode does not suppress every member inside a kept record: record construction has its ordinary member-construction contract. The omitted enclosing declaration still controls ordinary output eligibility. Arbitrary separately invoked constructors do not independently acquire top-level system filtering merely because their native cursor came from a system file.

### Extending and configuring these helpers

Change the corresponding public helper and shared model/printer contracts together. Independent regressions should inspect pointer identity, callback mode/results, omission state, generated signatures, and ordinary native behavior rather than restating private implementation steps. Relevant options include target OS/ABI, prefix and typedef unwrapping policies, bitfield helper generation, callbacks, strip flags, comment/spacing flags, and system-header inclusion. Unsupported native representation domains must not be claimed compatible; they do not justify leaving ordinary documented helper calls absent.

Platform coverage note: exports for Nintendo and PS5 targets are unspecified.
