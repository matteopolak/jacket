// Builds program.jai into a WASI module through the Compiler module, the way a metaprogram
// targets WebAssembly: os_target .WASM, cpu_target .CUSTOM, an LLVM triple and feature string,
// and extra wasm-ld arguments. The "wasi" in the triple makes jaic add Wasi_Runtime.
#import "Basic";
#import "Compiler";

#run {
    set_build_options_dc(.{ do_output = false });
    w := compiler_create_workspace("wasm");
    options := get_build_options(w);
    copy_commonly_propagated_fields(get_build_options(), *options);
    options.output_type = .EXECUTABLE;
    options.backend = .LLVM;
    options.os_target = .WASM;
    options.cpu_target = .CUSTOM;
    options.emit_debug_info = .NONE;
    options.output_executable_name = "program";
    options.llvm_options.target_system_triple = "wasm64-unknown-wasi";
    options.llvm_options.target_system_features = "+bulk-memory,+sign-ext";
    options.additional_linker_arguments = .["-z", "stack-size=1048576"];
    set_build_options(options, w);
    add_build_file(tprint("%program.jai", #filepath), w);
}
