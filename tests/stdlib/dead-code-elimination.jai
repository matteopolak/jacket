// Which unreferenced code is type-checked under `Build_Options.dead_code_elimination`. The modes
// that reject a program are negative corpus cases (tests/corpus/negative/rule-dce-*.jai): a
// workspace that fails makes the whole compile fail.
// rules: dce.4 dce.5 dce.6 dce.7 dce.9
#import "Basic";
#import "Compiler";
// Its bodies that nothing calls have type errors and a failing #run, one of them in a file the
// module loads: none is checked by default (dce.6, dce.7).
#import "Dce_Unreferenced";

// Nothing below is used. With the default mode the program's own files are checked, but a
// polymorphic procedure waits for an instance and a macro for an expansion (dce.4)...
generic_unused :: (x: $T) -> T {
    wrong: int = "a polymorphic body is checked per instance";
    return x;
}

expand_unused :: () #expand {
    wrong: int = "a macro body is checked per expansion";
}

// ...and an inactive #if branch declares nothing (dce.5).
#if false {
    inactive :: () {
        wrong: int = "an inactive branch is never checked";
    }
}

// A program with an error only in a procedure nothing calls.
UNUSED_BODY :: #string END
unused :: () { x: int = "wrong"; }
main :: () {}
END

Mode :: type_of(Build_Options.dead_code_elimination);

// Whether `source` compiles in a workspace of its own with dead-code elimination `mode`.
compiles :: (source: string, mode: Mode) -> bool {
    w := compiler_create_workspace("dce");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.dead_code_elimination = mode;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(source, w);
    ok := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            ok = (cast(*Message_Complete) message).error_code == .NONE;
            break;
        }
    }
    compiler_end_intercept(w);
    return ok;
}

#run {
    assert(get_build_options().dead_code_elimination == .MODULES_ONLY);
    // .ALL checks only what the program reaches (dce.9).
    assert(compiles(UNUSED_BODY, .ALL));
}

main :: () {
    assert(twice(21) == 42);
    print("ok\n");
}
