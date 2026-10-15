// Independent producer request: actual foreign-header flags at TYPECHECKED.
#import "Basic";
#import "Compiler";

#run {
    workspace := compiler_create_workspace("Original foreign metadata probe");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);
    compiler_begin_intercept(workspace);
    add_build_string("libc :: #foreign_library \"c\";\noriginal_abs :: (s32) -> s32 #foreign libc \"abs\";\nmain :: () {}\n", workspace);
    checked := false;

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.workspace == workspace && message.kind == .TYPECHECKED {
            for item: (cast(*Message_Typechecked) message).procedure_headers {
                header := item.expression;
                if header.name != "original_abs" continue;

                print("foreign header metadata: flags=% foreign_name_bytes=% procedure_type_flags=%\n", cast(u32) header.procedure_flags, header.foreign_function_name.count, cast(u32) (cast(*Type_Info_Procedure) header.type).procedure_flags);
                assert((header.procedure_flags & .ELSEWHERE) != 0);
                assert(((cast(*Type_Info_Procedure) header.type).procedure_flags & .IS_ELSEWHERE) != 0);
                checked = true;
            }
        }

        if message.workspace == workspace && message.kind == .COMPLETE break;
    }

    compiler_end_intercept(workspace);
    assert(checked);
}

main :: () {}
