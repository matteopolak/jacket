//! Typed keys and values for the compiler builtins that stdlib code calls with an enum.
//!
//! Each enum here mirrors one declared in `stdlib` (`Syntax_Key` in
//! `stdlib/internal/program-print/attributes.jai`, the `Workspace_*` option enums and the
//! `Build_Options` enums in `stdlib/internal/compiler-api` and `stdlib/Compiler`). The builtin
//! receives the enum's integer value, so a misspelt key is a type error in Jai and an
//! out-of-range value is rejected by [`from_raw`](SyntaxKey::from_raw), never matched by text.
//! `tests/typed_builtin_keys.rs` fails when a declaration here and its Jai twin drift apart.

macro_rules! raw_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$vmeta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $name {
            $($(#[$vmeta])* $variant = $value),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// The member whose Jai integer value is `raw`.
            pub fn from_raw(raw: u64) -> Option<Self> {
                Self::ALL.iter().copied().find(|member| *member as u64 == raw)
            }

            /// Member names and values, as the drift test compares them with the Jai side.
            #[doc(hidden)]
            pub fn declared() -> Vec<(&'static str, i64)> {
                vec![$((stringify!($variant), $value)),+]
            }
        }
    };
}

raw_enum! {
    /// A `Build_Options.Output_Type`.
    pub enum OutputType {
        NoOutput = 0,
        Executable = 1,
        DynamicLibrary = 2,
        StaticLibrary = 3,
        ObjectFile = 4,
    }
}

raw_enum! {
    /// The `Operating_System_Tag` members jaic can build for.
    pub enum OsTag {
        Windows = 2,
        Linux = 3,
        MacOS = 6,
        Wasm = 11,
    }
}

raw_enum! {
    /// The `CPU_Tag` members jaic can build for (`.CUSTOM` is a wasm target's spelling).
    pub enum CpuTag {
        Custom = 2,
        X64 = 3,
        Arm64 = 4,
    }
}

raw_enum! {
    /// `Build_Options.array_bounds_check`.
    pub enum ArrayBoundsCheck {
        Off = 0,
        On = 1,
        Always = 2,
    }
}

raw_enum! {
    /// `Build_Options.cast_bounds_check` and `arithmetic_overflow_check`.
    pub enum CheckLevel {
        Off = 0,
        NonFatal = 1,
        Fatal = 2,
    }
}

raw_enum! {
    /// `Build_Options.null_pointer_check` and `backtrace_on_crash`.
    pub enum Toggle {
        Off = 0,
        On = 1,
    }
}

raw_enum! {
    /// `Build_Options.runtime_support_definitions`.
    pub enum RuntimeSupport {
        Auto = 0,
        EntryPointAndInit = 1,
        OnlyInit = 2,
        Omit = 3,
    }
}

raw_enum! {
    /// `Build_Options.dead_code_elimination`.
    pub enum DeadCodeSetting {
        None = 0,
        All = 1,
        ModulesOnly = 2,
    }
}

raw_enum! {
    /// `Build_Options.emit_debug_info`.
    pub enum DebugInfoSetting {
        None = 0,
        Dwarf = 1,
        Codeview = 2,
        Default = 3,
    }
}

raw_enum! {
    /// `Llvm_Bitcode_Optimization_Setting`.
    pub enum BitcodeOptimization {
        Unset = 0,
        O0 = 1,
        O1 = 2,
        O2 = 3,
        O3 = 4,
        Os = 5,
        Oz = 6,
    }
}

raw_enum! {
    /// `Llvm_Machine_Code_Optimization_Setting`.
    pub enum MachineCodeOptimization {
        Unset = 0,
        None = 1,
        Less = 2,
        Default = 3,
        Aggressive = 4,
    }
}

raw_enum! {
    /// `Workspace_Text`: options that hold text.
    pub enum WorkspaceText {
        OutputExecutableName = 0,
        OutputPath = 1,
        IntermediatePath = 2,
        EntryPointName = 3,
        LlvmTargetSystemTriple = 4,
        LlvmTargetSystemCpu = 5,
        LlvmTargetSystemFeatures = 6,
        MinimumOsVersion = 7,
    }
}

raw_enum! {
    /// `Workspace_Flag`: options that are on or off.
    pub enum WorkspaceFlag {
        DoOutput = 0,
        StackTrace = 1,
        DisableRedzone = 2,
        EnableFramePointers = 3,
        AppendExecutableFilenameExtension = 4,
        UseCustomLinkCommand = 5,
        OutputBitcode = 6,
        OutputLlvmIr = 7,
        OutputBitcodeBeforeOptimizations = 8,
        OutputLlvmIrBeforeOptimizations = 9,
        LlvmEnableTailCalls = 10,
        LlvmEnableLoopUnrolling = 11,
        LlvmEnableSlpVectorization = 12,
        LlvmEnableLoopVectorization = 13,
        LlvmPreserveDebugInfo = 14,
        LlvmMergeFunctions = 15,
        LlvmDisableInlining = 16,
        LlvmDisableMem2reg = 17,
        LlvmEnableSplitModules = 18,
        LlvmFunctionSections = 19,
    }
}

raw_enum! {
    /// `Workspace_Number`: options that hold an integer or the value of a `Build_Options` enum.
    pub enum WorkspaceNumber {
        OutputType = 0,
        OsTarget = 1,
        CpuTarget = 2,
        TemporaryStorageSize = 3,
        ArrayBoundsCheck = 4,
        CastBoundsCheck = 5,
        NullPointerCheck = 6,
        ArithmeticOverflowCheck = 7,
        DeadCodeElimination = 8,
        RuntimeSupportDefinitions = 9,
        BacktraceOnCrash = 10,
        EmitDebugInfo = 11,
        BitcodeOptimization = 12,
        MachineCodeOptimization = 13,
    }
}

raw_enum! {
    /// `Workspace_List`: options that hold a list of text.
    pub enum WorkspaceList {
        ImportPath = 0,
        AdditionalLinkerArgument = 1,
    }
}

raw_enum! {
    /// `Syntax_Key`: the private attributes a syntax record carries beyond the public `Code_*`
    /// layouts. Each is a record field named `__syntax_<member>` (see [`SyntaxKey::field`]).
    pub enum SyntaxKey {
        Alias = 0,
        AliasExpression = 1,
        Alignment = 2,
        AsmItems = 3,
        AsmKind = 4,
        AsmPresent = 5,
        AssertArgs = 6,
        AssertMessage = 7,
        BacktickLabel = 8,
        BindLabel = 9,
        Broadcast = 10,
        BuiltinName = 11,
        CallHint = 12,
        CallerCode = 13,
        Class = 14,
        Colon = 15,
        ContextBare = 16,
        DeprecationNamed = 17,
        DirectiveArgs = 18,
        DirectiveName = 19,
        ExportName = 20,
        ExportNamed = 21,
        Features = 22,
        Filter = 23,
        FilterArguments = 24,
        FilterExpression = 25,
        FilterKind = 26,
        Flags = 27,
        Foreign = 28,
        ForeignLibrary = 29,
        ForeignNamed = 30,
        ForeignSymbol = 31,
        ImportKind = 32,
        Incomplete = 33,
        InitialExpression = 34,
        Interface = 35,
        Iterator = 36,
        Label = 37,
        LambdaBody = 38,
        Mask = 39,
        Mnemonic = 40,
        Name = 41,
        Negate = 42,
        Operand = 43,
        Operands = 44,
        OriginalContext = 45,
        OriginalKind = 46,
        Pin = 47,
        PolyBaked = 48,
        ProgramExport = 49,
        Restriction = 50,
        ReturnMust = 51,
        Round = 52,
        RunBody = 53,
        Scale = 54,
        Size = 55,
        StatementNotes = 56,
        StaticFilter = 57,
        Suffix = 58,
        Terms = 59,
        UnionTag = 60,
        Using = 61,
        Value = 62,
        Zeroing = 63,
    }
}

impl SyntaxKey {
    /// The record field this key reads and writes.
    pub fn field(self) -> crate::records::Name {
        crate::records::Name::syntax(self)
    }
}
