#import "Compiler";
#run { w := compiler_create_workspace(); add_build_string("main :: (", w); add_build_file("/nonexistent", w); add_build_string("", -5); }
main :: () {}
