// LLVM 22 operations without C API equivalents. Rust validates owned handles.
#include <llvm/BinaryFormat/Dwarf.h>
#include <llvm/IR/DIBuilder.h>
#include <llvm/IR/DebugProgramInstruction.h>
#include <llvm/IR/Metadata.h>

extern "C" void jai_llvm_erase_debug_record(LLVMDbgRecordRef record) {
    llvm::unwrap(record)->eraseFromParent();
}

extern "C" bool jai_llvm_set_function_signature(LLVMMetadataRef scope,
    LLVMMetadataRef signature) {
    auto *function = llvm::dyn_cast<llvm::DISubprogram>(llvm::unwrap(scope));
    auto *type = llvm::dyn_cast<llvm::DISubroutineType>(llvm::unwrap(signature));
    if (!function || !type) return false;
    function->replaceType(type);
    return true;
}

extern "C" LLVMMetadataRef jai_llvm_subroutine_type(
    LLVMDIBuilderRef builder, LLVMMetadataRef *types, unsigned count,
    unsigned convention) {
    auto *debug = llvm::unwrap(builder);
    auto array = llvm::ArrayRef<LLVMMetadataRef>(types, count);
    llvm::SmallVector<llvm::Metadata *, 16> parameters;
    for (auto type : array) parameters.push_back(llvm::unwrap(type));
    auto signature = debug->getOrCreateTypeArray(parameters);
    unsigned cc;
    switch (convention) {
        case 0: cc = llvm::dwarf::DW_CC_normal; break;
        case 1: cc = llvm::dwarf::DW_CC_BORLAND_stdcall; break;
        case 2: cc = llvm::dwarf::DW_CC_BORLAND_thiscall; break;
        default: return nullptr;
    }
    return llvm::wrap(debug->createSubroutineType(signature,
        llvm::DINode::FlagPrototyped, cc));
}
