use super::*;
pub(in crate::modules) struct HostMaps {
    pub(in crate::modules) files: HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>,
    pub(in crate::modules) heap: HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>,
    pub(in crate::modules) processes:
        HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>,
}
impl Worklist<'_> {
    pub(in crate::modules) fn host_maps(&self) -> HostMaps {
        HostMaps {
            files: self.file_abi.clone(),
            heap: self.heap_abi.clone(),
            processes: self.process_abi.clone(),
        }
    }
}
