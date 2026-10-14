#import "Basic";
C :: #import "Compiler";

#run {
    w := C.compiler_create_workspace("orchid contract");
    assert(C.get_name(w) == "orchid contract");
    options := C.get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.temporary_storage_size = 65536;
    options.user_data_u64 = 47;
    C.set_optimization(*options, .VERY_DEBUG);
    C.set_build_options(options, w);
    got := C.get_build_options(w);
    assert(got.output_type == .NO_OUTPUT && got.temporary_storage_size == 65536);
    assert(got.user_data_u64 == 47 && got.llvm_options.bitcode_optimization_setting == .O0);
    C.add_build_string("#assert(13 + 29 == 42);", w);
    discarded := C.compiler_create_workspace("discarded");
    C.add_build_string("#assert(false);", discarded);
    C.compiler_destroy_workspace(discarded);
}

main :: () { print("ok\n"); }
