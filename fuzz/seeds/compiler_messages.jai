#import "Compiler";
#run { w := compiler_create_workspace("x"); compiler_begin_intercept(w); add_build_string("main :: () {}", w); while true { m := compiler_wait_for_message(); if m.kind == .COMPLETE break; } compiler_end_intercept(w); }
main :: () {}
