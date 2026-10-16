// Independent of Minimal_Metaprogram: selected child settings must remain visible there.
#import "Basic";
C :: #import "Compiler";

#run,stallable {
    w := C.compiler_create_workspace("options projection reduction");

    assert(w >= 0);
    options := C.get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.stack_trace = false;
    options.llvm_options.bitcode_optimization_setting = .O3;
    C.set_build_options(options, w);
    stored := C.get_build_options(w);

    assert(!stored.stack_trace && stored.llvm_options.bitcode_optimization_setting == .O3);
    C.add_build_string("C :: #import \"Compiler\"; #import \"Basic\"; #run { o := C.get_build_options(); assert(!o.stack_trace); assert(o.llvm_options.bitcode_optimization_setting == .O3); }", w);
    C.set_build_options_dc(.{ do_output = false, write_added_strings = false });
}
