//! Dispatching compiler calls from interpreted metaprograms to the registry.
use super::meta_op::MetaOp;
use super::state::SharedWorkspaces;
use super::{host, messages, program_data, records, sources, workspace};
use crate::interp::{Interp, Rets, Trap};
use crate::ir;

/// The primitives are grouped by what they act on; each family has its own module.
enum Family {
    Workspace,
    Messages,
    Sources,
    Host,
    ProgramData,
    Records,
}

impl Family {
    /// Exhaustive, so a new [`MetaOp`] must be given a family.
    fn of(op: MetaOp) -> Family {
        match op {
            MetaOp::WorkspaceCreate
            | MetaOp::CurrentWorkspace
            | MetaOp::ValidateWorkspace
            | MetaOp::DestroyWorkspace
            | MetaOp::WorkspaceStatus
            | MetaOp::MakeLive
            | MetaOp::AddLibraryDir
            | MetaOp::RemapImport
            | MetaOp::ProvideImport
            | MetaOp::SetText
            | MetaOp::SetFlag
            | MetaOp::SetNumber
            | MetaOp::ClearList
            | MetaOp::AppendList
            | MetaOp::OptionsSnapshot
            | MetaOp::CustomLinkComplete => Family::Workspace,
            MetaOp::BeginIntercept
            | MetaOp::EndIntercept
            | MetaOp::NextEvent
            | MetaOp::EventInt
            | MetaOp::EventString
            | MetaOp::Report => Family::Messages,
            MetaOp::AddFile
            | MetaOp::AddString
            | MetaOp::AddStringToModule
            | MetaOp::AddStringToCode
            | MetaOp::CodeIsNull
            | MetaOp::CodeNodes => Family::Sources,
            MetaOp::CompilerVersion
            | MetaOp::BasePath
            | MetaOp::CommandLineCount
            | MetaOp::CommandLineArg => Family::Host,
            MetaOp::SetTypeInfoFlags
            | MetaOp::TypeTable
            | MetaOp::StructLocation
            | MetaOp::RuntimeData
            | MetaOp::DataSegment
            | MetaOp::GlobalData => Family::ProgramData,
            MetaOp::SyntaxInt
            | MetaOp::SyntaxString
            | MetaOp::SyntaxChild
            | MetaOp::RecBuilder
            | MetaOp::RecType
            | MetaOp::RecPlan
            | MetaOp::RecBuild
            | MetaOp::RecOf
            | MetaOp::RecCount
            | MetaOp::RecInt
            | MetaOp::RecBuiltin
            | MetaOp::RecRef
            | MetaOp::RecItemRef => Family::Records,
        }
    }
}

/// What one primitive call sees besides the interpreter: the registry, the program, and the
/// arguments (without the context pointer).
pub(super) struct Call<'a> {
    pub(super) shared: &'a SharedWorkspaces,
    pub(super) program: &'a ir::Program,
    args: &'a [u64],
}

impl Call<'_> {
    /// Argument `i`. The Jai declaration of a primitive fixes how many it passes, so asking for
    /// one past them is a mistake in the Rust side of that primitive.
    pub(super) fn arg(&self, i: usize) -> u64 {
        *self
            .args
            .get(i)
            .expect("invariant: a primitive reads only the arguments its declaration passes")
    }

    pub(super) fn string(&self, interp: &Interp, i: usize) -> Result<Vec<u8>, Trap> {
        let p = self.arg(i);
        if p == 0 {
            return Ok(Vec::new());
        }
        let (count, data) = interp.read_pair(p).ok_or_else(|| Trap::unreadable(p, 16))?;
        let count = count as usize;
        if count == 0 || data == 0 {
            Ok(Vec::new())
        } else {
            interp
                .read(data, count)
                .ok_or_else(|| Trap::unreadable(data, count))
        }
    }

    pub(super) fn text(&self, interp: &Interp, i: usize) -> Result<String, Trap> {
        Ok(String::from_utf8_lossy(&self.string(interp, i)?).into_owned())
    }

    pub(super) fn return_string(
        &self,
        interp: &mut Interp,
        bytes: &[u8],
        out_index: usize,
    ) -> Result<(), Trap> {
        let (count, data) = self.shared.borrow_mut().keep_string(bytes);
        store_pair(interp, self.arg(out_index), count, data)
    }
}

/// Primitives that only read the registry (or make records nobody else sees yet) leave a
/// compile-time run repeatable; the rest are effects.
fn is_effect(op: MetaOp) -> bool {
    !matches!(
        op,
        MetaOp::CurrentWorkspace
            | MetaOp::ValidateWorkspace
            | MetaOp::RuntimeData
            | MetaOp::EventInt
            | MetaOp::EventString
            | MetaOp::CommandLineCount
            | MetaOp::CommandLineArg
            | MetaOp::CompilerVersion
            | MetaOp::StructLocation
            | MetaOp::BasePath
            | MetaOp::TypeTable
            | MetaOp::CodeIsNull
            | MetaOp::CodeNodes
            | MetaOp::RecInt
            | MetaOp::RecRef
            | MetaOp::RecCount
            | MetaOp::RecItemRef
            | MetaOp::RecBuilder
            | MetaOp::RecType
            | MetaOp::RecBuiltin
            | MetaOp::RecPlan
            | MetaOp::RecBuild
            | MetaOp::RecOf
            | MetaOp::SyntaxInt
            | MetaOp::SyntaxString
            | MetaOp::SyntaxChild
    )
}

/// Execute a primitive. `args` follow the IR calling convention: the context
/// pointer (when `has_context`), the parameters (memory types by address),
/// then result addresses for memory-typed results.
pub fn call(
    shared: &SharedWorkspaces,
    op: MetaOp,
    has_context: bool,
    args: &[u64],
    program: &ir::Program,
    interp: &mut Interp,
) -> Result<Rets, Trap> {
    // Hooks reach the compiler, which only the thread holding the baton may touch.
    #[cfg(not(target_arch = "wasm32"))]
    debug_assert!(interp.holds_baton(), "{op:?} called without the baton");
    let args = if has_context {
        &args[1..]
    } else {
        args
    };
    if is_effect(op) {
        interp.effects += 1;
    }
    let cx = Call {
        shared,
        program,
        args,
    };
    match Family::of(op) {
        Family::Workspace => workspace::call(&cx, op, interp),
        Family::Messages => messages::call(&cx, op, interp),
        Family::Sources => sources::call(&cx, op, interp),
        Family::Host => host::call(&cx, op, interp),
        Family::ProgramData => program_data::call(&cx, op, interp),
        Family::Records => records::call(&cx, op, interp),
    }
}

/// Store `bytes` at program address `p`, or the error for one that cannot be written.
pub(super) fn store(interp: &mut Interp, p: u64, bytes: &[u8]) -> Result<(), Trap> {
    if interp.write(p, bytes) {
        Ok(())
    } else {
        Err(Trap::unwritable(p, bytes.len()))
    }
}

/// Store a Jai `string` or array view (`count`, `data`) at program address `p`.
pub(super) fn store_pair(interp: &mut Interp, p: u64, count: u64, data: u64) -> Result<(), Trap> {
    store(interp, p, &count.to_le_bytes())?;
    store(interp, p + 8, &data.to_le_bytes())
}
