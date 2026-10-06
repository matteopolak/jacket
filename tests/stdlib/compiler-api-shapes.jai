// Shape probe for the Compiler module: only type-checks the API surface used by metaprograms.
#import "Basic";
#import "Compiler";

options_shapes :: () {
    options := get_build_options();
    options.output_type = .EXECUTABLE;
    options.output_executable_name = "probe";
    set_optimization(*options, .OPTIMIZED);
    set_optimization(*options, .DEBUG, preserve_debug_info = false);
    options.llvm_options.output_llvm_ir = true;
    options.array_bounds_check = .OFF;
    options.os_target = OS;
    child := get_build_options(compiler_create_workspace("child"));
    copy_commonly_propagated_fields(options, *child);
    paths: [..] string;
    array_add(*paths, "modules");
    child.import_path = paths;
    child.compile_time_command_line = options.compile_time_command_line;
    w := compiler_create_workspace("probe");
    set_build_options(child, w);
    set_build_options_dc(.{ do_output = false, output_path = "out" });
    add_build_file("main.jai", w);
    add_build_string("x :: 1;", w);
    compiler_custom_link_command_is_complete(w);
    info: Version_Info;
    text := compiler_get_version_info(*info);
    if text.count > 0 && info.major >= 0 compiler_report("x", mode = .WARNING);
    compiler_report("file", 1, 2, "text");
    ws := get_current_workspace();
    compiler_set_workspace_status(.FAILED, ws);
}

message_shapes :: (w: Workspace) {
    compiler_begin_intercept(w, .SKIP_ALL);
    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .FILE;
                file := cast(*Message_File) message;
                print("%\n", file.fully_pathed_filename);
            case .IMPORT;
                import := cast(*Message_Import) message;
                print("%\n", import.module_name);
            case .PHASE;
                phase := cast(*Message_Phase) message;
                if phase.phase == .READY_FOR_CUSTOM_LINK_COMMAND || phase.phase == .PRE_WRITE_EXECUTABLE {
                    for phase.compiler_generated_object_files print("%\n", it);
                }
            case .TYPECHECKED;
                typechecked := cast(*Message_Typechecked) message;
                for typechecked.all {
                    node := it.expression;
                    if node.kind == .DECLARATION {
                        decl := cast(*Code_Declaration) node;
                        if decl.name == "x" compiler_report(make_location(node).fully_pathed_filename);
                    }
                }
            case .COMPLETE;
                complete := cast(*Message_Complete) message;
                if complete.error_code != .NONE compiler_set_workspace_status(.FAILED, w);
                break;
        }
    }
    compiler_end_intercept(w);
}

ast_shapes :: (code: Code) {
    root, expressions := compiler_get_nodes(code);
    if root && root.kind == .IDENT {
        ident := cast(*Code_Ident) root;
        print("% %\n", ident.name, expressions.count);
    }
    back := compiler_get_code(root);
    print("%\n", code_to_string(back));
    builder: String_Builder;
    print_expression(*builder, root);
    literal := make_integer_literal(3);
    text_literal := make_string_literal("a");
    print("% % %\n", literal._s64, text_literal._string, operator_to_string(.IS_EQUAL));
    base, path := get_basename_and_path("a/b/c.foo.txt");
    print("% %\n", base, path);
    table := get_type_table();
    print("% %\n", table.count, is_subclass_of(table[0], "X"));
    status, found_type := get_root_type(code);
    if status == .SUCCESS print("%\n", found_type);
    print("%\n", compiler_get_base_path());
    print("%\n", get_toplevel_command_line().count);
    print("%\n", get_name(1));
    compiler_modify_procedure(1, null);
    loc := compiler_get_struct_location(1, null);
    print("%\n", loc.line_number);
    plugin: Metaprogram_Plugin;
    plugin.workspace = 1;
    flags: Intercept_Flags = .SKIP_DECLARATIONS | .SKIP_STRUCTS;
    print("%\n", cast(u32) flags);
}

probe_enabled := false;

main :: () {
    if probe_enabled {
        options_shapes();
        message_shapes(1);
        ast_shapes(#code 1 + 2);
    }
}
