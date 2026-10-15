#import "Compiler";

#run {
    workspace := compiler_create_workspace("Independent typed expression metadata");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.runtime_support_definitions = .OMIT;
    set_build_options(options, workspace);
    compiler_begin_intercept(workspace);
    add_build_string("#program_export \"sum\" sum :: (a:u8,b:u8)->u8 #c_call { return a+b; } main :: () {}", workspace);
    count := 0;

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.kind == .TYPECHECKED {
            for tc: (cast(*Message_Typechecked) message).procedure_bodies {
                for node: tc.subexpressions {
                    if node.kind != .BINARY_OPERATOR continue;
                    if !node.type || node.type != type_info(u8)
                        compiler_report("Expected canonical u8 result descriptor");
                    count += 1;
                }
            }
        }

        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(workspace);
    if count != 1 compiler_report("Expected exactly one checked binary result");
}

main :: () {}
