//! Private inventory of remaining source contracts, not intrinsic admission.
//!
//! Registration requires selected-module origin, exact canonical source schema,
//! and a real provider for the capability below. An enum entry alone never
//! grants a body, compiler effect, callback, or host-data capability.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PendingCompilerApi {
    ModifyProcedure,
    MakeProcedureLive,
    CustomLinkCommandComplete,
    StructLocation,
    UnresolvedIdentifierErrors,
    UntypedDeclarationErrors,
    MemoryBreakpoint,
    LibrarySearchDirectory,
    GetNodes,
    GetCode,
    RootType,
    SetTypeInfoFlags,
    GetType,
    AddStringByMessage,
    RemapImport,
    ProvideImport,
    SetOptionsDuringCompile,
    AddGlobalData,
    AddDataSegment,
    DeveloperDebug,
    CommandLine,
    BasePath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompilerCapability {
    CheckedProcedureMutation,
    ProcedureLiveness,
    ActualLinkCompletion,
    OwnedTypeLocation,
    UnresolvedSourceDiagnostics,
    MemoryDebugger,
    NativeSearchPaths,
    ImmutableSourceNodes,
    SourceCodeConstruction,
    CheckedCodeRootType,
    AtomicRecordReflectionMutation,
    OwnedRuntimeTypeIdentity,
    MessageScopeIdentity,
    ImportResolution,
    DuringCompileSettings,
    NativeDataSegments,
    DeveloperDebugger,
    InvocationArguments,
    CompilerBasePath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SourceNominal {
    ProcedureBody,
    ProcedureHeader,
    StructTypeInfo,
    SourceLocation,
    CodeNode,
    RootTypeStatus,
    TypeInfoFlags,
    TypeInfo,
    Message,
    FailedImportMessage,
    ProvidedImportType,
    OptionsDuringCompile,
    DataSegmentIndex,
    DataSegment,
    DataSegmentCharacteristics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SourceShape {
    Workspace,
    String,
    VoidPointer,
    Signed32,
    Bool,
    Type,
    Code,
    Named(SourceNominal),
    Pointer(SourceNominal),
    ByteSlice,
    StringSlice,
    NodePointerSlice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SourceDefault {
    None,
    CurrentWorkspace,
    NullCode,
    NullPointer,
    EnumBits(u128),
    Alignment16,
    CallerLocation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourceArgument {
    pub shape: SourceShape,
    pub default: SourceDefault,
    pub variadic: bool,
}
macro_rules! required {
    ($shape:expr) => {
        SourceArgument {
            shape: $shape,
            default: SourceDefault::None,
            variadic: false,
        }
    };
}
macro_rules! defaulted {
    ($shape:expr, $default:expr) => {
        SourceArgument {
            shape: $shape,
            default: $default,
            variadic: false,
        }
    };
}
pub(super) struct PendingSourceContract {
    pub arguments: &'static [SourceArgument],
    pub results: &'static [SourceShape],
    pub capability: CompilerCapability,
}

/// Dispatch preserves missing authority without a successful placeholder value.
/// The source caller supplies its actual use span when rendering the diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UnavailableCompilerCapability {
    pub api: PendingCompilerApi,
    pub required: CompilerCapability,
}

impl PendingCompilerApi {
    pub(super) fn unavailable(self) -> UnavailableCompilerCapability {
        UnavailableCompilerCapability {
            api: self,
            required: self.contract().capability,
        }
    }
    /// Parse once at the explicit #compiler source boundary. Ordinary procedure
    /// spellings do not reach this function and unknown tags remain unknown.
    pub(super) fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "compiler_modify_procedure" => Self::ModifyProcedure,
            "compiler_make_procedure_live" => Self::MakeProcedureLive,
            "compiler_custom_link_command_is_complete" => Self::CustomLinkCommandComplete,
            "compiler_get_struct_location" => Self::StructLocation,
            "compiler_report_errors_for_unresolved_identifiers" => Self::UnresolvedIdentifierErrors,
            "compiler_report_errors_for_untyped_declarations_with_these_notes" => {
                Self::UntypedDeclarationErrors
            }
            "compiler_set_memory_breakpoint" => Self::MemoryBreakpoint,
            "compiler_add_library_search_directory" => Self::LibrarySearchDirectory,
            "compiler_get_nodes" => Self::GetNodes,
            "compiler_get_code" => Self::GetCode,
            "get_root_type" => Self::RootType,
            "compiler_set_type_info_flags" => Self::SetTypeInfoFlags,
            "get_type" => Self::GetType,
            "add_build_string_scoped_by_message" => Self::AddStringByMessage,
            "remap_import" => Self::RemapImport,
            "provide_import" => Self::ProvideImport,
            "set_build_options_dc" => Self::SetOptionsDuringCompile,
            "add_global_data" => Self::AddGlobalData,
            "add_data_segment" => Self::AddDataSegment,
            "developer_debug" => Self::DeveloperDebug,
            "get_toplevel_command_line" => Self::CommandLine,
            "compiler_get_base_path" => Self::BasePath,
            _ => return None,
        })
    }

    /// Compile-only Code slots stay in this source contract. They must not be
    /// inserted into a canonical runtime ProcedureType or replaced by pointers.
    pub(super) fn contract(self) -> PendingSourceContract {
        use CompilerCapability as C;
        use SourceDefault as D;
        use SourceNominal as N;
        use SourceShape as S;
        let (arguments, results, capability): (
            &'static [SourceArgument],
            &'static [SourceShape],
            C,
        ) = match self {
            Self::ModifyProcedure => (
                &[
                    required!(S::Workspace),
                    required!(S::Pointer(N::ProcedureBody)),
                ],
                &[],
                C::CheckedProcedureMutation,
            ),
            Self::MakeProcedureLive => (
                &[
                    required!(S::Workspace),
                    required!(S::Pointer(N::ProcedureHeader)),
                ],
                &[],
                C::ProcedureLiveness,
            ),
            Self::CustomLinkCommandComplete => {
                (&[required!(S::Workspace)], &[], C::ActualLinkCompletion)
            }
            Self::StructLocation => (
                &[
                    required!(S::Workspace),
                    required!(S::Pointer(N::StructTypeInfo)),
                ],
                &[S::Named(N::SourceLocation)],
                C::OwnedTypeLocation,
            ),
            Self::UnresolvedIdentifierErrors => (
                &[
                    required!(S::String),
                    defaulted!(S::Workspace, D::CurrentWorkspace),
                ],
                &[],
                C::UnresolvedSourceDiagnostics,
            ),
            Self::UntypedDeclarationErrors => (
                &[
                    required!(S::Workspace),
                    SourceArgument {
                        shape: S::String,
                        default: D::None,
                        variadic: true,
                    },
                ],
                &[],
                C::UnresolvedSourceDiagnostics,
            ),
            Self::MemoryBreakpoint => (&[required!(S::VoidPointer)], &[], C::MemoryDebugger),
            Self::LibrarySearchDirectory => (&[required!(S::String)], &[], C::NativeSearchPaths),
            Self::GetNodes => (
                &[required!(S::Code)],
                &[S::Pointer(N::CodeNode), S::NodePointerSlice],
                C::ImmutableSourceNodes,
            ),
            Self::GetCode => (
                &[
                    required!(S::Pointer(N::CodeNode)),
                    defaulted!(S::Code, D::NullCode),
                ],
                &[S::Code],
                C::SourceCodeConstruction,
            ),
            Self::RootType => (
                &[required!(S::Code)],
                &[S::Named(N::RootTypeStatus), S::Type],
                C::CheckedCodeRootType,
            ),
            Self::SetTypeInfoFlags => (
                &[required!(S::Type), required!(S::Named(N::TypeInfoFlags))],
                &[],
                C::AtomicRecordReflectionMutation,
            ),
            Self::GetType => (
                &[required!(S::Pointer(N::TypeInfo))],
                &[S::Type],
                C::OwnedRuntimeTypeIdentity,
            ),
            Self::AddStringByMessage => (
                &[
                    required!(S::String),
                    required!(S::Workspace),
                    required!(S::Pointer(N::Message)),
                    defaulted!(S::Named(N::SourceLocation), D::CallerLocation),
                ],
                &[],
                C::MessageScopeIdentity,
            ),
            Self::RemapImport => (
                &[
                    required!(S::Workspace),
                    required!(S::String),
                    required!(S::String),
                    required!(S::String),
                ],
                &[],
                C::ImportResolution,
            ),
            Self::ProvideImport => (
                &[
                    required!(S::Workspace),
                    required!(S::Pointer(N::FailedImportMessage)),
                    required!(S::Named(N::ProvidedImportType)),
                    required!(S::String),
                ],
                &[],
                C::ImportResolution,
            ),
            Self::SetOptionsDuringCompile => (
                &[
                    required!(S::Named(N::OptionsDuringCompile)),
                    defaulted!(S::Workspace, D::CurrentWorkspace),
                ],
                &[],
                C::DuringCompileSettings,
            ),
            Self::AddGlobalData => (
                &[
                    required!(S::ByteSlice),
                    required!(S::Named(N::DataSegmentIndex)),
                    defaulted!(S::Pointer(N::DataSegment), D::NullPointer),
                    defaulted!(S::Workspace, D::CurrentWorkspace),
                ],
                &[S::ByteSlice],
                C::NativeDataSegments,
            ),
            Self::AddDataSegment => (
                &[
                    required!(S::String),
                    defaulted!(S::Named(N::DataSegmentCharacteristics), D::EnumBits(3)),
                    defaulted!(S::Signed32, D::Alignment16),
                    defaulted!(S::Workspace, D::CurrentWorkspace),
                ],
                &[S::Pointer(N::DataSegment), S::Bool],
                C::NativeDataSegments,
            ),
            Self::DeveloperDebug => (&[required!(S::VoidPointer)], &[], C::DeveloperDebugger),
            Self::CommandLine => (&[], &[S::StringSlice], C::InvocationArguments),
            Self::BasePath => (&[], &[S::String], C::CompilerBasePath),
        };
        PendingSourceContract {
            arguments,
            results,
            capability,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiler_only_code_slots_and_results_remain_source_metadata() {
        let nodes = PendingCompilerApi::parse("compiler_get_nodes")
            .unwrap()
            .contract();
        assert_eq!(nodes.arguments[0].shape, SourceShape::Code);
        assert_eq!(
            nodes.results,
            [
                SourceShape::Pointer(SourceNominal::CodeNode),
                SourceShape::NodePointerSlice
            ]
        );
        let code = PendingCompilerApi::parse("compiler_get_code")
            .unwrap()
            .contract();
        assert_eq!(code.arguments[1].default, SourceDefault::NullCode);
        assert_eq!(code.results, [SourceShape::Code]);
    }
    #[test]
    fn scoped_string_tag_preserves_real_source_order_and_location_default() {
        let contract = PendingCompilerApi::parse("add_build_string_scoped_by_message")
            .unwrap()
            .contract();
        assert_eq!(contract.arguments.len(), 4);
        assert_eq!(contract.arguments[0].shape, SourceShape::String);
        assert_eq!(contract.arguments[1].shape, SourceShape::Workspace);
        assert_eq!(
            contract.arguments[2].shape,
            SourceShape::Pointer(SourceNominal::Message)
        );
        assert_eq!(contract.arguments[3].default, SourceDefault::CallerLocation);
        assert_eq!(
            contract.capability,
            CompilerCapability::MessageScopeIdentity
        );
    }
    #[test]
    fn unknown_tags_and_plain_function_variants_do_not_gain_capabilities() {
        assert_eq!(PendingCompilerApi::parse("compiler_invented_effect"), None);
        assert_eq!(PendingCompilerApi::parse("add_build_string"), None);
        let contract = PendingCompilerApi::parse("compiler_custom_link_command_is_complete")
            .unwrap()
            .contract();
        assert_eq!(
            contract.capability,
            CompilerCapability::ActualLinkCompletion
        );
        assert!(contract.results.is_empty());
        assert_eq!(
            PendingCompilerApi::CustomLinkCommandComplete.unavailable(),
            UnavailableCompilerCapability {
                api: PendingCompilerApi::CustomLinkCommandComplete,
                required: CompilerCapability::ActualLinkCompletion,
            }
        );
    }
}
