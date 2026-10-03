HELP_STRING :: "Icon: -icon <filename> attaches an ICO file after the executable is written.\n";

get_plugin :: () -> *C.Metaprogram_Plugin {
    state := B.New(Plugin_State);
    if !state return null;

    state.api.init = init;
    state.api.message = message;
    state.api.shutdown = shutdown;
    state.api.log_help = log_help;

    return *state.api;
}

init :: (p: *C.Metaprogram_Plugin, options: [] string) -> bool {
    state := cast(*Plugin_State) p;
    state.filename = "";
    index: s64;
    while index < options.count {
        option := options[index];
        if option != "-icon" {
            B.log_error("Unknown Icon option '%'", option);
            return false;
        }

        if index + 1 >= options.count {
            B.log_error("Icon -icon requires a filename");
            return false;
        }

        state.filename = options[index + 1];
        index += 2;
    }

    return true;
}

shutdown :: (p: *C.Metaprogram_Plugin) {
    B.free(p);
}

log_help :: (p: *C.Metaprogram_Plugin) {
    B.log("%", HELP_STRING);
}

message :: (p: *C.Metaprogram_Plugin, message: *C.Message) {
    if message.kind != .PHASE return;

    phase := cast(*C.Message_Phase) message;
    if phase.phase != .POST_WRITE_EXECUTABLE return;

    state := cast(*Plugin_State) p;
    if state.filename.count == 0 {
        B.log("Icon: no -icon file was supplied");
        return;
    }

    options := C.get_build_options(message.workspace);
    filename := B.sprint("%/%", options.output_path, phase.executable_name);

    defer B.free(filename);

    #if OS == .WINDOWS && BUILD_OS == .WINDOWS {
        if options.os_target == .WINDOWS {
            if !Resources.set_icon_by_filename(filename, state.filename) B.log_error("Icon: failed to set ICO on '%'", filename);
            return;
        }
    }

    B.log("Icon: setting executable icons requires a Windows compiler host and target (%)", options.os_target);
}

#scope_file
B :: #import "Basic";
C :: #import "Compiler";
#if OS == .WINDOWS Resources :: #import "Windows_Resources";

Plugin_State :: struct {
    api: C.Metaprogram_Plugin;
    filename: string;
}
