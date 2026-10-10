// rules: bo.16
// Before a metaprogram sets anything, the program's own workspace reports where its output goes:
// the main file's name, its directory (with a trailing slash) and a `.build` folder beside it.
#import "Basic";
#import "Compiler";
#import "String";

#run {
    options := get_build_options();
    assert(options.output_executable_name == "build-options-main-defaults");
    assert(options.output_path == path_strip_filename(#file));
    assert(options.intermediate_path == tprint("%.build", options.output_path));
    // A new workspace starts blank.
    other := get_build_options(compiler_create_workspace("other"));
    assert(other.output_executable_name == "" && other.output_path == "");
}

main :: () {
    print("ok\n");
}
