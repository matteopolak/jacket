#import "Compiler";
#run { compiler_begin_intercept(-1); m := compiler_wait_for_message(); compiler_end_intercept(12345); }
main :: () {}
