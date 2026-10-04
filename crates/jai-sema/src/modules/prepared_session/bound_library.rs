//! Host proofs stay inseparable from the exact frozen library that owns their IDs.
use super::*;
use jai_vm::{ProcedureAvailability, ProcedureProvider, RuntimeInfoAvailability};

pub enum BoundLibraryReadiness {
    Complete(Box<BoundLibrary>),
    Pending(LibraryPending),
    Failed(LocatedDiagnostic),
}

pub(super) struct HostBindings {
    compiler: HashMap<ProcedureId, jai_vm::CompilerProcedure>,
    files: HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>,
    heap: HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>,
    processes: HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>,
    target: Option<jai_types::BuildTarget>,
    workspace: Option<jai_vm::WorkspaceId>,
    sources: HashMap<ProcedureId, OriginInput>,
}
struct OriginInput {
    identity: jai_ir::SourceProcedureIdentity,
    environment: Vec<u8>,
}
static NEXT_INVOCATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// A fresh trusted runtime invocation; numeric source values cannot reconstruct one.
pub struct RuntimeInvocation(u64);
impl RuntimeInvocation {
    pub fn allocate() -> Self {
        use std::sync::atomic::Ordering;
        Self(
            NEXT_INVOCATION
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("runtime invocation identities exhausted"),
        )
    }
}
impl HostBindings {
    pub(super) fn capture(
        phase: &PreparedPhase<'_>,
        worklist: &compile_time::Worklist<'_>,
    ) -> Result<Self, String> {
        let maps = worklist.host_maps();
        let mut sources = HashMap::new();
        for (declaration, signature) in &phase.declarations.signatures {
            let declaration = phase
                .graph
                .declaration(*declaration)
                .ok_or_else(|| "bound procedure has no actual source declaration".to_string())?;
            if !matches!(declaration.syntax().kind, FileDeclarationKind::Procedure(_)) {
                continue;
            }
            let location = declaration.location();
            let source = phase
                .graph
                .sources()
                .get(location.source)
                .ok_or_else(|| "bound procedure source is absent".to_string())?;
            let identity = jai_ir::SourceProcedureIdentity::new(source, location)
                .map_err(|error| error.to_string())?;
            let environment = phase
                .graph
                .module_environment_origin(declaration.file())
                .map_err(|error| error.to_string())?;
            sources.insert(
                signature.id,
                OriginInput {
                    identity,
                    environment,
                },
            );
        }
        Ok(Self {
            compiler: phase.compiler.clone(),
            files: maps.files,
            heap: maps.heap,
            processes: maps.processes,
            target: phase.options.target.clone(),
            workspace: phase
                .options
                .compiler
                .as_ref()
                .map(|context| context.current_workspace),
            sources,
        })
    }
}

pub struct BoundLibrary {
    library: Library,
    bindings: HostBindings,
}
impl BoundLibrary {
    // No public map-taking constructor: raw IDs from another arena cannot be rebound.
    pub(super) fn from_prepared(library: Library, bindings: HostBindings) -> Result<Self, String> {
        let mut granted = std::collections::HashSet::new();
        let mut check = |id: ProcedureId,
                         signature: TypeId,
                         expected_library: Option<jai_ir::ForeignLibraryId>|
         -> Result<(), String> {
            if !granted.insert(id) || library.checked_procedure(id).is_some() {
                return Err("host capability duplicates a frozen procedure identity".into());
            }
            let prototype = library
                .prototypes()
                .iter()
                .find(|prototype| prototype.id == id)
                .ok_or_else(|| "host capability has no actual frozen prototype".to_string())?;
            if prototype.signature != signature || library.signatures().get(&id) != Some(&signature)
            {
                return Err("host capability signature differs from its frozen prototype".into());
            }
            let origin_matches = match (expected_library, &prototype.origin) {
                (None, PrototypeOrigin::Compiler) => true,
                (
                    Some(expected),
                    PrototypeOrigin::Foreign {
                        library: Some(actual),
                        ..
                    },
                ) => expected == actual.id,
                _ => false,
            };
            if !origin_matches {
                return Err("host capability origin differs from its frozen prototype".into());
            }
            Ok(())
        };
        for (&id, binding) in &bindings.compiler {
            check(id, binding.signature, None)?;
        }
        for (&id, binding) in &bindings.files {
            check(id, binding.signature, Some(binding.library()))?;
            binding
                .validate(library.types())
                .map_err(|error| error.to_string())?;
        }
        for (&id, binding) in &bindings.heap {
            check(id, binding.signature, Some(binding.library()))?;
            binding
                .validate(library.types())
                .map_err(|error| error.to_string())?;
        }
        for (&id, binding) in &bindings.processes {
            check(id, binding.signature(), Some(binding.library()))?;
            let target = bindings
                .target
                .as_ref()
                .ok_or_else(|| "process capability has no retained source target".to_string())?;
            binding
                .validate(target, library.types())
                .map_err(|error| error.to_string())?;
        }
        Ok(Self { library, bindings })
    }
    pub fn library(&self) -> &Library {
        &self.library
    }
    pub fn target(&self) -> Option<&jai_types::BuildTarget> {
        self.bindings.target.as_ref()
    }
    pub fn workspace(&self) -> Option<jai_vm::WorkspaceId> {
        self.bindings.workspace
    }
    pub fn runtime_origin(
        &self,
        workspace: jai_vm::WorkspaceId,
        entry: ProcedureId,
        invocation: RuntimeInvocation,
    ) -> Result<jai_vm::SourceOrigin, String> {
        use std::hash::{Hash, Hasher};
        if self
            .bindings
            .workspace
            .is_some_and(|selected| selected != workspace)
        {
            return Err("runtime workspace differs from its prepared source authority".into());
        }
        if self.library.checked_procedure(entry).is_none() {
            return Err("runtime entry has no checked body in this bound library".into());
        }
        let input = self
            .bindings
            .sources
            .get(&entry)
            .ok_or_else(|| "runtime entry has no retained genuine source body".to_string())?;
        let span = input.identity.location().span;
        let body = input
            .identity
            .source_text()
            .as_bytes()
            .get(span.start..span.end)
            .ok_or_else(|| "runtime source range is invalid".to_string())?
            .to_vec();
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        body.hash(&mut hash);
        let mut specialization = input.environment.clone();
        specialization.extend_from_slice(b"\0runtime-invocation\0");
        specialization.extend_from_slice(&invocation.0.to_le_bytes());
        Ok(jai_vm::SourceOrigin {
            workspace,
            path: input.identity.path().to_owned(),
            start: span.start,
            end: span.end,
            body_hash: hash.finish(),
            body,
            specialization,
        })
    }
    /// Discard capabilities when returning the legacy immutable IR-only library.
    pub fn into_library(self) -> Library {
        self.library
    }
}
impl ProcedureProvider for BoundLibrary {
    fn build_target(&self) -> Option<&jai_types::BuildTarget> {
        self.target()
    }
    fn types(&self) -> &dyn jai_types::TypeView {
        self.library.types()
    }
    fn procedure_execution(&self, id: ProcedureId) -> jai_types::ProcedureExecution {
        if self.bindings.compiler.contains_key(&id) {
            jai_types::ProcedureExecution::CompileTimeOnly
        } else {
            self.library.procedure_phase(id)
        }
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.library.signatures()
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if let Some(binding) = self.bindings.compiler.get(&id) {
            ProcedureAvailability::Compiler(*binding)
        } else if let Some(binding) = self.bindings.files.get(&id) {
            ProcedureAvailability::FileAbi(*binding)
        } else if let Some(binding) = self.bindings.heap.get(&id) {
            ProcedureAvailability::HeapAbi(*binding)
        } else if let Some(binding) = self.bindings.processes.get(&id) {
            ProcedureAvailability::ProcessAbi(binding.clone())
        } else {
            ProcedureProvider::procedure(&self.library, id)
        }
    }
    fn globals(&self) -> &[Global] {
        self.library.globals()
    }
    fn places(&self) -> Option<&Places> {
        Some(self.library.places())
    }
    fn context(&self) -> Option<&ContextDefinition> {
        self.library.context()
    }
    fn storage_alignments(&self) -> Option<&StorageAlignments> {
        Some(self.library.storage_alignments())
    }
    fn runtime_info(&self) -> RuntimeInfoAvailability<'_> {
        ProcedureProvider::runtime_info(&self.library)
    }
}
