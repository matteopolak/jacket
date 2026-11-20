#import "Basic";
#import "Compiler";

#run {
    set_build_options_dc(.{ do_output = false });
    w := compiler_create_workspace("freestanding");
    options := get_build_options(w);
    options.output_executable_name = "prog.elf";
    options.os_target = .NONE;
    options.cpu_target = .ARM64;
    options.llvm_options.target_system_triple = "aarch64-none-elf";
    options.runtime_support_definitions = .OMIT;
    LINKER_ARGUMENTS
    set_build_options(options, w);
    add_build_file("prog.jai", w);
}
