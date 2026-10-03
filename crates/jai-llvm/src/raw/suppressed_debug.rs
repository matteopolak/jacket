//! Normalize only inlined locations anchored in this compiler's artificial-call ledger.
use inkwell::{
    debug_info::DILocation,
    llvm_sys::{core::*, debuginfo::*, prelude::*},
    module::Module,
};
use std::{collections::HashSet, ptr};
const LEDGER: &[u8] = b"jai.suppressed.calls\0";
unsafe extern "C" {
    fn jai_llvm_erase_debug_record(record: LLVMDbgRecordRef);
}

pub(crate) fn retain(module: &Module<'_>, location: DILocation<'_>) {
    // SAFETY: DebugSession constructed this live DILocation using its exact
    // borrowed module/context and checked owning function scope.
    unsafe {
        LLVMAddNamedMetadataOperand(
            module.as_mut_ptr(),
            LEDGER.as_ptr().cast(),
            LLVMMetadataAsValue(
                LLVMGetModuleContext(module.as_mut_ptr()),
                location.as_mut_ptr(),
            ),
        );
    }
}

pub(crate) fn normalize(module: &Module<'_>) {
    // SAFETY: All LLVM values are enumerated from the borrowed owning module.
    // Location kinds are checked before any DILocation-specific query. Record
    // handles are live members of each instruction's own record range.
    unsafe {
        let module = module.as_mut_ptr();
        let count = LLVMGetNamedMetadataNumOperands(module, LEDGER.as_ptr().cast());
        if count == 0 {
            return;
        }
        let mut values = vec![ptr::null_mut(); count as usize];
        LLVMGetNamedMetadataOperands(module, LEDGER.as_ptr().cast(), values.as_mut_ptr());
        let anchors: HashSet<_> = values
            .into_iter()
            .filter_map(|value| {
                if value.is_null() {
                    return None;
                }
                let metadata = LLVMValueAsMetadata(value);
                (!metadata.is_null()
                    && matches!(
                        LLVMGetMetadataKind(metadata),
                        LLVMMetadataKind::LLVMDILocationMetadataKind
                    )
                    && LLVMDILocationGetLine(metadata) == 0
                    && LLVMDILocationGetColumn(metadata) == 0)
                    .then(|| LLVMDILocationGetScope(metadata))
            })
            .collect();
        if anchors.is_empty() {
            return;
        }
        let anchor = |location: LLVMMetadataRef| {
            if location.is_null()
                || !matches!(
                    LLVMGetMetadataKind(location),
                    LLVMMetadataKind::LLVMDILocationMetadataKind
                )
            {
                return None;
            }
            let mut parent = LLVMDILocationGetInlinedAt(location);
            let mut seen = HashSet::new();
            while !parent.is_null() && seen.insert(parent) {
                if !matches!(
                    LLVMGetMetadataKind(parent),
                    LLVMMetadataKind::LLVMDILocationMetadataKind
                ) {
                    break;
                }
                // Inlining creates a distinct location with the same owned
                // caller scope and zero coordinates.
                if LLVMDILocationGetLine(parent) == 0
                    && LLVMDILocationGetColumn(parent) == 0
                    && anchors.contains(&LLVMDILocationGetScope(parent))
                {
                    return Some(parent);
                }
                parent = LLVMDILocationGetInlinedAt(parent);
            }
            None
        };
        let mut function = LLVMGetFirstFunction(module);
        while !function.is_null() {
            let mut block = LLVMGetFirstBasicBlock(function);
            while !block.is_null() {
                let mut instruction = LLVMGetFirstInstruction(block);
                while !instruction.is_null() {
                    let next = LLVMGetNextInstruction(instruction);
                    let mut record = LLVMGetFirstDbgRecord(instruction);
                    while !record.is_null() {
                        let next_record = LLVMGetNextDbgRecord(record);
                        if anchor(LLVMDbgRecordGetDebugLoc(record)).is_some() {
                            jai_llvm_erase_debug_record(record);
                        }
                        record = next_record;
                    }
                    let location = LLVMInstructionGetDebugLoc(instruction);
                    if let Some(parent) = anchor(location) {
                        LLVMInstructionSetDebugLoc(instruction, parent);
                    }
                    instruction = next;
                }
                block = LLVMGetNextBasicBlock(block);
            }
            function = LLVMGetNextFunction(function);
        }
    }
}
