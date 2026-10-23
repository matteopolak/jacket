//! The operations a metaprogram can call on the compiler.

use crate::macros::raw_enum;

raw_enum! {
/// The `#compiler` primitives, bound by name in `stdlib/Compiler`.
pub enum MetaOp by name {
    WorkspaceCreate = "__jaic_workspace_create",
    CurrentWorkspace = "__jaic_current_workspace" | "get_current_workspace",
    ValidateWorkspace = "__jaic_workspace_validate",
    DataSegment = "__jaic_data_segment",
    GlobalData = "__jaic_global_data",
    RuntimeData = "__jaic_runtime_data",
    AddFile = "__jaic_workspace_add_file",
    AddString = "__jaic_workspace_add_string",
    DestroyWorkspace = "__jaic_workspace_destroy",
    SetText = "__jaic_workspace_set_text",
    SetFlag = "__jaic_workspace_set_flag",
    SetNumber = "__jaic_workspace_set_number",
    ClearList = "__jaic_workspace_clear_list",
    AppendList = "__jaic_workspace_append_list",
    OptionsSnapshot = "__jaic_workspace_options_snapshot",
    BeginIntercept = "__jaic_workspace_begin_intercept",
    EndIntercept = "__jaic_workspace_end_intercept",
    WorkspaceStatus = "__jaic_workspace_status",
    NextEvent = "__jaic_workspace_next_event",
    EventInt = "__jaic_event_int",
    EventString = "__jaic_event_string",
    CommandLineCount = "__jaic_command_line_count",
    CommandLineArg = "__jaic_command_line_arg",
    Report = "__jaic_report",
    CompilerVersion = "__jaic_compiler_version",
    CustomLinkComplete = "__jaic_custom_link_complete",
    AddStringToModule = "__jaic_workspace_add_string_to_module",
    AddStringToCode = "__jaic_workspace_add_string_to_code",
    CodeNodes = "__jaic_code_nodes",
    CodeIsNull = "__jaic_code_is_null",
    SetTypeInfoFlags = "__jaic_set_type_info_flags",
    StructLocation = "__jaic_struct_location",
    BasePath = "__jaic_base_path",
    TypeTable = "__jaic_type_table",
    AddLibraryDir = "__jaic_add_library_dir",
    RemapImport = "__jaic_workspace_remap_import",
    ProvideImport = "__jaic_workspace_provide_import",
    MakeLive = "__jaic_make_procedure_live",
    RecInt = "__jaic_rec_int",
    RecRef = "__jaic_rec_ref",
    RecCount = "__jaic_rec_count",
    RecItemRef = "__jaic_rec_item_ref",
    RecBuilder = "__jaic_rec_builder",
    RecType = "__jaic_rec_type",
    RecBuiltin = "__jaic_rec_builtin",
    RecPlan = "__jaic_rec_plan",
    RecBuild = "__jaic_rec_build",
    RecOf = "__jaic_rec_of",
    SyntaxInt = "__jaic_syntax_int",
    SyntaxString = "__jaic_syntax_string",
    SyntaxChild = "__jaic_syntax_child",
}
}

/// Id of the program being compiled by the command line (workspace 1 is reserved, as in `jai`).
pub const TOP_LEVEL_WORKSPACE: i64 = 2;

/// Jai language version reported to programs, as major / minor / micro. Build scripts check it
/// against a supported range, so it stays in the 0.2 line and comes first in the version text.
pub const LANGUAGE_VERSION: (u32, u32, u32) = (0, 2, 99);

/// Version text returned by `compiler_get_version_info`: the language version, whose three
/// integers fill `Version_Info`, then the Jacket product version.
pub fn compiler_version() -> String {
    let (major, minor, micro) = LANGUAGE_VERSION;
    format!(
        "Jai {major}.{minor}.{micro} (Jacket {})",
        env!("CARGO_PKG_VERSION")
    )
}
