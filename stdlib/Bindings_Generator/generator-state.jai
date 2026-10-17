// Single canonical storage, from the public type inventory.
Generator :: struct #type_info_none {
    global_scope: Block;
    pointer_size: s64;
    anon_declarations: [..] *Declaration;
    type_def_void: *CType;
    type_def_bool: *CType;
    type_def_char: *CType;
    type_def_u8: *CType;
    type_def_u16: *CType;
    type_def_u32: *CType;
    type_def_u64: *CType;
    type_def_u128: *CType;
    type_def_s8: *CType;
    type_def_s16: *CType;
    type_def_s32: *CType;
    type_def_s64: *CType;
    type_def_s128: *CType;
    type_def_float32: *CType;
    type_def_float64: *CType;
    type_def_pointer_to_void: *CType;
    type_def_ptr_void: *CType;
    type_def_objc_class: *CType;
    type_def_objc_selector: *CType;
    type_def_objc_block_pointer: *CType;
    type_def_objc_id: *CType;
    type_def_jai_string: *CType;
    type_def_type: *CType;
    type_def_msvc_destructor_flags: *CType;
    declarations_by_cursor: Table(CXCursor, *Declaration, get_hash_cxcursor, compare_cxcursors);
    macro_token_replacements: [..] Replacement;
    Replacement :: struct {
        location: Location;
        replacement: string;
    }
    enum_macros: [..] *Enum;
    macros_converted_to_enum_values: Table(string, *Enum);
    unwrapped_typedefs: Table(string, *Typedef);
    extern_declarations: [..] *Declaration;
    unique_objc_selectors: [..] string;
    libraries: [..] *Library_Info;
    com_uuids: [..] ComUuid;
    extra_infos: Table(*Struct, Struct_Extra_Info);
    pending_bitfields: [..] *Bitfield;
    structs_to_check: [..] *Struct;
    flattened_global_scope_members: [] *Declaration;
    declarations_for_global_scope: [..] *Declaration;
    int128_was_referenced: bool;
    generated_printf_wrapper: bool;
    tab_depth := 0;
    template_depth := 0;
    clang_index: CXIndex;
    translation_unit: CXTranslationUnit;
    tokens_to_free: [..] Token_Info;
    filenames: Table(CXFile, string);
    counts: struct {
        ignored_system_declarations: u32;
        stripped: u32;
        functions: u32;
        structs: u32;
        enums: u32;
    }
}

Token_Info :: struct {
    token_pointer: *CXToken;
    num_tokens: u32;
}

Library_Symbols :: struct {
    os: Operating_System_Tag;
    cpu: CPU_Tag;
    full_path: string;
    symbol_names: [] string;
}

struct_extra_info_unknown := Struct_Extra_Info.{ largest_natural_alignment = -1, struct_alignment = -1 };

#scope_file
#import "Hash_Table";

get_hash_cxcursor :: (cur: CXCursor) -> u32 {
    return clang_hashCursor(cur);
}

compare_cxcursors :: (a: CXCursor, b: CXCursor) -> bool {
    return clang_equalCursors(a, b) != 0;
}
