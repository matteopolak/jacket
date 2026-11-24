#import "Basic";
#import "Compiler";

// The program (workspace 2) compiles this file again in a workspace of its own and reads the
// notes of the procedures that workspace typechecks.
#run {
    if get_current_workspace() != 2 return;

    w := compiler_create_workspace("notes");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_file(#file, w);

    while true {
        message := compiler_wait_for_message();
        if message.kind == .TYPECHECKED {
            for item: (cast(*Message_Typechecked) message).procedure_headers {
                for note: item.expression.notes {
                    if note.text == "Exported" print("exported %\n", item.expression.name);
                }
            }
        }

        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(w);
}

exported :: () -> int {
    return 42;
} @Exported

// The compiler reads these notes itself.
log :: (format: string, args: ..Any) {
    print(format, ..args);
} @PrintLike

cold :: () {} @Cold

main :: () {
    log("%\n", exported());
    cold();
}
