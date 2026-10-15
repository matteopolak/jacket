// Original producer-mechanism probe; no library/producer source is changed.
#import "Basic";
#import "Compiler";

counter: s64;

enter_hook :: () {
    counter += 1;
}

leave_hook :: () {
    counter += 100;
}

#run {
    for mode, index: Optimization_Type.[.DEBUG, .OPTIMIZED] {
        workspace := compiler_create_workspace("Original procedure instrumentation probe");
        options := get_build_options(workspace);

        copy_commonly_propagated_fields(get_build_options(), *options);
        set_optimization(*options, mode);
        options.output_type = .EXECUTABLE;
        options.output_path = tprint("%../../../target", #filepath);
        options.output_executable_name = tprint("developer-tools-procedure-edit-o%", ifx index == 0 then 0 else 2);
        set_build_options(options, workspace);
        compiler_begin_intercept(workspace);
        add_build_file(tprint("%procedure-edit-input.jai", #filepath), workspace);
        edited := false;

        while true {
            message := compiler_wait_for_message();
            if !message break;

            if message.kind == .TYPECHECKED {
                for tc: (cast(*Message_Typechecked) message).procedure_bodies {
                    if !tc.expression.header || tc.expression.header.name != "original_target" || edited continue;

                    prefix := compiler_get_nodes(#code {
                        enter_hook();
                        defer leave_hook();
                    });

                    assert(prefix.kind == .BLOCK);
                    block := cast(*Code_Block) prefix;
                    statements: [..] *Code_Node;

                    for node: block.statements array_add(*statements, node);
                    for node: tc.expression.block.statements array_add(*statements, node);

                    tc.expression.block.statements = .{ statements.count, statements.data };
                    compiler_modify_procedure(workspace, tc.expression);
                    edited = true;
                }
            }

            if message.kind == .COMPLETE {
                assert((cast(*Message_Complete) message).error_code == .NONE);
                break;
            }
        }

        compiler_end_intercept(workspace);
        assert(edited);
    }
}

main :: () {
    print("ok\n");
}
