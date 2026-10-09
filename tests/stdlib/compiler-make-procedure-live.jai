// rules: compiler.25
// compiler_make_procedure_live keeps a procedure nothing calls: with dead_code_elimination = .ALL
// the workspace lowers only what the program reaches, so the body of `spare` shows up in a
// TYPECHECKED message only when the metaprogram asked for it.
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
#import "Basic";
spare :: () { print("spare\n"); }
other :: () { print("other\n"); }
main :: () {}
JAI

// The names of the procedure bodies the workspace reported, `spare` made live or not.
bodies_of :: (live: bool) -> [] string {
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.dead_code_elimination = .ALL;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    names: [..] string;
    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .TYPECHECKED;
                typechecked := cast(*Message_Typechecked) message;
                for typechecked.declarations {
                    decl := it.expression;
                    if live && decl.name == "spare" && decl.expression.kind == .PROCEDURE_HEADER {
                        compiler_make_procedure_live(w, cast(*Code_Procedure_Header) decl.expression);
                    }
                }
                for typechecked.procedure_bodies array_add(*names, copy_string(it.expression.header.name));
            case .COMPLETE;
                assert((cast(*Message_Complete) message).error_code == .NONE);
                break;
        }
    }
    compiler_end_intercept(w);
    return names;
}

contains :: (names: [] string, wanted: string) -> bool {
    for names if it == wanted return true;
    return false;
}

checks :: () {
    plain := bodies_of(false);
    assert(!contains(plain, "spare") && !contains(plain, "other"));

    live := bodies_of(true);
    assert(contains(live, "spare"));
    assert(!contains(live, "other"));
    print("ok\n");
}

// Compile-time only: the Compiler module's procedures need the compiler.
#run checks();

main :: () {}
