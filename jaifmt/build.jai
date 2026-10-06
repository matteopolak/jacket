// Builds jaifmt through the Compiler module (see docs/tools/jaifmt.md):
//
//     jaic build jaifmt/build.jai                    native, optimised: target/jaifmt
//     jaic build jaifmt/build.jai - wasm             WASI module: target/jaifmt.wasm
//     jaic build jaifmt/build.jai - -o <file>        choose the output file
//
// Use `jaic build`, not `jaic run`: only `build` writes the workspaces a metaprogram creates.
// The default output is the repository's target/ directory, wherever jaic was started. jaic runs
// metaprograms from their own directory, so a relative `-o` path is relative to jaifmt/.
#import "Basic";
#import "Compiler";
#import "String";

#run build_jaifmt();

build_jaifmt :: () {
    set_build_options_dc(.{ do_output = false });

    wasm := false;
    output := "";
    args := get_build_options().compile_time_command_line;
    i := 0;
    while i < args.count {
        arg := args[i];
        if arg == "wasm" {
            wasm = true;
        } else if arg == "-o" && i + 1 < args.count {
            i += 1;
            output = args[i];
        } else {
            compiler_report(tprint("unknown argument '%' (use wasm or -o <file>)", arg));
            return;
        }
        i += 1;
    }
    if !output output = tprint("%../target/%", #filepath, ifx wasm then "jaifmt.wasm" else "jaifmt");

    w := compiler_create_workspace(ifx wasm then "jaifmt.wasm" else "jaifmt");
    options := get_build_options(w);
    copy_commonly_propagated_fields(get_build_options(), *options);
    options.output_type = .EXECUTABLE;
    options.emit_debug_info = .NONE;
    set_optimization(*options, .OPTIMIZED);
    path, name := split_output(output);
    options.output_path = path;
    options.output_executable_name = name;
    source := "main.jai";
    if wasm {
        // A WASI command: "wasi" in the triple makes jaic link its Wasi_Runtime module.
        options.os_target = .WASM;
        options.cpu_target = .CUSTOM;
        options.llvm_options.target_system_triple = "wasm64-unknown-wasi";
        source = "wasm.jai";
    }
    set_build_options(options, w);
    add_build_file(tprint("%1%2", #filepath, source), w);
}

// "dir/name" -> "dir", "name"; a bare name stays in the current directory.
split_output :: (output: string) -> string, string {
    slash := find_index_from_right(output, #char "/");
    if slash < 0 return ".", output;
    return slice(output, 0, slash), slice(output, slash + 1, output.count - slash - 1);
}
