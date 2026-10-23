//! The public #compiler contract, independent of the selected standard library's bodies.
mod abi;
mod binding;
mod convert;
mod messages;
mod operations;
mod options;
mod records;

pub(crate) use abi::{Binding, Field, Kind, Layout, Schema};
pub(crate) use binding::bind;
pub(crate) use operations::{State, call};

use crate::macros::raw_enum;

raw_enum! {
pub enum Operation by name {
    CreateWorkspace = "compiler_create_workspace",
    DestroyWorkspace = "compiler_destroy_workspace",
    CurrentWorkspace = "get_current_workspace",
    Name = "get_name",
    GetOptions = "get_build_options",
    SetOptions = "set_build_options",
    SetOptionsDuringCompile = "set_build_options_dc",
    AddFile = "add_build_file",
    AddString = "add_build_string_scoped_by_code" | "add_build_string",
    AddStringToModule = "add_build_string_scoped_by_message",
    Report = "compiler_report",
    Version = "compiler_get_version_info",
    CommandLine = "get_toplevel_command_line",
    BasePath = "compiler_get_base_path",
    AddLibraryDirectory = "compiler_add_library_search_directory",
    SetTypeInfoFlags = "compiler_set_type_info_flags",
    StructLocation = "compiler_get_struct_location",
    RemapImport = "remap_import",
    ProvideImport = "provide_import",
    GetNodes = "compiler_get_nodes",
    GetCode = "compiler_get_code",
    RootType = "get_root_type",
    GetType = "get_type",
    TypeTable = "get_type_table",
    MakeLive = "compiler_make_procedure_live",
    ModifyProcedure = "compiler_modify_procedure",
    BeginIntercept = "compiler_begin_intercept",
    EndIntercept = "compiler_end_intercept",
    WaitForMessage = "compiler_wait_for_message",
    SetWorkspaceStatus = "compiler_set_workspace_status",
    CustomLinkComplete = "compiler_custom_link_command_is_complete",
    AddDataSegment = "add_data_segment",
    AddGlobalData = "add_global_data",
    RuntimeInfo = "get_runtime_info",
    MemoryBreakpoint = "compiler_set_memory_breakpoint",
    DeveloperDebug = "developer_debug",
}
}

impl Operation {
    /// Whether the operation changes something a compile-time run can observe, so the run
    /// cannot be repeated (`Interp::effects`). Waiting for a message counts only when it steps
    /// a workspace (`messages::call`).
    fn is_effect(self) -> bool {
        !matches!(
            self,
            Self::CurrentWorkspace
                | Self::Name
                | Self::GetType
                | Self::GetNodes
                | Self::GetCode
                | Self::RootType
                | Self::TypeTable
                | Self::RuntimeInfo
                | Self::Version
                | Self::BasePath
                | Self::CommandLine
                | Self::StructLocation
                | Self::WaitForMessage
                | Self::MemoryBreakpoint
                | Self::DeveloperDebug
        )
    }

    fn records(self) -> bool {
        matches!(
            self,
            Self::GetNodes
                | Self::GetCode
                | Self::RootType
                | Self::MakeLive
                | Self::ModifyProcedure
                | Self::WaitForMessage
                | Self::ProvideImport
                | Self::AddStringToModule
        )
    }
}
