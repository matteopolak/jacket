// Public data contract from the specification; private algorithms are independent.

Generate_Bindings_Options :: struct #type_info_none {
    os := TARGET_OS;
    cpu := CPU;
    library_search_paths: [..] string;
    libraries: [..] Library;
    system_library_search_paths: [..] string;
    system_libraries: [..] Library;
    libnames: [..] string;
    libpaths: [..] string;
    system_library_names: [..] string;
    system_library_paths: [..] string;
    static_library_suffix: string;
    shared_library_suffix: string;
    system_include_paths: [..] string;
    include_paths: [..] string;
    source_files: [..] string;
    extra_clang_arguments: [..] string;
    path_fragments_to_treat_as_non_system_paths: [..] string;
    system_types_to_include: [..] string;
    log_ignored_system_declarations := false;
    flatten_namespaces: [..] string;
    default_namespace_flags: Declaration_Flags;
    strip_prefixes: [..] string;
    generate_enums_from_macros_with_prefixes: [..] string;
    convert_macro_value_to_enum_callback: #type (decl: *Declaration) -> enum_name : string;
    strip_enum_prefixes := true;
    auto_detect_enum_prefixes := true;
    enum_prefix_delimiter := "_";
    alias_original_enum_names := true;
    strip_enum_values_with_suffix: string;
    strip_enum_value_prefixes: [..][2] string;
    typedef_prefixes_to_unwrap: [..] string;
    macro_prefixes_to_unwrap: [..] string;
    move_struct_functions_to_global_scope := false;
    omit_global_declarations := false;
    strip_flags := Strip_Flags.FUNCTIONS_WITH_VALIST | .SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS;
    log_stripped_declarations := true;
    try_to_preserve_comments := true;
    generate_library_declarations := true;
    generated_library_path_prefix := "";
    add_generator_command := true;
    header := "";
    footer := "";
    c_enum_emulation := false;
    prepare_all_structs_for_inheritance := false;
    generate_compile_time_struct_checks := true;
    generate_printf_wrappers := true;
    generate_c_style_api_for_vtable := true;
    generate_vtable_helpers := true;
    generate_bitfield_helpers := true;
    add_overloaded_const_and_ref_comments := true;
    add_placeholders_for_unresolved_types := true;
    mimic_spacing_flags := Mimic_Spacing_Flags.ALL;
    get_func_args_for_printing: Get_Function_Arguments_For_Printing_Func = _default_get_func_args_for_printing;
    will_print_bindings: #type ();
    visitor: Declaration_Visitor;
    visitor_with_data: Declaration_Visitor_With_Data;
    visitor_data: *void;
    log_unsupported := false;
}

Library :: struct {
    filename: string;
    identifier: string;
}

Library_Info :: struct {
    identifier: string;
    full_path: string;
    #if OS == .WINDOWS {
        handle: W.HMODULE;
    } else {
        handle: *void;
    }
    is_shared_library: bool;
    is_system_library: bool;
    symbol_names: [] string;
    flags: Flags;
    Flags :: enum_flags {
        LINK_ALWAYS
    }
}

Bridge_Data :: struct {
    functions: [..] *Function;
    needs_placement_new: bool;
}

Cpp_Functions_Visitor_Data :: struct {
    visitor: Declaration_Visitor;
    visitor_with_data: Declaration_Visitor_With_Data;
    visitor_data: *void;
}

#scope_file
#if OS == .WINDOWS W :: #import "Windows";
