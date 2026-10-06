// rules: ws.6 ws.7 ws.9 ws.10 ws.11 ws.12 ws.13 ws.15
// Claims of docs/metaprogramming/workspaces.md: event order of an intercepted workspace,
// failure codes, nested metaprograms and sources a #run adds to its own workspace.
#import "Basic";
#import "Compiler";
#import "File";

GOOD :: #string END
#import "Basic";
main :: () { print("unused\n"); }
END

// The target is itself a metaprogram: its #run drives a workspace of its own.
NESTED :: #string END
#import "Basic";
#import "Compiler";
#run {
    inner := compiler_create_workspace("innermost");
    options := get_build_options(inner);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, inner);
    compiler_begin_intercept(inner);
    add_build_string("main :: () {}", inner);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    compiler_end_intercept(inner);
}
main :: () {}
END

Event :: struct {
    kind: type_of(Message.kind);
    phase: s64 = -1;
}

// Runs `source` in a new intercepted workspace, returning its events and completion code.
collect :: (name: string, source: string, events: *[..] Event, executable := "") -> s64 {
    w := compiler_create_workspace(name);
    options := get_build_options(w);
    if executable {
        options.output_type = .EXECUTABLE;
        options.output_executable_name = executable;
    } else {
        options.output_type = .NO_OUTPUT;
    }
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(source, w);
    code := -1;
    while true {
        message := compiler_wait_for_message();
        event := Event.{ kind = message.kind };
        if message.kind == .PHASE event.phase = cast(s64) (cast(*Message_Phase) message).phase;
        array_add(events, event);
        if message.kind == .COMPLETE {
            code = cast(s64) (cast(*Message_Complete) message).error_code;
            break;
        }
    }
    compiler_end_intercept(w);
    return code;
}

check_order :: () {
    events: [..] Event;
    code := collect("order", GOOD, *events);

    // ws.13: a workspace that compiles completes with error code 0 (failure: corpus negative rule-ws-13).
    assert(code == 0, "ws.13: good workspace completes with NONE");

    parsed := -1;
    first_typechecked_all := -1;
    first_file := -1;
    last_file_before_parsed := -1;
    for events {
        if it.kind == .FILE {
            if first_file < 0 first_file = it_index;
            if parsed < 0 last_file_before_parsed = it_index;
        }
        if it.kind == .PHASE && it.phase == 0 && parsed < 0 parsed = it_index;
        if it.kind == .PHASE && it.phase == 1 && first_typechecked_all < 0 first_typechecked_all = it_index;
    }
    // ws.7: FILE events, then PHASE ALL_SOURCE_CODE_PARSED, then PHASE TYPECHECKED_ALL_WE_CAN.
    assert(first_file >= 0 && parsed > last_file_before_parsed, "ws.7: files come before ALL_SOURCE_CODE_PARSED");
    assert(first_typechecked_all > parsed, "ws.7: TYPECHECKED_ALL_WE_CAN follows ALL_SOURCE_CODE_PARSED");

    // ws.11: an IMPORT precedes the first FILE.
    first_import := -1;
    for events if it.kind == .IMPORT {
        first_import = it_index;
        break;
    }
    assert(first_import >= 0 && first_import < first_file, "ws.11: IMPORT before FILE");

    // ws.12: a TYPECHECKED message comes right before the first TYPECHECKED_ALL_WE_CAN.
    assert(events[first_typechecked_all - 1].kind == .TYPECHECKED, "ws.12: TYPECHECKED just before the phase");

    // COMPLETE is the last event.
    assert(events[events.count - 1].kind == .COMPLETE, "COMPLETE ends the stream");
}

check_nested :: () {
    events: [..] Event;
    // ws.9: a target workspace may run a metaprogram of its own.
    code := collect("nested", NESTED, *events);
    assert(code == 0, "ws.9: nested metaprogram completes");
}

check_no_output :: () {
    events: [..] Event;
    // ws.15: `jaic run` only checks workspaces: an EXECUTABLE target writes nothing.
    name := "rules_ws_not_written";
    code := collect("no output", GOOD, *events, name);
    assert(code == 0, "ws.15: workspace with EXECUTABLE output checks");
    assert(!file_exists(name), "ws.15: no executable is written under jaic run");
}

#run check_order();
#run check_nested();
#run check_no_output();

// ws.6: warnings are reported without stopping the compile.
#run compiler_report("rules-workspaces: this warning is expected", mode = .WARNING);

// ws.10: a #run that adds source to its own workspace defines a placeholder right away.
#placeholder FILLED_IN;
#run add_build_string("FILLED_IN :: 17;", -1);
#assert FILLED_IN == 17;

main :: () {
    assert(FILLED_IN == 17, "ws.10: placeholder filled");
    print("ok\n");
}
