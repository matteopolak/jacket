// A TYPECHECKED message reports the body of every ordinary procedure, also one nothing calls,
// and marks file-level declarations IS_GLOBAL (SCOPE_FILE below #scope_file) so a metaprogram can
// tell them from locals.
// rules: records.6 records.32
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
#import "Basic";
counter := 0;
used :: () { counter += 1; }
never_called :: (x: int) -> int { local := x * 2; return local; }
main :: () { used(); }
#scope_file
hidden := 3;
JAI

#run {
    w := compiler_create_workspace("flags");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    unused_body := false;
    global_counter := false;
    file_hidden := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
        if message.kind != .TYPECHECKED continue;
        typechecked := cast(*Message_Typechecked) message;
        for typechecked.procedure_bodies {
            if it.expression.header.name == "never_called" unused_body = true;
        }
        for typechecked.declarations {
            decl := it.expression;
            if decl.name == "counter" global_counter = (decl.flags & .IS_GLOBAL) != 0;
            if decl.name == "hidden" file_hidden = (decl.flags & .SCOPE_FILE) != 0 && (decl.flags & .IS_GLOBAL) != 0;
        }
    }
    compiler_end_intercept(w);
    assert(unused_body, "the body of an uncalled procedure is reported");
    assert(global_counter, "a global variable is IS_GLOBAL");
    assert(file_hidden, "a #scope_file declaration is SCOPE_FILE");
    set_build_options_dc(.{ do_output = false });
}

main :: () {}
