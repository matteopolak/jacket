// The metaprogramming API: create and drive builds ("workspaces") from compile-time code, read and edit the program as syntax trees, and listen to what the compiler does.
//
// - [compiler_create_workspace] starts a build; [add_build_string] and [add_build_file] give it code; [set_build_options] says what to produce.
// - [compiler_begin_intercept] and [compiler_wait_for_message] let you watch the build as [Message]s.
// - [Code_Node] and its `Code_*` relatives are the syntax tree; [compiler_get_code] and [compiler_modify_procedure] read and rewrite it.
// - [Build_Options] holds every setting of a build; [set_optimization] fills in a preset.
//
// ```jai
// #run {
//     w := compiler_create_workspace("demo");
//     options := get_build_options(w);
//     options.output_type = .NO_OUTPUT;
//     set_build_options(options, w);
//     add_build_string("value :: 47; main :: () {}", w);
//
//     compiler_begin_intercept(w);
//
//     while true {
//         message := compiler_wait_for_message();
//         if message.kind == .COMPLETE break;
//     }
//
//     compiler_end_intercept(w);
// }
//
// main :: () {}
// ```

#load "contract-records.jai";
#load "helpers.jai";
#load "workspace.jai";

OPERATOR_TYPE_MAX_ASCII_VALUE :: 127;

#scope_file
Impl :: #import "internal/compiler-api";
