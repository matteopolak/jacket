// Canonical procedure aliases serialize as references, rather than cloned bodies.
#import "Basic";
#import "Compiler";
PP :: #import "Program_Print";

#run {
    w := compiler_create_workspace("Original alias source identity");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_string("original :: () -> int {return 11;} alias :: original; main :: () {}", w);
    file: *Message;
    copied: bool;

    while true {
        event := compiler_wait_for_message();

        assert(event != null);

        if event.kind == .FILE {
            candidate := cast(*Message_File) event;
            if candidate.enclosing_import && candidate.enclosing_import.module_name == "main" file = event;
        }

        if event.kind == .TYPECHECKED for checked: (cast(*Message_Typechecked) event).declarations {
            declaration := checked.expression;
            if !copied && declaration.name == "alias" {
                assert(file != null);
                declaration.name = "alias_copy";
                text := PP.print_declaration(declaration);

                assert(text.count > 0);
                terminated := sprint("%;", text);

                add_build_string(terminated, w, message = file);
                free(terminated);
                add_build_string("#assert alias_copy == original; #assert alias_copy() == 11;", w, message = file);
                free(text);
                copied = true;
            }
        }

        if event.kind == .ERROR assert(false, "alias reconstruction must retain procedure identity");

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(copied);
}

main :: () {}
