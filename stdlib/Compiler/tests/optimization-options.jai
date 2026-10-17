// Compile-time build settings mutate only the documented option fields.
#import "Basic";
Compiler :: #import "Compiler";

main :: () {
    options: Compiler.Build_Options;

    assert(options.output_type == .EXECUTABLE);
    Compiler.set_optimization(*options, .DEBUG);
    assert(options.llvm_options.bitcode_optimization_setting == .O0);
    assert(options.llvm_options.machine_code_optimization_setting == .NONE);
    assert(options.array_bounds_check == .ON);
    assert(options.cast_bounds_check == .FATAL && options.null_pointer_check == .ON);
    assert(!options.llvm_options.enable_loop_unrolling);

    Compiler.set_optimization(*options, .VERY_OPTIMIZED, preserve_debug_info = false);
    assert(options.llvm_options.bitcode_optimization_setting == .O3);
    assert(options.llvm_options.machine_code_optimization_setting == .AGGRESSIVE);
    assert(options.llvm_options.enable_loop_unrolling);
    assert(options.llvm_options.enable_slp_vectorization && options.llvm_options.enable_loop_vectorization);
    assert(options.emit_debug_info == .NONE && !options.llvm_options.preserve_debug_info);
    print("ok\n");
}
