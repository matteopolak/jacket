//! Metadata constructors are reachable only through session-owned checked types.
use crate::debug_records::{
    DebugCallingConvention, DebugVariableKind,
    types::{DebugRecordKind, RecordSeed},
};
use inkwell::{
    debug_info::{AsDIScope, DICompileUnit, DIFile, DIScope, DebugInfoBuilder},
    llvm_sys::{
        debuginfo::*,
        prelude::{LLVMDIBuilderRef, LLVMMetadataRef},
    },
};
use std::ptr;
unsafe extern "C" {
    fn jai_llvm_set_function_signature(scope: LLVMMetadataRef, signature: LLVMMetadataRef) -> bool;
    fn jai_llvm_subroutine_type(
        builder: LLVMDIBuilderRef,
        types: *mut LLVMMetadataRef,
        count: u32,
        convention: u32,
    ) -> LLVMMetadataRef;
}

pub(crate) fn set_function_signature(scope: DIScope<'_>, signature: LLVMMetadataRef) -> bool {
    // SAFETY: The sealed session owns both live metadata handles in the same
    // context. The shim checks their concrete kinds before replacing the type.
    unsafe { jai_llvm_set_function_signature(scope.as_mut_ptr(), signature) }
}

pub(crate) fn subroutine(
    debug: &DebugInfoBuilder<'_>,
    signature: &mut [LLVMMetadataRef],
    convention: DebugCallingConvention,
) -> LLVMMetadataRef {
    // SAFETY: Each non-null entry is a live same-session storage DIType; only
    // the void return and optional C ellipsis are null. Count fits the C API.
    unsafe {
        jai_llvm_subroutine_type(
            debug.as_mut_ptr(),
            signature.as_mut_ptr(),
            signature.len() as u32,
            match convention {
                DebugCallingConvention::Normal => 0,
                DebugCallingConvention::X86Stdcall => 1,
                DebugCallingConvention::CppThiscall => 2,
            },
        )
    }
}

pub(crate) fn result_member(
    debug: &DebugInfoBuilder<'_>,
    unit: &DICompileUnit<'_>,
    ty: LLVMMetadataRef,
    size: u64,
    alignment: u32,
    offset: u64,
) -> LLVMMetadataRef {
    // SAFETY: Unit and child type belong to this session, and member placement
    // has been checked against the target tuple extent. There is no source name.
    unsafe {
        LLVMDIBuilderCreateMemberType(
            debug.as_mut_ptr(),
            unit.as_debug_info_scope().as_mut_ptr(),
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            size,
            alignment,
            offset,
            LLVMDIFlagArtificial,
            ty,
        )
    }
}

pub(crate) fn result_tuple(
    debug: &DebugInfoBuilder<'_>,
    unit: &DICompileUnit<'_>,
    size: u64,
    alignment: u32,
    members: &mut [LLVMMetadataRef],
) -> LLVMMetadataRef {
    // SAFETY: All members were built from checked same-session storage types.
    // This unnamed artificial aggregate describes compiler-generated return storage.
    unsafe {
        LLVMDIBuilderCreateStructType(
            debug.as_mut_ptr(),
            unit.as_debug_info_scope().as_mut_ptr(),
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            size,
            alignment,
            LLVMDIFlagArtificial,
            ptr::null_mut(),
            members.as_mut_ptr(),
            members.len() as u32,
            0,
            ptr::null_mut(),
            ptr::null(),
            0,
        )
    }
}

pub(crate) fn pointer(
    debug: &DebugInfoBuilder<'_>,
    pointee: LLVMMetadataRef,
    size: u64,
    alignment: u32,
) -> LLVMMetadataRef {
    // SAFETY: Session lookup proved live same-context pointee metadata, or an
    // explicit void pointee. Target sizes and alignments are checked before entry.
    unsafe {
        LLVMDIBuilderCreatePointerType(
            debug.as_mut_ptr(),
            pointee,
            size,
            alignment,
            0,
            ptr::null(),
            0,
        )
    }
}
pub(crate) fn array(
    debug: &DebugInfoBuilder<'_>,
    element: LLVMMetadataRef,
    count: i64,
    size: u64,
    alignment: u32,
) -> LLVMMetadataRef {
    // SAFETY: Same-session element is a live DIType; count is checked nonnegative.
    // The single subrange and element/aggregate widths are checked by the facade.
    unsafe {
        let mut range = LLVMDIBuilderGetOrCreateSubrange(debug.as_mut_ptr(), 0, count);
        LLVMDIBuilderCreateArrayType(debug.as_mut_ptr(), size, alignment, element, &mut range, 1)
    }
}
pub(crate) fn record(
    debug: &DebugInfoBuilder<'_>,
    unit: &DICompileUnit<'_>,
    seed: &RecordSeed<'_>,
    size: u64,
    alignment: u32,
    members: Option<&mut Vec<LLVMMetadataRef>>,
) -> LLVMMetadataRef {
    // SAFETY: The unit/file originate in this session. Members, when present,
    // were created from same-session type slots and checked aggregate offsets.
    unsafe {
        match members {
            None => LLVMDIBuilderCreateReplaceableCompositeType(
                debug.as_mut_ptr(),
                match seed.kind {
                    DebugRecordKind::Struct => 0x13,
                    DebugRecordKind::Union => 0x17,
                },
                seed.name.as_ptr().cast(),
                seed.name.len(),
                unit.as_debug_info_scope().as_mut_ptr(),
                seed.file.as_mut_ptr(),
                seed.line,
                0,
                size,
                alignment,
                LLVMDIFlagFwdDecl,
                seed.identifier.as_ptr().cast(),
                seed.identifier.len(),
            ),
            Some(elements) => {
                complete(debug, unit, seed, size, alignment, elements, LLVMDIFlagZero)
            }
        }
    }
}
pub(crate) fn forward(
    debug: &DebugInfoBuilder<'_>,
    unit: &DICompileUnit<'_>,
    seed: &RecordSeed<'_>,
    size: u64,
    alignment: u32,
) -> LLVMMetadataRef {
    // SAFETY: A genuine named/anonymous source record is retained as a forward
    // declaration when its public construction token was not completed.
    unsafe {
        complete(
            debug,
            unit,
            seed,
            size,
            alignment,
            &mut [],
            LLVMDIFlagFwdDecl,
        )
    }
}
unsafe fn complete(
    debug: &DebugInfoBuilder<'_>,
    unit: &DICompileUnit<'_>,
    seed: &RecordSeed<'_>,
    size: u64,
    alignment: u32,
    elements: &mut [LLVMMetadataRef],
    flags: LLVMDIFlags,
) -> LLVMMetadataRef {
    // SAFETY: Only record/forward call this helper with their owned checked inputs.
    unsafe {
        match seed.kind {
            DebugRecordKind::Struct => LLVMDIBuilderCreateStructType(
                debug.as_mut_ptr(),
                unit.as_debug_info_scope().as_mut_ptr(),
                seed.name.as_ptr().cast(),
                seed.name.len(),
                seed.file.as_mut_ptr(),
                seed.line,
                size,
                alignment,
                flags,
                ptr::null_mut(),
                elements.as_mut_ptr(),
                elements.len() as u32,
                0,
                ptr::null_mut(),
                seed.identifier.as_ptr().cast(),
                seed.identifier.len(),
            ),
            DebugRecordKind::Union => LLVMDIBuilderCreateUnionType(
                debug.as_mut_ptr(),
                unit.as_debug_info_scope().as_mut_ptr(),
                seed.name.as_ptr().cast(),
                seed.name.len(),
                seed.file.as_mut_ptr(),
                seed.line,
                size,
                alignment,
                flags,
                elements.as_mut_ptr(),
                elements.len() as u32,
                0,
                seed.identifier.as_ptr().cast(),
                seed.identifier.len(),
            ),
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn member(
    debug: &DebugInfoBuilder<'_>,
    scope: LLVMMetadataRef,
    name: &str,
    file: DIFile<'_>,
    line: u32,
    ty: LLVMMetadataRef,
    size: u64,
    alignment: u32,
    offset: u64,
) -> LLVMMetadataRef {
    // SAFETY: Pending record scope is a real DICompositeType, not an empty tuple;
    // type and file are session-owned. Byte/bit arithmetic was checked beforehand.
    unsafe {
        LLVMDIBuilderCreateMemberType(
            debug.as_mut_ptr(),
            scope,
            name.as_ptr().cast(),
            name.len(),
            file.as_mut_ptr(),
            line,
            size,
            alignment,
            offset,
            LLVMDIFlagZero,
            ty,
        )
    }
}
pub(crate) fn replace(temporary: LLVMMetadataRef, replacement: LLVMMetadataRef) {
    // SAFETY: The session proves a one-shot live replaceable composite. LLVM 22
    // replaces all uses AND deletes this temporary. The slot is updated immediately;
    // no raw handle is exposed or reused after this call.
    unsafe { LLVMMetadataReplaceAllUsesWith(temporary, replacement) }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn variable(
    debug: &DebugInfoBuilder<'_>,
    scope: DIScope<'_>,
    file: DIFile<'_>,
    line: u32,
    name: &str,
    kind: DebugVariableKind,
    ty: LLVMMetadataRef,
    alignment: u32,
) -> LLVMMetadataRef {
    // SAFETY: Scope/file/type were minted by and checked against the same session.
    // Parameter ordinal and alignment were checked by the safe factory.
    unsafe {
        match kind {
            DebugVariableKind::Automatic => LLVMDIBuilderCreateAutoVariable(
                debug.as_mut_ptr(),
                scope.as_mut_ptr(),
                name.as_ptr().cast(),
                name.len(),
                file.as_mut_ptr(),
                line,
                ty,
                1,
                LLVMDIFlagZero,
                alignment,
            ),
            DebugVariableKind::Parameter(ordinal) => LLVMDIBuilderCreateParameterVariable(
                debug.as_mut_ptr(),
                scope.as_mut_ptr(),
                name.as_ptr().cast(),
                name.len(),
                ordinal.get(),
                file.as_mut_ptr(),
                line,
                ty,
                1,
                LLVMDIFlagZero,
            ),
        }
    }
}
