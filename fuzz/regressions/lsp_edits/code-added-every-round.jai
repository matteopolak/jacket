// Top-level code that needs a `#placeholder` waits until the metaprogram defines it
// (here at TYPECHECKED_ALL_WE_CAN), instead of failing.
// rules: code.15
#import "Basic";
#import "Compiler";

PROGRAM :: #string END
#import "Basic";
#placeholder Kinds;

#insert -> string {
    return tprint("Kind_Count :: %;", Kinds.count);
}

#if Kind_Count == 3 {
    Ready :: true;
}

main :: () { #assert Ready; }
END

#run {
    w := compiler_create_workspace("t");
    opts := get_build_options(w);
    opts.output_type = .NO_OUTPUT;
    set_build_options(opts, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    added := false;
    failed := false;
    while true {
        msg := compiler_wait_for_message();
        if msg.kind == .PHASE && !added {
            phase := cast(*Message_Phase) msg;
            if phase.phase == .TYPECHECKED_ALL_WE_CAN {
                add_build_string("Kinds :: string.[\"a\", \"b\", \"c\"];", w);
              a  dded = true;
            }
        }
        if msg.kind == .COMPLETE {
            failed = (cast(*Message_Complete) msg).error_code != .NONE;
            break;
        }
    }
    compiler_end_intercept(w);
    assert(added && !failed);
}

main :: () {
    print("ok\n");
}
