My_Plugin :: struct {
    #as using base: C.Metaprogram_Plugin;
    some_state := 0;
}

HELP_STRING :: #string END_HELP
Example_Plugin options: repeat -option VALUE to report a value at initialization.
END_HELP

get_plugin :: () -> *C.Metaprogram_Plugin {
    p := New(My_Plugin);
    p.init = init;
    p.before_intercept = before_intercept;
    p.add_source = add_source;
    p.message = message;
    p.finish = finish;
    p.shutdown = shutdown;
    p.log_help = log_help;

    return *p.base;
}

init :: (p: *C.Metaprogram_Plugin, options: [] string) -> bool {
    example := cast(*My_Plugin) p;
    example.some_state = 33;
    index: s64;
    while index < options.count {
        if options[index] != "-option" {
            print("Example_Plugin: unrecognized option '%'.\n", options[index]);
            return false;
        }

        if index + 1 == options.count {
            print("Example_Plugin: -option requires a value.\n");
            return false;
        }

        print("Example_Plugin option: %\n", options[index + 1]);
        index += 2;
    }

    return true;
}

before_intercept :: (p: *C.Metaprogram_Plugin, flags: *C.Intercept_Flags) {
    print("Example_Plugin: starting compiler interception.\n");
}

add_source :: (p: *C.Metaprogram_Plugin) {
    print("Example_Plugin: target sources are ready.\n");
}

message :: (p: *C.Metaprogram_Plugin, message: *C.Message) {
    example := cast(*My_Plugin) p;
    example.some_state += 1;

    if message.kind == {
        case .PHASE;
            phase := cast(*C.Message_Phase) message;

            print("Example_Plugin phase: %\n", phase.phase);
            if phase.phase == .PRE_WRITE_EXECUTABLE print("Example_Plugin executable: %\n", phase.executable_name);
        case .TYPECHECKED;
            typed := cast(*C.Message_Typechecked) message;

            print("Example_Plugin typechecked: % declarations, % headers, % bodies, % structs, % other, % total.\n",
                typed.declarations.count, typed.procedure_headers.count, typed.procedure_bodies.count,
                typed.structs.count, typed.others.count, typed.all.count);
        case .COMPLETE;
            complete := cast(*C.Message_Complete) message;

            print("Example_Plugin complete: %\n", complete.error_code);
    }
}

finish :: (p: *C.Metaprogram_Plugin) {
    example := cast(*My_Plugin) p;
    print("Example_Plugin: observed % messages.\n", example.some_state - 33);
}

shutdown :: (p: *C.Metaprogram_Plugin) {
    print("Example_Plugin: releasing plugin.\n");
    free(p);
}

log_help :: (p: *C.Metaprogram_Plugin) {
    write_string(HELP_STRING);
}

#scope_file
using #import "Basic";
C :: #import "Compiler";
