run_native :: (executable_path_without_extension: string, args: [] string) {
    result, output, errors, timed := Process.run_command(executable_path_without_extension, ..args);

    defer free(output);
    defer free(errors);
    if result.type == .EXITED && result.exit_code == 0 && !timed return;

    diagnostic := sprint("Autorun process failed: type %, exit %, signal %\n", cast(s64) result.type, result.exit_code, result.signal);

    defer free(diagnostic);
    context.logger(diagnostic, context.logger_data, Log_Info.{ common_flags = .ERROR });
}

run_build_result_of_workspace :: (w: Workspace, args: [] string = .[]) {
    options := C.get_build_options(w);
    if options.output_type != .EXECUTABLE {
        context.logger("Autorun skipped a non-executable workspace\n", context.logger_data, .{});
        return;
    }

    executable := sprint("%%", options.output_path, options.output_executable_name);

    defer free(executable);
    run_native(executable, args);
}

get_plugin :: () -> *Plugin {
    return *singleton;
}

init :: (_p: *Plugin, options: [] string) -> bool {
    shutdown(_p);
    _p.allocator = context.allocator;
    for option: options array_add(*_p.arguments, copy_string(option));
    return true;
}

message :: (_p: *Plugin, message: *C.Message) {
    if message.kind != .COMPLETE || message.workspace != _p.base.workspace return;
    completed := cast(*C.Message_Complete) message;
    if completed.error_code != .NONE return;
    run_build_result_of_workspace(message.workspace, _p.arguments);
}

shutdown :: (_p: *Plugin) {
    for argument: _p.arguments _p.allocator.proc(.FREE, 0, argument.count, argument.data, _p.allocator.data);

    array_free(_p.arguments);
    _p.arguments.count = 0;
    _p.arguments.allocated = 0;
    _p.arguments.data = null;
    _p.arguments.allocator = .{};
}

#scope_file
#import "Basic";
C :: #import "Compiler";
Process :: #import "Process";

Plugin :: struct {
    #as using base: C.Metaprogram_Plugin;
    arguments: [..] string;
    allocator: Allocator;
}

singleton: Plugin = .{ base = .{ init = initialize_plugin, message = receive_message, shutdown = shut_down_plugin } };

initialize_plugin :: (p: *C.Metaprogram_Plugin, options: [] string) -> bool {
    return init(cast(*Plugin) p, options);
}

receive_message :: (p: *C.Metaprogram_Plugin, event: *C.Message) {
    message(cast(*Plugin) p, event);
}

shut_down_plugin :: (p: *C.Metaprogram_Plugin) {
    shutdown(cast(*Plugin) p);
}
