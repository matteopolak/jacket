// A workspace a metaprogram builds and intercepts checks the stdlib bodies its program calls, and
// sends them as typechecked messages, even when the enclosing compile skips library bodies (an
// editor check).
#import "Basic";
#import "Compiler";

#run {
    w := compiler_create_workspace("Intercepted stdlib body");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_string("#import \"Basic\";\nmain :: () { p := alloc(8); free(p); }\n", w);

    seen := false;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            if tc.expression && tc.expression.header && tc.expression.header.name == "alloc" seen = true;
        }

        if event.kind == .COMPLETE break;
    }

    compiler_end_intercept(w);
    assert(seen, "the intercepted workspace sent no body for alloc");
}

main :: () {}
