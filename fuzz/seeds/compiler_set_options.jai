#import "Compiler";
#run { w := compiler_create_workspace(); o := get_build_options(w); o.output_executable_name = ""; o.import_path = .[]; set_build_options(o, w); set_build_options(o, 99999); }
main :: () {}
