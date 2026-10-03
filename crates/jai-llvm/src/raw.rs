//! All unsafe LLVM calls in this crate are confined to these checked operations.
use super::{CheckedComparison, CheckedConstGep, CheckedGep, Error};
use inkwell::{
    llvm_sys::core::{LLVMBuildICmp, LLVMTypeIsSized},
    types::{AsTypeRef, BasicTypeEnum},
    values::{AsValueRef, IntValue, PointerValue},
};
use std::ffi::CString;
pub(super) mod debug_types;
pub(super) mod suppressed_debug;

pub(super) fn function_in_module(
    function: inkwell::values::FunctionValue<'_>,
    module: &inkwell::module::Module<'_>,
) -> bool {
    // SAFETY: The live typed function handle can be queried without dereferencing
    // a foreign module. The comparison proves ownership by the borrowed module.
    unsafe {
        inkwell::llvm_sys::core::LLVMGetGlobalParent(function.as_value_ref()) == module.as_mut_ptr()
    }
}

pub(super) fn debug_storage_in_owner(
    storage: PointerValue<'_>,
    function: inkwell::values::FunctionValue<'_>,
    module: &inkwell::module::Module<'_>,
) -> bool {
    use inkwell::llvm_sys::core::{
        LLVMGetBasicBlockParent, LLVMGetGlobalParent, LLVMGetInstructionParent, LLVMGetParamParent,
        LLVMIsAArgument, LLVMIsAGlobalVariable, LLVMIsAInstruction,
    };
    // SAFETY: The live pointer is classified before each kind-specific ownership
    // query. Unsupported constants (including GEP expressions) are never traversed.
    unsafe {
        let value = storage.as_value_ref();
        if !LLVMIsAArgument(value).is_null() {
            LLVMGetParamParent(value) == function.as_value_ref()
        } else if !LLVMIsAGlobalVariable(value).is_null() {
            LLVMGetGlobalParent(value) == module.as_mut_ptr()
        } else if !LLVMIsAInstruction(value).is_null() {
            let block = LLVMGetInstructionParent(value);
            !block.is_null() && LLVMGetBasicBlockParent(block) == function.as_value_ref()
        } else {
            false
        }
    }
}

pub(super) fn debug_declare(
    checked: super::debug_records::CheckedDebugDeclare<'_, '_>,
) -> Result<(), Error> {
    use inkwell::llvm_sys::{
        core::{LLVMGetGlobalParent, LLVMIsNewDbgInfoFormat},
        debuginfo::LLVMDIBuilderInsertDeclareRecordAtEnd,
    };
    let function = checked.block.get_parent().ok_or(Error::NullResult)?;
    // SAFETY: The checked block has a live owning function. Reading its module
    // requires no value reinterpretation and the nonnull module is checked below.
    let module = unsafe { LLVMGetGlobalParent(function.as_value_ref()) };
    if module.is_null() {
        return Err(Error::NullResult);
    }
    // SAFETY: The function's live parent is a valid module.
    if unsafe { LLVMIsNewDbgInfoFormat(module) } == 0 {
        return Err(Error::InvalidDebugFormat);
    }
    // SAFETY: Live typed Inkwell metadata handles were produced by the owning
    // debug builder, and the private input proves a positioned block and matching
    // storage context. LLVM returns a DbgRecord: it is never cast to LLVMValueRef.
    let record = unsafe {
        LLVMDIBuilderInsertDeclareRecordAtEnd(
            checked.debug.as_mut_ptr(),
            checked.storage.as_value_ref(),
            checked.variable,
            checked.expression.as_mut_ptr(),
            checked.location.as_mut_ptr(),
            checked.block.as_mut_ptr(),
        )
    };
    if record.is_null() {
        return Err(Error::NullResult);
    }
    Ok(())
}

pub(super) fn sized(ty: BasicTypeEnum<'_>) -> bool {
    // SAFETY: Inkwell's live type handle owns a valid LLVM type in its borrowed context.
    unsafe { LLVMTypeIsSized(ty.as_type_ref()) != 0 }
}
pub(super) fn gep<'ctx>(checked: CheckedGep<'_, 'ctx>) -> Result<PointerValue<'ctx>, Error> {
    // SAFETY: The private checked value proves a sized element, positioned builder,
    // shared context, and valid aggregate index path. No inbounds promise is made.
    unsafe {
        checked
            .builder
            .build_gep(
                checked.element,
                checked.pointer,
                checked.indices,
                checked.name,
            )
            .map_err(Error::from)
    }
}
pub(super) fn const_gep<'ctx>(checked: CheckedConstGep<'_, 'ctx>) -> PointerValue<'ctx> {
    // SAFETY: The private input proves a sized element, shared live context,
    // constant operands and valid structural indices. No inbounds promise is made.
    unsafe { checked.pointer.const_gep(checked.element, checked.indices) }
}
pub(super) fn compare<'ctx>(checked: CheckedComparison<'_, 'ctx>) -> Result<IntValue<'ctx>, Error> {
    let name = CString::new(checked.name).map_err(|_| Error::InvalidName)?;
    // SAFETY: The private checked value proves equal pointer types, a shared live
    // context and a positioned builder. LLVM accepts scalar pointer comparisons.
    let value = unsafe {
        LLVMBuildICmp(
            checked.builder.as_mut_ptr(),
            checked.predicate.into(),
            checked.left.as_value_ref(),
            checked.right.as_value_ref(),
            name.as_ptr(),
        )
    };
    if value.is_null() {
        return Err(Error::NullResult);
    }
    // SAFETY: Successful LLVMBuildICmp returns a nonnull scalar i1 value.
    Ok(unsafe { IntValue::new(value) })
}
