// Original general-phase probe: source registration, typed metadata and native body replacement.
#import "Basic";
#import "Compiler";

#run {
    for mode, index: Optimization_Type.[.DEBUG, .OPTIMIZED] {
        workspace := compiler_create_workspace("Original late-source instrumentation probe");
        options := get_build_options(workspace);

        copy_commonly_propagated_fields(get_build_options(), *options);
        set_optimization(*options, mode);
        options.output_type = .EXECUTABLE;
        options.output_path = tprint("%../../../target", #filepath);
        options.output_executable_name = tprint("developer-tools-late-source-o%", ifx index == 0 then 0 else 2);
        set_build_options(options, workspace);
        compiler_begin_intercept(workspace);
        add_build_file(tprint("%late-source-input.jai", #filepath), workspace);
        target: *Code_Procedure_Body;
        added := false;
        edited := false;
        live_marked := false;
        phases: s64;

        while true {
            message := compiler_wait_for_message();
            if !message break;

            if message.kind == .TYPECHECKED {
                for tc: (cast(*Message_Typechecked) message).procedure_headers {
                    if tc.expression.name == "original_late_value" && !live_marked {
                        compiler_make_procedure_live(workspace, tc.expression);
                        live_marked = true;
                    }
                }

                for tc: (cast(*Message_Typechecked) message).procedure_bodies {
                    if !tc.expression.header continue;
                    if tc.expression.header.name == "original_target" target = tc.expression;

                    if tc.expression.header.name == "original_late_value" && !edited {
                        assert(added && target != null);
                        replacement := compiler_get_nodes(#code {
                            return original_late_value();
                        });

                        assert(replacement.kind == .BLOCK);
                        target.block.statements = (cast(*Code_Block) replacement).statements;
                        compiler_modify_procedure(workspace, target);
                        edited = true;
                    }
                }
            }

            if message.kind == .PHASE && (cast(*Message_Phase) message).phase == .TYPECHECKED_ALL_WE_CAN {
                phases += 1;

                if !added && target != null {
                    add_build_string("original_late_value :: () -> s64 { return 42; }", workspace, message = cast(*Message) null);
                    added = true;
                }
            }

            if message.kind == .COMPLETE {
                assert((cast(*Message_Complete) message).error_code == .NONE);
                break;
            }
        }

        compiler_end_intercept(workspace);
        assert(added && edited && live_marked && phases >= 2);
    }
}

main :: () {
    print("ok\n");
}
