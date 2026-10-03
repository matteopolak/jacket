//! Refresh selected generic and local prototype bindings after argument resolution.
use super::*;
use jai_ir::PrototypeOrigin;
use std::collections::HashSet;

pub(crate) struct Bindings {
    pub process_abi: HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>,
    pub heap_abi: HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>,
    pub file_abi: HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>,
    pub runtime: HashMap<ProcedureId, jai_vm::RuntimeProcedure>,
    pub foreign: HashSet<ProcedureId>,
}
pub(crate) fn snapshot(context: &Context<'_>, meta: &crate::reflection::MetaContext) -> Bindings {
    let mut bindings = Bindings {
        file_abi: context.file_abi.clone(),
        heap_abi: context.heap_abi.clone(),
        process_abi: context.process_abi.clone(),
        runtime: context.runtime.clone(),
        foreign: context.foreign.clone(),
    };
    let generated = context.generics.borrow().prototype_snapshot();
    for prototype in generated
        .into_iter()
        .chain(meta.local_declarations.prototypes())
    {
        match prototype.origin {
            PrototypeOrigin::Intrinsic(intrinsic) => {
                bindings.runtime.insert(
                    prototype.id,
                    jai_vm::RuntimeProcedure {
                        signature: prototype.signature,
                        intrinsic,
                    },
                );
            }
            PrototypeOrigin::Foreign { .. } | PrototypeOrigin::SourceContract { .. } => {
                bindings.foreign.insert(prototype.id);
            }
            PrototypeOrigin::Compiler => {}
        }
    }
    bindings
}
