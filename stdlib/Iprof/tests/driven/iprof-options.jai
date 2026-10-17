#import "Basic";
#import "Compiler";
I :: #import "Iprof"(USE_GRAPHICS = false);

errors: s64;

capture :: (text: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

#run {
    invalid: [6][] string = .[
        .["-unknown"], .["-csv"], .["-min_size"],
        .["-min_size", "12tail"], .["-min_size", "9223372036854775808"],
        .["-min_size", ""]];

    push_context {
        context.logger = capture;

        for options: invalid {
            p := I.get_plugin();
            assert(type_of(p) == *Metaprogram_Plugin);
            assert(!I.init(p, options));
            I.shutdown(p);
        }
    }

    assert(errors == 6);
    p := I.get_plugin();

    assert(I.init(p, .["-min_size", " \t-2 \n", "-modules"]));
    assert(p.init == I.init && p.before_intercept == I.before_intercept && p.add_source == I.add_source && p.message == I.message);
    assert(p.finish == I.finish && p.shutdown == I.shutdown && p.log_help == I.log_help);
    flags: Intercept_Flags = .SKIP_ALL;

    I.before_intercept(p, *flags);
    assert(!(flags & (.SKIP_EXPRESSIONS_WITHOUT_NOTES | .SKIP_PROCEDURE_HEADERS | .SKIP_PROCEDURE_BODIES)));
    assert((flags & .SKIP_DECLARATIONS) != 0);
    I.shutdown(p);
    // Shutdown releases canonical plugin activity so a subsequent session works.
    p = I.get_plugin();
    assert(I.init(p, .[]));
    I.shutdown(p);
}

main :: () {
    print("ok\n");
}
