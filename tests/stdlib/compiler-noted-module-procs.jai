// A metaprogram that intercepts messages sees the bodies of the program's own noted
// procedures even when nothing calls them, but an imported module's uncalled noted
// procedures stay unchecked (Vk-Engine's Common module has one whose body is stale).
// rules: records.22 records.23 records.28
#import "Basic";
#import "Compiler";

TARGET :: #string END
#import "Noted_Unused";
local_only :: () -> int { z := 4; return z; } @Tagged
main :: () { bump(1); }
END

#run {
    options := get_build_options();
    child: Build_Options;
    copy_commonly_propagated_fields(options, *child);
    paths: [..] string;
    array_add(*paths, "modules");
    child.import_path = paths;
    child.output_type = .NO_OUTPUT;
    w := compiler_create_workspace("noted");
    set_build_options(child, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    saw_local := false;
    saw_stale := false;
    failed := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            failed = (cast(*Message_Complete) message).error_code != .NONE;
            break;
        }
        if message.kind != .TYPECHECKED continue;
        for (cast(*Message_Typechecked) message).procedure_bodies {
            name := it.expression.header.name;
            if name == "local_only" saw_local = true;
            if name == "stale_wrapper" saw_stale = true;
        }
    }
    compiler_end_intercept(w);
    assert(!failed, "the uncalled module procedure was checked");
    assert(saw_local);
    assert(!saw_stale);
    set_build_options_dc(.{ do_output = false });
}

main :: () {}
