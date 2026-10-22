compiler_create_workspace :: (name := "") -> Workspace #compiler;

compiler_destroy_workspace :: (w: Workspace) #compiler;

compiler_set_memory_breakpoint :: (pointer: *void) #compiler;

developer_debug :: (x: *void) #compiler;

get_name :: (w: Workspace = -1) -> string #compiler;

get_build_options :: (w: Workspace = -1) -> Build_Options #compiler;

set_build_options :: (options: Build_Options, w: Workspace = -1, loc := #caller_location) #compiler;

set_build_options_dc :: (options: Build_Options_During_Compile, w: Workspace = -1) #compiler;

add_build_file :: (filename: string, w: Workspace, loc := #caller_location) #compiler;

add_build_string :: (data: string, w: Workspace, code := #code,null, loc := #caller_location) #compiler "add_build_string_scoped_by_code";

add_build_string :: (data: string, w: Workspace, message: *Message, loc := #caller_location) #compiler "add_build_string_scoped_by_message";

compiler_report :: (message: string, loc := #caller_location, mode := Report.ERROR) #compiler;

compiler_report :: (filename: string, line_number: s64, character_number: s64, error_text: string, mode := Report.ERROR) #compiler;

compiler_get_version_info :: (version_info_return: *Version_Info) -> string #compiler;

get_toplevel_command_line :: () -> [] string #compiler;

compiler_get_base_path :: () -> string #compiler;

compiler_add_library_search_directory :: (path: string) #compiler;

compiler_set_type_info_flags :: (type: Type, flags: Type_Info_Flags) #compiler;

compiler_get_struct_location :: (w: Workspace, info: *Type_Info_Struct) -> Source_Code_Location #compiler;

remap_import :: (w: Workspace, host_module_name: string, import_name: string, replacement_name: string) #compiler;

provide_import :: (w: Workspace, message: *Message_Failed_Import, type: Provided_Import_Type, value: string) #compiler;

compiler_get_nodes :: (code: Code) -> (root: *Code_Node, expressions: [] *Code_Node) #compiler;

compiler_get_code :: (node: *Code_Node, code_to_copy_scope_from: Code = #code,null) -> Code #compiler;

get_root_type :: (code: Code) -> (status: Get_Root_Type_Status, type: Type) #compiler;

get_type :: (ti: *Type_Info) -> Type #compiler {
    return cast(Type) ti;
}

get_type_table :: (w: Workspace = -1) -> [] *Type_Info #compiler {
    return get_runtime_info(w).type_table;
}

compiler_make_procedure_live :: (w: Workspace, header: *Code_Procedure_Header) #compiler;

compiler_modify_procedure :: (w: Workspace, body: *Code_Procedure_Body) #compiler;

compiler_begin_intercept :: (w: Workspace, flags: Intercept_Flags = 0) #compiler;

compiler_end_intercept :: (w: Workspace) #compiler;

compiler_set_workspace_status :: (status: Workspace_Status, w: Workspace = -1) #compiler;

compiler_wait_for_message :: () -> *Message #compiler;

compiler_custom_link_command_is_complete :: (w: Workspace) #compiler;

add_data_segment :: (section_name: string, characteristics := Data_Segment_Characteristics.READ | .WRITE, alignment: s32 = 16, w: Workspace = -1) -> (segment: *Data_Segment, actual_segment_will_be_created: bool) #compiler;

add_global_data :: (data: [] u8, segment: Data_Segment_Index, user_segment: *Data_Segment = null, w: Workspace = -1) -> [] u8 #compiler;

get_runtime_info :: (w: Workspace = -1) -> Runtime_Info #compiler {
    return __runtime_info;
}

#scope_file
__runtime_info: Runtime_Info #elsewhere;
