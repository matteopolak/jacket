// Builds jaifmt through the Compiler module (see docs/tools/jaifmt.md):
//
//     jaic build jaifmt/build.jai                    native, optimised: target/jaifmt
//     jaic build jaifmt/build.jai - wasm             WASI module: target/jaifmt.wasm
//     jaic build jaifmt/build.jai - -o <file>        choose the output file
//     jaic build jaifmt/build.jai - --help           the options (parsed by Extensions/Args)
//
// Use `jaic build`, not `jaic run`: only `build` writes the workspaces a metaprogram creates.
// The default output is the repository's target/ directory, wherever jaic was started. jaic runs
// metaprograms from their own directory, so a relative `-o` path is relative to jaifmt/.
#import "Basic";
#import "Compiler";
#import "String";
Args :: #import "Extensions/Args";

Target :: enum {
    NATIVE;
    WASM;
}

Options :: struct {
    target: Target; @positional
    output: string; @short=o @value=FILE
}

HELP :: Args.Help(Options).{
    about = "Builds jaifmt (pass these after `-`).",
    target = "native, or wasm for a WASI module",
    output = "The output file [default: target/jaifmt or target/jaifmt.wasm]",
};

#run build_jaifmt();

build_jaifmt :: () {
    set_build_options_dc(.{ do_output = false });

    argv: [..] string;
    array_add(*argv, "jaifmt/build.jai");
    array_add(*argv, ..get_build_options().compile_time_command_line);
    options, error := Args.try_parse(Options, HELP, argv);
    if error.kind == .HELP {
        print("%", error.text);
        return;
    }
    if error.kind != .NONE {
        compiler_report(tprint("%; usage: %", error.message, error.usage));
        return;
    }
    wasm := options.target == .WASM;
    output := options.output;
    if !output output = tprint("%../target/%", #filepath, ifx wasm then "jaifmt.wasm" else "jaifmt");

    w := compiler_create_workspace(ifx wasm then "jaifmt.wasm" else "jaifmt");
    build := get_build_options(w);
    copy_commonly_propagated_fields(get_build_options(), *build);
    build.output_type = .EXECUTABLE;
    build.emit_debug_info = .NONE;
    set_optimization(*build, .OPTIMIZED);
    path, name := split_output(output);
    build.output_path = path;
    build.output_executable_name = name;
    source := "main.jai";
    if wasm {
        // A WASI command: "wasi" in the triple makes jaic link its Wasi_Runtime module.
        build.os_target = .WASM;
        build.cpu_target = .CUSTOM;
        build.llvm_options.target_system_triple = "wasm64-unknown-wasi";
        source = "wasm.jai";
    }
    set_build_options(build, w);
    add_build_file(tprint("%1%2", #filepath, source), w);
}

// "dir/name" -> "dir", "name"; a bare name stays in the current directory.
split_output :: (output: string) -> string, string {
    slash := find_index_from_right(output, #char "/");
    if slash < 0 return ".", output;
    return slice(output, 0, slash), slice(output, slash + 1, output.count - slash - 1);
}
