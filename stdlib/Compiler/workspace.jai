// Starts a new workspace, an independent build with its own options, code and output. `name` shows up in messages and errors.
//
// ```jai
// #run {
//     w := compiler_create_workspace("scratch");
//     assert(get_name(w) == "scratch");
//     compiler_destroy_workspace(w);
// }
//
// main :: () {}
// ```
compiler_create_workspace :: (name := "") -> Workspace #compiler;

// Frees a workspace made with [compiler_create_workspace]. Do it after its build is complete.
compiler_destroy_workspace :: (w: Workspace) #compiler;

compiler_set_memory_breakpoint :: (pointer: *void) #compiler;

developer_debug :: (x: *void) #compiler;

// Returns the name of workspace `w`; the default is the workspace running the code.
get_name :: (w: Workspace = -1) -> string #compiler;

// Returns a copy of the options of workspace `w`; the default is the workspace running the code. Change the copy and hand it back with [set_build_options].
get_build_options :: (w: Workspace = -1) -> Build_Options #compiler;

// Replaces the options of workspace `w`. Set them before adding code.
//
// ```jai
// #run {
//     w := compiler_create_workspace("options");
//     options := get_build_options(w);
//     options.output_type = .NO_OUTPUT;
//     set_optimization(*options, .OPTIMIZED);
//     set_build_options(options, w);
//     assert(get_build_options(w).output_type == .NO_OUTPUT);
//     compiler_destroy_workspace(w);
// }
//
// main :: () {}
// ```
set_build_options :: (options: Build_Options, w: Workspace = -1, loc := #caller_location) #compiler;

// Changes the few options that may still change while a build is running, such as its output name and path. `Build_Options_During_Compile` says which.
set_build_options_dc :: (options: Build_Options_During_Compile, w: Workspace = -1) #compiler;

// Adds the source file `filename` to workspace `w`, as if it were passed on the command line.
//
// ```jai,skip_on(wasm)
// #import "File";
//
// #run {
//     path := tprint("%/extra.jai", #filepath);
//     write_entire_file(path, "extra :: 1;");
//     w := compiler_create_workspace("from file");
//     options := get_build_options(w);
//     options.output_type = .NO_OUTPUT;
//     set_build_options(options, w);
//     add_build_file(path, w);
// }
//
// main :: () {}
// ```
add_build_file :: (filename: string, w: Workspace, loc := #caller_location) #compiler;

// Adds the source text `data` to workspace `w`. Pass `code` to compile it in the scope that code was written in, so it can see that scope's names.
//
// ```jai
// #run {
//     w := compiler_create_workspace("strings");
//     options := get_build_options(w);
//     options.output_type = .NO_OUTPUT;
//     set_build_options(options, w);
//     add_build_string("answer :: 42; main :: () {}", w);
// }
//
// main :: () {}
// ```
add_build_string :: (data: string, w: Workspace, code := #code,null, loc := #caller_location) #compiler "add_build_string_scoped_by_code";

// Adds the source text `data` to workspace `w`, in the scope of the code that `message` is about, so the new code can see the names around it.
add_build_string :: (data: string, w: Workspace, message: *Message, loc := #caller_location) #compiler "add_build_string_scoped_by_message";

// Reports a problem at `loc` (by default the call site). The default mode, `.ERROR`, fails the build once the current step ends; `.WARNING` and `.INFO` only print.
//
// ```jai
// #run compiler_report("heads up", mode = .INFO);
//
// main :: () {}
// ```
compiler_report :: (message: string, loc := #caller_location, mode := Report.ERROR) #compiler;

// Reports a problem at an explicit file, line and character instead of a code location.
compiler_report :: (filename: string, line_number: s64, character_number: s64, error_text: string, mode := Report.ERROR) #compiler;

// Fills `version_info` with the compiler's version numbers and returns the version as a string.
compiler_get_version_info :: (version_info_return: *Version_Info) -> string #compiler;

// Returns the arguments of the command line the compiler was started with, so a build script can read its own options.
get_toplevel_command_line :: () -> [] string #compiler;

// Returns the folder the compiler installation lives in.
compiler_get_base_path :: () -> string #compiler;

// Adds `path` to the folders searched for native libraries named by `#library`.
compiler_add_library_search_directory :: (path: string) #compiler;

compiler_set_type_info_flags :: (type: Type, flags: Type_Info_Flags) #compiler;

compiler_get_struct_location :: (w: Workspace, info: *Type_Info_Struct) -> Source_Code_Location #compiler;

// Makes `#import "import_name"` inside `host_module_name` load `replacement_name` instead. Use it from a message handler to swap in another version of a module.
remap_import :: (w: Workspace, host_module_name: string, import_name: string, replacement_name: string) #compiler;

// Answers a [Message_Failed_Import]: tells the compiler where to find the module it could not. `type` says whether `value` is a module name, a file, a folder or the full source text.
provide_import :: (w: Workspace, message: *Message_Failed_Import, type: Provided_Import_Type, value: string) #compiler;

// Returns the syntax tree of `code` as its root node and the flat list of expressions in it.
compiler_get_nodes :: (code: Code) -> (root: *Code_Node, expressions: [] *Code_Node) #compiler;

// Turns the node `node` back into a `Code` value you can insert with `#insert`. `code_to_copy_scope_from` says which scope the new code is read in.
compiler_get_code :: (node: *Code_Node, code_to_copy_scope_from: Code = #code,null) -> Code #compiler;

// Returns the type a typed `Code` value stands for, and a status that says why there is none when it fails.
get_root_type :: (code: Code) -> (status: Get_Root_Type_Status, type: Type) #compiler;

// Returns the `Type` that goes with a `Type_Info` pointer.
get_type :: (ti: *Type_Info) -> Type #compiler {
    return cast(Type) ti;
}

// Returns the `Type_Info` of every type in workspace `w`.
get_type_table :: (w: Workspace = -1) -> [] *Type_Info #compiler {
    return get_runtime_info(w).type_table;
}

// Makes sure the procedure `header` is compiled into the program even if nothing calls it.
compiler_make_procedure_live :: (w: Workspace, header: *Code_Procedure_Header) #compiler;

// Replaces the body of a procedure with the (edited) tree under `body`, then compiles it again.
compiler_modify_procedure :: (w: Workspace, body: *Code_Procedure_Body) #compiler;

// Starts receiving the compiler's [Message]s for workspace `w`. Read them with [compiler_wait_for_message] and finish with [compiler_end_intercept]. `flags` skips kinds of typechecked code you do not care about.
compiler_begin_intercept :: (w: Workspace, flags: Intercept_Flags = 0) #compiler;

// Stops receiving messages for `w`, which lets the build carry on to its end.
compiler_end_intercept :: (w: Workspace) #compiler;

// Marks workspace `w` as failed, so that building it ends with an error.
compiler_set_workspace_status :: (status: Workspace_Status, w: Workspace = -1) #compiler;

// Blocks until the compiler has a message for an intercepted workspace, and returns it. Check `kind` and cast to the matching `Message_*` type. The last message of a build is `.COMPLETE`.
//
// ```jai
// #run {
//     w := compiler_create_workspace("messages");
//     options := get_build_options(w);
//     options.output_type = .NO_OUTPUT;
//     set_build_options(options, w);
//     add_build_string("main :: () {}", w);
//
//     compiler_begin_intercept(w);
//     files := 0;
//     while true {
//         message := compiler_wait_for_message();
//         if message.kind == .FILE files += 1;
//         if message.kind == .COMPLETE break;
//     }
//     compiler_end_intercept(w);
//     assert(files > 0);
// }
//
// main :: () {}
// ```
compiler_wait_for_message :: () -> *Message #compiler;

compiler_custom_link_command_is_complete :: (w: Workspace) #compiler;

// Asks for a named data segment in the program being built. Returns it and whether the compiler will really create it on this target.
add_data_segment :: (section_name: string, characteristics := Data_Segment_Characteristics.READ | .WRITE, alignment: s32 = 16, w: Workspace = -1) -> (segment: *Data_Segment, actual_segment_will_be_created: bool) #compiler;

// Copies `data` into a data segment and returns the stored bytes, which live in the built program.
add_global_data :: (data: [] u8, segment: Data_Segment_Index, user_segment: *Data_Segment = null, w: Workspace = -1) -> [] u8 #compiler;

// Returns the type table and the global data layout of workspace `w`. Used by tools that read a program's memory.
get_runtime_info :: (w: Workspace = -1) -> Runtime_Info #compiler {
    return __runtime_info;
}

#scope_file
__runtime_info: Runtime_Info #elsewhere;
