#program_export "configuration_check"
check :: () -> s32 #c_call {
    if __jaic_runtime_temporary_storage_size() != 65536 return 1;
    getter: () -> s64 #no_context = __jaic_runtime_temporary_storage_size;
    if getter() != 65536 return 2;
    return 0;
}

main :: () {}

#scope_file
__jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
