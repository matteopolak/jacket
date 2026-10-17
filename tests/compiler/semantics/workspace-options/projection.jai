C :: #import "Compiler";
#import "Basic";

#run {
    C.set_build_options_dc(.{ do_output = false });
    child := C.compiler_create_workspace("projection");
    options := C.get_build_options(child);
    options.output_type = .NO_OUTPUT;
    options.llvm_options.bitcode_optimization_setting = .O3;
    options.stack_trace = false;
    C.set_build_options(options, child);
    assert(C.get_build_options(child).llvm_options.bitcode_optimization_setting == .O3);
    C.add_build_string("C :: #import \"Compiler\"; #import \"Basic\"; #run { options := C.get_build_options(); assert(options.llvm_options.bitcode_optimization_setting == .O3); assert(!options.stack_trace); } main :: () {}", child);
}

main :: () {}
