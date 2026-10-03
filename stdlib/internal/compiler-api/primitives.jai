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
__jaic_workspace_set_option :: (w: Workspace, key: string, value: string) #compiler;
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
