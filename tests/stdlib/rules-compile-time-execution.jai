// rules: ctexec.1 ctexec.2 ctexec.3 ctexec.4 ctexec.5 ctexec.11 ctexec.12 ctexec.13
// Pins the claims of docs/metaprogramming/compile-time-execution.md.
#import "Basic";
#import "Compiler";
#import "File";
#import "Process";
#import "String";

// ctexec.1 / ctexec.2: the forms of #run and what their results become.
factorial :: (n: int) -> int {
    if n <= 1 return 1;
    return n * factorial(n - 1);
}
F10 :: #run factorial(10); // expression form, a constant
NAMES :: #run -> [3] string {
    return .["a", "b", "c"];
}; // body with a return type
CHOSEN :: #run -> Type {
    return u16;
}; // a Type
initialized_global := #run factorial(5); // a global initializer
#assert F10 == 3628800;
#assert CHOSEN == u16;

// ctexec.3: top-level #run directives run in declaration order; top-level #asserts are
// evaluated after all of them, so this one, written before the last #run, sees three steps.
#no_reset run_order: [3] int;
#no_reset run_steps := 0;
#run {
    run_order[run_steps] = 1;
    run_steps += 1;
}
#assert run_steps == 3;
#run {
    run_order[run_steps] = 2;
    run_steps += 1;
}
#run {
    run_order[run_steps] = 3;
    run_steps += 1;
}

// ctexec.5: #compile_time is true while compile-time code runs and false at run time.
PHASE :: #run -> bool {
    return #compile_time;
};
#assert PHASE;

// ctexec.11: compile-time code reads files and starts processes.
own_source_mentions_marker :: () -> bool {
    text, ok := read_entire_file(#file);
    if !ok return false;
    return contains(text, "ctexec-11-marker");
}
SOURCE_READ :: #run own_source_mentions_marker();
echo_output :: () -> string {
    result, output := run_command("echo", "from child", capture_and_return_output = true);
    if result.exit_code != 0 return "";
    return trim(output);
}
// The browser engine cannot start processes.
#if OS != .WASM CHILD_OUTPUT :: #run echo_output();

// ctexec.12: foreign procedures are callable at compile time. The browser engine has no C
// library to call into.
#if OS != .WASM {
    libc :: #system_library "libc";
    c_abs :: (x: s32) -> s32 #foreign libc "abs";
    FOREIGN_ABS :: #run c_abs(-12);
    #assert FOREIGN_ABS == 12;
}

// ctexec.13: `#run,stallable` is accepted and runs like a plain #run, synchronously.
#no_reset stall_steps := 0;
#run,stallable {
    stall_steps += 1;
}
#assert stall_steps == 1;
STALLED :: #run,stallable factorial(4);

// ctexec.4: a top-level #run that needs a #placeholder waits until a metaprogram defines it.
PROGRAM :: #string END
#import "Basic";
#placeholder Width;
Doubled :: #run Width * 2;
#run {
    assert(Width == 9, "top-level #run saw the placeholder's value");
}
main :: () { #assert Doubled == 18; }
END

#run {
    w := compiler_create_workspace("placeholder run");
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
                add_build_string("Width :: 9;", w);
                added = true;
            }
        }
        if msg.kind == .COMPLETE {
            failed = (cast(*Message_Complete) msg).error_code != .NONE;
            break;
        }
    }
    compiler_end_intercept(w);
    assert(added, "ctexec.4: the metaprogram defined the placeholder");
    assert(!failed, "ctexec.4: the waiting #run succeeded once the placeholder existed");
}

main :: () {
    // ctexec.1 / ctexec.2
    assert(F10 == 3628800, "ctexec.2: expression form");
    assert(NAMES[0] == "a" && NAMES[2] == "c", "ctexec.2: body with return type");
    assert(CHOSEN == u16, "ctexec.1: #run yields a Type");
    assert(initialized_global == 120, "ctexec.1: #run as a global initializer");
    local := #run factorial(6);
    assert(local == 720, "ctexec.2: #run inside a procedure body");

    // ctexec.3
    assert(run_order[0] == 1 && run_order[1] == 2 && run_order[2] == 3, "ctexec.3: declaration order");

    // ctexec.5
    assert(PHASE, "ctexec.5: #compile_time inside #run");
    assert(!#compile_time, "ctexec.5: #compile_time in main");

    // ctexec.11
    assert(SOURCE_READ, "ctexec.11: read_entire_file at compile time");
    #if OS != .WASM assert(CHILD_OUTPUT == "from child", "ctexec.11: run_command output at compile time");

    // ctexec.13
    assert(stall_steps == 1, "ctexec.13: #run,stallable ran once");
    assert(STALLED == 24, "ctexec.13: #run,stallable result");
}
