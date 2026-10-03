//! Shared compiler configuration; independent of VM and LLVM implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BitcodeOptimization {
    #[default]
    Unset,
    O0,
    O1,
    O2,
    O3,
    Os,
    Oz,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MachineOptimization {
    #[default]
    Unset,
    None,
    Less,
    Default,
    Aggressive,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildOutputKind {
    None,
    #[default]
    Executable,
    DynamicLibrary,
    StaticLibrary,
    Object,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSupportMode {
    #[default]
    Auto,
    EntryPointAndInitialization,
    InitializationOnly,
    Omit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BacktraceOnCrash {
    Off,
    #[default]
    On,
}

/// Target facts used by source selection and compile-time memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildTarget {
    pub operating_system: OperatingSystem,
    pub architecture: Architecture,
    pub layout: crate::LayoutPolicy,
    pub byte_order: ByteOrder,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatingSystem {
    MacOS,
    Linux,
    Windows,
    Android,
    IOS,
    WebAssembly,
    Other(String),
}
impl OperatingSystem {
    /// Source-visible names from the supplied Preload enum, never host guesses.
    pub fn source_tag(&self) -> Option<&'static str> {
        Some(match self {
            Self::MacOS => "MACOS",
            Self::Linux => "LINUX",
            Self::Windows => "WINDOWS",
            Self::Android => "ANDROID",
            Self::IOS => "IOS",
            Self::WebAssembly => "WASM",
            Self::Other(_) => return None,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Arm64,
    X86,
    Arm,
    WebAssembly32,
    WebAssembly64,
    Other(String),
}
impl Architecture {
    pub fn source_tag(&self) -> Option<&'static str> {
        match self {
            Self::X86_64 => Some("X64"),
            Self::Arm64 => Some("ARM64"),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    Little,
    Big,
}

/// Debug emission is explicit; line tables do not promise variable inspection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebugInformation {
    #[default]
    Off,
    LineTables,
    /// Source locations and the supported runtime variable types.
    Variables,
}
