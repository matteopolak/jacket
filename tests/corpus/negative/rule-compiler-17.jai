// rules: compiler.17
// An API call jaic cannot carry out reports a fatal error once it is actually made.
#import "Compiler";

#run add_build_string("X :: 1;", get_current_workspace(), #code 1 + 1);

main :: () {}
