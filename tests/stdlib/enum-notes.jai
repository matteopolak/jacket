// rules: note.9
// Notes on an enum declaration, before or after the base type and directives,
// reach metaprograms as Code_Enum.notes.
#import "Basic";
#import "Compiler";

Hue :: enum_flags @Hi {
    R;
    G;
}

Size :: enum u8 @Small @Tiny {
    S;
    M;
}

PROGRAM :: #string JAI
Hue :: enum_flags @Hi @There { R; G; }
main :: () {}
JAI

#run {
    w := compiler_create_workspace("notes");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    ok := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .TYPECHECKED {
            checked := cast(*Message_Typechecked) message;
            for checked.declarations {
                e := it.expression.expression;
                if e && it.expression.name == "Hue" && e.kind == .ENUM {
                    notes := (cast(*Code_Enum) e).notes;
                    ok = notes.count == 2 && notes[0].text == "Hi" && notes[1].text == "There";
                }
            }
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    assert(ok, "Code_Enum.notes");
}

main :: () {
    assert(cast(int) Hue.G == 2 && cast(int) Size.M == 1, "noted enums behave normally");
    print("ok\n");
}
