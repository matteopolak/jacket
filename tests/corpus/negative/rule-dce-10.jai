// Even with dead_code_elimination = .ALL every struct declaration is laid out.
#import "Compiler";

PROGRAM :: #string JAI
Unused :: struct { member: Missing_Member_Type; }
main :: () {}
JAI

#run {
    w := compiler_create_workspace("all dce");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.dead_code_elimination = .ALL;
    set_build_options(options, w);
    add_build_string(PROGRAM, w);
}
