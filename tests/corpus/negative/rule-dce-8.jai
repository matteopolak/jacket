// With dead_code_elimination = .NONE a module's bodies are checked though nothing calls them.
#import "Compiler";

PROGRAM :: #string JAI
#import "Dce_Called";
main :: () { fine(); }
JAI

#run {
    w := compiler_create_workspace("no dce");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.dead_code_elimination = .NONE;
    set_build_options(options, w);
    add_build_string(PROGRAM, w);
}
