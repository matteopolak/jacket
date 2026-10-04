// `#insert,scope(Top) -> string` inside a procedure declares into the file scope,
// and the declaration is reported to the metaprogram.
#import "Basic";
#import "Compiler";

Top :: #code();

generate :: () {
    #insert,scope(Top) -> string {
        return "Proxy :: struct { a: int; b: float; }";
    }
}

main :: () {
    generate();
    print("ok\n");
}

PROGRAM :: #string END
Top :: #code();
generate :: () {
    #insert,scope(Top) -> string { return "Proxy :: struct { a: int; } @Marker"; }
}
main :: () { generate(); }
END

#run {
    w := compiler_create_workspace("t");
    opts := get_build_options(w);
    opts.output_type = .NO_OUTPUT;
    set_build_options(opts, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    found := false;
    while true {
        msg := compiler_wait_for_message();
        if msg.kind == .TYPECHECKED {
            tc := cast(*Message_Typechecked) msg;
            for tc.declarations {
                for note: it.expression.notes if note.text == "Marker" found = true;
            }
        }
        if msg.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    assert(found, "Proxy was not reported");
}
