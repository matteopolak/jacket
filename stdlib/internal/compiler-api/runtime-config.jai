// Private compiler/runtime protocol. Read at initialization, after build options
// have settled; this is not a public Runtime_Support module parameter.
temporary_storage_size :: () -> s64 #no_context {
    return __jaic_runtime_temporary_storage_size();
}

#scope_file
__jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
