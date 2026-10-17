// Private bridge protocol implemented by this repository's Rust compiler.
__jaic_workspace_create :: (name: string) -> Workspace #compiler;
__jaic_current_workspace :: () -> Workspace #compiler;
__jaic_workspace_validate :: (w: Workspace) #compiler;
__jaic_data_segment :: (section_name: string, characteristics: u32, alignment: s32, w: Workspace) -> *void #compiler;
__jaic_global_data :: (w: Workspace, data: [] u8, segment: u16, user_segment: *void, result: *[] u8) #compiler;
__jaic_runtime_data :: (w: Workspace) -> *void #compiler;
__jaic_workspace_add_file :: (w: Workspace, filename: string) #compiler;
__jaic_workspace_add_string :: (w: Workspace, source: string) #compiler;
__jaic_workspace_add_string_to_module :: (w: Workspace, source: string, message_record: s64) #compiler;
__jaic_workspace_add_string_to_code :: (w: Workspace, source: string, code: Code) #compiler;
__jaic_workspace_destroy :: (w: Workspace) #compiler;
__jaic_workspace_set_text :: (w: Workspace, option: Workspace_Text, value: string) #compiler;
__jaic_workspace_set_flag :: (w: Workspace, option: Workspace_Flag, value: bool) #compiler;
__jaic_workspace_set_number :: (w: Workspace, option: Workspace_Number, value: s64) #compiler;
__jaic_workspace_clear_list :: (w: Workspace, option: Workspace_List) #compiler;
__jaic_workspace_append_list :: (w: Workspace, option: Workspace_List, value: string) #compiler;
__jaic_workspace_options_snapshot :: (w: Workspace, options: *void, size: s64, strings: [] s64, arrays: [] s64, save: bool) -> bool #compiler;
__jaic_workspace_begin_intercept :: (w: Workspace, flags: u32) #compiler;
__jaic_workspace_end_intercept :: (w: Workspace) #compiler;
__jaic_workspace_status :: (w: Workspace, status: u8) #compiler;
__jaic_workspace_next_event :: (w: Workspace) -> s64 #compiler;
__jaic_event_int :: (index: s64) -> s64 #compiler;
__jaic_event_string :: (index: s64, result: *string) #compiler;
__jaic_command_line_count :: () -> s64 #compiler;
__jaic_command_line_arg :: (index: s64, result: *string) #compiler;
__jaic_report :: (message: string, filename: string, line: s64, character: s64, mode: u8) #compiler;
__jaic_compiler_version :: (result: *string) #compiler;
__jaic_custom_link_complete :: (w: Workspace, exit_code: s64) #compiler;
__jaic_code_is_null :: (code: Code) -> bool #compiler;
__jaic_code_nodes :: (code: Code) -> s64 #compiler;
__jaic_parse_code :: (text: string, scope_from: Code) -> Code #compiler;
__jaic_modify_procedure :: (w: Workspace, body_record: s64, records: *s64, source_strings: *string, count: s64) #compiler;
__jaic_set_type_info_flags :: (type: Type, flags: u32) #compiler;
__jaic_struct_location :: (info: *Type_Info_Struct, path: *string, line: *s64, character: *s64) #compiler;
__jaic_base_path :: (result: *string) #compiler;
__jaic_type_table :: (w: Workspace, result: *[] *Type_Info) #compiler;
__jaic_add_library_dir :: (path: string) #compiler;
__jaic_workspace_remap_import :: (w: Workspace, host: string, name: string, replacement: string) #compiler;
__jaic_workspace_provide_import :: (w: Workspace, host: string, name: string, kind: u8, value: string) #compiler;
__jaic_make_procedure_live :: (w: Workspace, header_record: s64) #compiler;
__jaic_rec_builder :: () -> u64 #compiler;
__jaic_rec_type :: (builder: u64, name: string, info: *Type_Info) #compiler;
__jaic_rec_builtin :: (builder: u64, name: string, info: *Type_Info) #compiler;
__jaic_rec_plan :: (builder: u64, info: *Type_Info, size: u64, entries: *void, count: u64, is_type_info_struct: bool) #compiler;
__jaic_rec_build :: (builder: u64, record: s64, expected: *Type_Info) -> *void #compiler;
__jaic_rec_of :: (builder: u64, pointer: *void) -> s64 #compiler;
__jaic_rec_ref :: (record: s64, field: string) -> s64 #compiler;
__jaic_rec_int :: (record: s64, field: string) -> s64 #compiler;
__jaic_rec_count :: (record: s64, field: string) -> s64 #compiler;
__jaic_rec_item_ref :: (record: s64, field: string, index: s64) -> s64 #compiler;

// Options a workspace's build settings hold, by the kind of value they take. The Rust compiler
// declares the same members in `crates/jaic/src/builtin_keys.rs`; keep both in step.
Workspace_Text :: enum u16 {
    OUTPUT_EXECUTABLE_NAME :: 0;
    OUTPUT_PATH :: 1;
    INTERMEDIATE_PATH :: 2;
    ENTRY_POINT_NAME :: 3;
    LLVM_TARGET_SYSTEM_TRIPLE :: 4;
    LLVM_TARGET_SYSTEM_CPU :: 5;
    LLVM_TARGET_SYSTEM_FEATURES :: 6;
    MINIMUM_OS_VERSION :: 7;
}

Workspace_Flag :: enum u16 {
    DO_OUTPUT :: 0;
    STACK_TRACE :: 1;
    DISABLE_REDZONE :: 2;
    ENABLE_FRAME_POINTERS :: 3;
    APPEND_EXECUTABLE_FILENAME_EXTENSION :: 4;
    USE_CUSTOM_LINK_COMMAND :: 5;
    OUTPUT_BITCODE :: 6;
    OUTPUT_LLVM_IR :: 7;
    OUTPUT_BITCODE_BEFORE_OPTIMIZATIONS :: 8;
    OUTPUT_LLVM_IR_BEFORE_OPTIMIZATIONS :: 9;
    LLVM_ENABLE_TAIL_CALLS :: 10;
    LLVM_ENABLE_LOOP_UNROLLING :: 11;
    LLVM_ENABLE_SLP_VECTORIZATION :: 12;
    LLVM_ENABLE_LOOP_VECTORIZATION :: 13;
    LLVM_PRESERVE_DEBUG_INFO :: 14;
    LLVM_MERGE_FUNCTIONS :: 15;
    LLVM_DISABLE_INLINING :: 16;
    LLVM_DISABLE_MEM2REG :: 17;
    LLVM_ENABLE_SPLIT_MODULES :: 18;
    LLVM_FUNCTION_SECTIONS :: 19;
}

Workspace_Number :: enum u16 {
    OUTPUT_TYPE :: 0;
    OS_TARGET :: 1;
    CPU_TARGET :: 2;
    TEMPORARY_STORAGE_SIZE :: 3;
    ARRAY_BOUNDS_CHECK :: 4;
    CAST_BOUNDS_CHECK :: 5;
    NULL_POINTER_CHECK :: 6;
    ARITHMETIC_OVERFLOW_CHECK :: 7;
    DEAD_CODE_ELIMINATION :: 8;
    RUNTIME_SUPPORT_DEFINITIONS :: 9;
    BACKTRACE_ON_CRASH :: 10;
    EMIT_DEBUG_INFO :: 11;
    BITCODE_OPTIMIZATION :: 12;
    MACHINE_CODE_OPTIMIZATION :: 13;
}

Workspace_List :: enum u16 {
    IMPORT_PATH :: 0;
    ADDITIONAL_LINKER_ARGUMENT :: 1;
}


#scope_module
require :: (condition: bool, format: string, args: ..Any, loc := #caller_location) {
    if condition return;

    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);
    success := Basic.print_to_builder(*builder, format, ..args);
    message := Basic.builder_to_string(*builder);

    if !success message = format;

    __jaic_report(message, loc.fully_pathed_filename, loc.line_number, loc.character_number, 0);
}

#scope_file
Basic :: #import "Basic";
