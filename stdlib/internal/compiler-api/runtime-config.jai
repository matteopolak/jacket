// Private compiler/runtime protocol. Read at initialization, after build options
// have settled; this is not a public Runtime_Support module parameter.
temporary_storage_size :: () -> u32 #no_context {
    return __jaic_runtime_temporary_storage_size();
}

#scope_file
__jaic_runtime_temporary_storage_size :: () -> u32 #intrinsic #no_context;
