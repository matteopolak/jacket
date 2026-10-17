#import "Compiler";

#run {
    workspace := compiler_create_workspace("Original typed binary descriptor request");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);
    compiler_begin_intercept(workspace);
    add_build_file("../rejections/check-small-input.jai", workspace);

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.kind == .TYPECHECKED {
            for tc: (cast(*Message_Typechecked) message).procedure_bodies {
                for node: tc.subexpressions if node.kind == .BINARY_OPERATOR && !node.type
                    compiler_report("Typed binary result descriptor is missing from a TYPECHECKED procedure body");
            }
        }

        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(workspace);
}

main :: () {}
