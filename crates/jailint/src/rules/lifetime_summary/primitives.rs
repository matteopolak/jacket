//! Ownership at foreign ABI boundaries. Wrapper names play no part in inference.
use super::{Effect, HashMap, Kind, Path, ProcId, Root, Span, Summary};
use jaic::sema::Compiler;

/// A selected native contract, not a live allocation or a dynamic identity.
/// The caller still proves positive size, nonnull success and use-epoch liveness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Allocation {
    foreign: jaic::ir::ForeignId,
    signature: jaic::types::TypeId,
    pub alignment: u64,
}

impl Allocation {
    pub fn current(&self, c: &Compiler, id: ProcId) -> bool {
        native_allocation(c, id).as_ref() == Some(self)
    }
}

pub(super) struct Boundary<'a> {
    pub(super) foreign_id: jaic::ir::ForeignId,
    pub(super) foreign: &'a jaic::ir::Foreign,
    pub(super) library: &'a jaic::ir::Library,
    pub(super) signature: &'a jaic::sema::procs::Signature,
}

/// Authenticate the actual fixed C calling convention before interpreting a symbol.
pub(super) fn selected_c_boundary(c: &Compiler, id: ProcId) -> Option<Boundary<'_>> {
    use jaic::sema::procs::ProcTarget;
    use jaic::target::TargetCpu;
    use jaic::types::TypeKind;
    if !matches!(c.options.cpu, TargetCpu::X64 | TargetCpu::Arm64) {
        return None;
    }
    let p = c.pr.procs.get(id.0 as usize)?;
    let ProcTarget::Foreign(fid) = p.target? else {
        return None;
    };
    let foreign = c.out.program.foreigns.get(fid.0 as usize)?;
    let library = c.out.program.libraries.get(foreign.library?)?;
    let sig = p.sig.as_ref()?;
    if foreign.is_data
        || p.lit.body.is_some()
        || !sig.c_call
        || sig.has_context
        || sig.c_varargs
        || sig.params.len() > 8
        || sig.returns.len() > 1
        || sig.params.iter().any(|p| p.variadic)
    {
        return None;
    }
    let TypeKind::Proc(abi) = c.ty.types.kind(sig.ty) else {
        return None;
    };
    if !abi.c_call
        || !abi.no_context
        || abi.non_pod_return
        || abi.variadic
        || abi.c_varargs
        || abi.params.len() != sig.params.len()
        || abi.params.iter().zip(&sig.params).any(|(a, p)| *a != p.ty)
        || abi.returns != sig.returns
    {
        return None;
    }
    let lowered = &foreign.sig;
    if lowered.conv != jaic::ir::Conv::C
        || lowered.c_varargs
        || lowered.c_abi.is_some()
        || u32::try_from(sig.params.len()).ok() != Some(lowered.c_fixed)
        || lowered.params.len() != sig.params.len()
        || lowered.returns.len() != sig.returns.len()
        || lowered
            .params
            .iter()
            .zip(&sig.params)
            .any(|(actual, p)| c.ir_ty(p.ty) != Some(*actual))
        || lowered
            .returns
            .iter()
            .zip(&sig.returns)
            .any(|(actual, ty)| c.ir_ty(*ty) != Some(*actual))
    {
        return None;
    }
    Some(Boundary {
        foreign_id: fid,
        foreign,
        library,
        signature: sig,
    })
}

/// System C primitives keep their stronger library policy; private declarations
/// may separately carry an explicit, authenticated external-effect promise.
fn c_boundary(c: &Compiler, id: ProcId) -> Option<Boundary<'_>> {
    let boundary = selected_c_boundary(c, id)?;
    boundary.library.system.then_some(boundary)
}

fn unix_c(c: &Compiler, library: &jaic::ir::Library) -> bool {
    use jaic::target::TargetOs;
    match c.options.os {
        TargetOs::Linux => matches!(library.name.as_str(), "c" | "libc"),
        TargetOs::MacOS => matches!(library.name.as_str(), "c" | "libc" | "System"),
        _ => false,
    }
}

fn native_allocation(c: &Compiler, id: ProcId) -> Option<Allocation> {
    use jaic::types::TypeId;
    let boundary = c_boundary(c, id)?;
    let sig = boundary.signature;
    if !unix_c(c, boundary.library)
        || boundary.foreign.symbol != "malloc"
        || sig.params.len() != 1
        || c.ty.types.int_info(sig.params[0].ty) != Some((64, false))
        || sig.returns.len() != 1
        || c.ty.types.pointee(sig.returns[0]) != Some(TypeId::VOID)
        || c.ty.types.size_of(sig.returns[0]) != 8
    {
        return None;
    }
    Some(Allocation {
        foreign: boundary.foreign_id,
        signature: sig.ty,
        alignment: 8,
    })
}

fn c_runtime(c: &Compiler, library: &jaic::ir::Library) -> bool {
    unix_c(c, library)
        || (c.options.os == jaic::target::TargetOs::Windows
            && ["c", "libc", "msvcrt", "ucrtbase"]
                .iter()
                .any(|name| library.name.eq_ignore_ascii_case(name)))
}

/// The selected standard C abort ABI never returns normally. No wrapper or
/// similarly named private library gets that property from its spelling.
pub(super) fn native_abort(c: &Compiler, id: ProcId) -> Option<()> {
    let b = c_boundary(c, id)?;
    (b.foreign.symbol == "abort"
        && c_runtime(c, b.library)
        && b.signature.params.is_empty()
        && b.signature.returns.is_empty())
    .then_some(())
}

/// A same-named unknown foreign function cannot provide a must deallocation.
pub(in crate::rules) fn native_free(c: &Compiler, id: ProcId) -> Option<()> {
    let boundary = c_boundary(c, id)?;
    let sig = boundary.signature;
    (boundary.foreign.symbol == "free"
        && c_runtime(c, boundary.library)
        && sig.params.len() == 1
        && sig.returns.is_empty()
        && c.ty.types.is_pointer(sig.params[0].ty)
        && c.ty.types.size_of(sig.params[0].ty) == 8)
        .then_some(())
}

pub(super) fn seed(c: &Compiler, id: ProcId, summary: &mut Summary) {
    summary.native_allocation = native_allocation(c, id)
        .map(super::native_returns::Allocation::Foreign)
        .map(std::rc::Rc::new);
    let Some(symbol) = super::observable::symbol(c, id) else {
        return;
    };
    if symbol == "free" && native_free(c, id).is_none() {
        return;
    }
    let formal = match symbol {
        "free"
        | "cfree"
        | "fclose"
        | "close"
        | "closesocket"
        | "munmap"
        | "VirtualFree"
        | "CloseHandle"
        | "freeaddrinfo"
        | "LocalFree"
        | "GlobalFree"
        | "RegCloseKey"
        | "developer_debug_free_symbols" => 0,
        "VirtualFreeEx" => 1,
        "HeapFree" => 2,
        _ => return,
    };
    if c.pr.procs[id.0 as usize]
        .sig
        .as_ref()
        .is_none_or(|s| s.params.len() <= formal)
    {
        return;
    }
    let effect = Effect {
        target: Path {
            root: Root::Parameter(formal),
            steps: Vec::new(),
        },
        kind: Kind::Free,
        guard: None,
    };
    let witness = vec![c.pr.procs[id.0 as usize].span];
    insert(&mut summary.may, effect.clone(), witness.clone());
    // Fallible native releases may leave a valid resource live on failure.
    // A result-dependent success proof is required before exporting a must release.
    if matches!(
        symbol,
        "free" | "cfree" | "fclose" | "freeaddrinfo" | "developer_debug_free_symbols"
    ) {
        insert(&mut summary.guaranteed, effect, witness);
    }
}

fn insert(map: &mut HashMap<Effect, Vec<Span>>, effect: Effect, witness: Vec<Span>) {
    map.insert(effect, witness);
}
