// rules: compiler.17
// An API call jaic cannot carry out reports a fatal error once it is actually made.
#import "Compiler";

#run {
    w := compiler_create_workspace("other");
    add_build_string("X :: 1;", w, #code 1 + 1);
}

main :: () {}
