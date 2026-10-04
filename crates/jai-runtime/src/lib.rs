//! LLVM-independent Jai scripting using the compiler's checked IR interpreter.
mod entry;
mod sources;
pub use jai_modules::{GraphOptions, SourceProvider};
use jai_types::{
    Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout,
};
pub use jai_vm::{Limits, Statistics};
pub use sources::{DEFAULT_SOURCE_LIMIT, SourceBundle};
use std::{fmt, path::Path};

#[derive(Clone, Debug)]
pub struct Options {
    pub target: BuildTarget,
    pub graph: GraphOptions,
    pub limits: Limits,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            target: host_target(),
            graph: GraphOptions::default(),
            limits: Limits::default(),
        }
    }
}
/// Selected source/virtual-memory target; this never invokes LLVM target detection.
pub fn host_target() -> BuildTarget {
    if cfg!(target_arch = "wasm32") {
        return browser_target();
    }
    let operating_system = match std::env::consts::OS {
        "macos" => OperatingSystem::MacOS,
        "linux" => OperatingSystem::Linux,
        "windows" => OperatingSystem::Windows,
        "android" => OperatingSystem::Android,
        "ios" => OperatingSystem::IOS,
        other => OperatingSystem::Other(other.into()),
    };
    let architecture = match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Arm64,
        "x86" => Architecture::X86,
        "arm" => Architecture::Arm,
        other => Architecture::Other(other.into()),
    };
    BuildTarget {
        operating_system,
        architecture,
        layout: if cfg!(target_pointer_width = "32") {
            pointer32()
        } else {
            LayoutPolicy::lp64()
        },
        byte_order: if cfg!(target_endian = "big") {
            ByteOrder::Big
        } else {
            ByteOrder::Little
        },
    }
}
fn pointer32() -> LayoutPolicy {
    LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 8),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
        ScalarLayout::new(1, 1),
    )
    .expect("fixed wasm32 layout is valid")
}
pub fn browser_target() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::WebAssembly,
        architecture: Architecture::WebAssembly32,
        layout: pointer32(),
        byte_order: ByteOrder::Little,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptParameters {
    None,
    Arguments(jai_types::TypeId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptResult {
    Void,
    Int,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptEntry {
    pub procedure: jai_sema::ProcedureId,
    pub parameters: ScriptParameters,
    pub result: ScriptResult,
}
/// Host services require actual authenticated adapters; the portable profile has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCapability {
    Files,
    Processes,
    Graphics,
    ForeignFunctions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingHostBinding {
    ForeignProcedure(jai_sema::ProcedureId),
    ExternalGlobal(jai_sema::GlobalId),
}
#[derive(Debug)]
pub enum Error {
    Source(jai_modules::GraphError),
    Diagnostic(String),
    Entry(&'static str),
    Types(String),
    Arguments(&'static str),
    Execution(jai_vm::Error),
    Pending(Vec<jai_vm::Dependency>),
    HostBindingRequired(MissingHostBinding),
    InvalidResult,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => error.fmt(f),
            Self::Diagnostic(text) | Self::Types(text) => f.write_str(text),
            Self::Entry(text) | Self::Arguments(text) => f.write_str(text),
            Self::Execution(jai_vm::Error::Limit(kind)) => {
                write!(f, "script execution exceeded {kind:?} limit")
            }
            Self::Execution(error) => write!(f, "script execution failed: {error}"),
            Self::Pending(dependencies) => write!(
                f,
                "script requires unresolved dependencies: {dependencies:?}"
            ),
            Self::HostBindingRequired(MissingHostBinding::ForeignProcedure(id)) => write!(
                f,
                "script foreign procedure {} requires an explicit host binding",
                id.index()
            ),
            Self::HostBindingRequired(MissingHostBinding::ExternalGlobal(id)) => write!(
                f,
                "script external global {} requires an explicit host binding",
                id.index()
            ),
            Self::InvalidResult => f.write_str("script main returned an invalid result"),
        }
    }
}
impl std::error::Error for Error {
}
impl From<jai_vm::Error> for Error {
    fn from(error: jai_vm::Error) -> Self {
        match error {
            jai_vm::Error::UnsupportedForeignProcedure(id) => {
                Self::HostBindingRequired(MissingHostBinding::ForeignProcedure(id))
            }
            jai_vm::Error::UnsupportedExternalGlobal(id) => {
                Self::HostBindingRequired(MissingHostBinding::ExternalGlobal(id))
            }
            error => Self::Execution(error),
        }
    }
}

pub struct Script {
    library: jai_sema::Library,
    entry: ScriptEntry,
    target: BuildTarget,
    limits: Limits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub exit_code: i64,
    pub statistics: Statistics,
}
impl RunResult {
    /// Ordinary OS process status exposes the least significant eight bits.
    pub fn process_status(self) -> u8 {
        self.exit_code as u8
    }
}
impl Script {
    /// Compile a complete source graph using an explicit source provider.
    /// Source loading is distinct from runtime filesystem capability.
    pub fn prepare(
        path: &Path,
        provider: &dyn SourceProvider,
        options: Options,
    ) -> Result<Self, Error> {
        let graph = jai_modules::ModuleGraph::load_with_target(
            path,
            options.graph,
            provider,
            options.target.clone(),
        )
        .map_err(Error::Source)?;
        let resolve = jai_sema::ResolveOptions {
            target: Some(options.target.clone()),
            compile_time_limits: options.limits,
            ..Default::default()
        };
        let library =
            jai_sema::resolve_library_with_options(&graph, &resolve, &mut jai_vm::NoEffects)
                .map_err(|error| Error::Diagnostic(error.render(graph.sources())))?;
        let entry = entry::select(&graph, &library)?;
        Ok(Self {
            library,
            entry,
            target: options.target,
            limits: options.limits,
        })
    }
    pub fn from_source(source: &str, options: Options) -> Result<Self, Error> {
        let mut bundle = SourceBundle::default();
        let path = bundle
            .insert("main.jai", source.as_bytes().to_vec())
            .map_err(|error| Error::Diagnostic(error.to_string()))?;
        Self::prepare(&path, &bundle, options)
    }
    pub fn entry(&self) -> ScriptEntry {
        self.entry
    }
    pub fn library(&self) -> &jai_sema::Library {
        &self.library
    }
    pub fn host_capabilities(&self) -> &'static [HostCapability] {
        &[]
    }
    /// Each run receives isolated virtual globals, context and argument backing.
    /// Argument boundaries are preserved and no implicit argv[0] is inserted.
    pub fn run(&self, arguments: &[String]) -> Result<RunResult, Error> {
        let mut vm = jai_vm::Vm::new_with_execution_phase(
            &self.library,
            jai_vm::NoEffects,
            self.limits,
            jai_vm::ByteTarget::from(&self.target),
            jai_vm::ExecutionPhase::Runtime,
        )?;
        let values = match self.entry.parameters {
            ScriptParameters::None if !arguments.is_empty() => {
                return Err(Error::Arguments("this script main takes no arguments"));
            }
            ScriptParameters::None => vec![],
            ScriptParameters::Arguments(ty) => vec![vm.allocate_string_slice(ty, arguments)?],
        };
        let execution = vm.execute(self.entry.procedure, values);
        let exit_code = match execution.outcome {
            jai_vm::Outcome::Complete(values) => match (self.entry.result, values.as_slice()) {
                (ScriptResult::Void, []) => 0,
                (ScriptResult::Int, [jai_vm::Value::Int(value)]) => {
                    i64::try_from(value.value()).map_err(|_| Error::InvalidResult)?
                }
                _ => return Err(Error::InvalidResult),
            },
            jai_vm::Outcome::Failed(error) => return Err(error.into()),
            jai_vm::Outcome::Pending(dependencies) => return Err(Error::Pending(dependencies)),
        };
        Ok(RunResult {
            exit_code,
            statistics: execution.statistics,
        })
    }
}
