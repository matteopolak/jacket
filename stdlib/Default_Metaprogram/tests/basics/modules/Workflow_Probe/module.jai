#module_parameters(TAG := "A");

get_plugin :: () -> *C.Metaprogram_Plugin {
    p := New(Probe);
    p.base.init = initialize;
    p.base.before_intercept = prepare;
    p.base.add_source = add_source;
    p.base.message = receive;
    p.base.finish = finish;
    p.base.shutdown = shutdown;
    p.base.log_help = help;

    return *p.base;
}

#scope_module
using #import "Basic";
C :: #import "Compiler";

Probe :: struct {
    #as using base: C.Metaprogram_Plugin;
    check_options: bool;
    complete: bool;
    messages: s64;
}

initialize :: (base: *C.Metaprogram_Plugin, options: [] string) -> bool {
    p := cast(*Probe) base;

    print("init % %\n", TAG, options.count);

    for option: options {
        if option == "-fail" return false;
        if option == "-options" p.check_options = true;
        else if option != "value" return false;
    }

    return true;
}

prepare :: (base: *C.Metaprogram_Plugin, flags: *C.Intercept_Flags) {
    p := cast(*Probe) base;
    options := C.get_build_options(p.workspace);

    if p.check_options {
        assert(!options.stack_trace);
        assert(options.dead_code_elimination == .NONE);
        assert(options.llvm_options.disable_inlining);
        assert(!options.llvm_options.enable_split_modules);
        assert(!options.use_ansi_color);
        assert(options.use_visual_studio_message_format);
        assert(!options.use_natvis_compatible_types);
        assert(options.debug_for_expansions);
        assert(options.backtrace_on_crash == .OFF);
        assert(options.text_output_flags == 0);
        assert(options.context_size_max == 8192);
        assert(options.compile_time_command_line.count == 2);
        assert(options.compile_time_command_line[0] == "first");
        assert(options.compile_time_command_line[1] == "-literal");
    }

    print("prepare %\n", TAG);
}

add_source :: (base: *C.Metaprogram_Plugin) {
    print("source %\n", TAG);
}

receive :: (base: *C.Metaprogram_Plugin, message: *C.Message) {
    p := cast(*Probe) base;
    p.messages += 1;
    assert(!p.complete);

    if message.kind == .COMPLETE {
        p.complete = true;
        print("complete %\n", TAG);
    }
}

finish :: (base: *C.Metaprogram_Plugin) {
    p := cast(*Probe) base;
    assert(p.complete);
    assert(p.messages > 1);
    print("finish %\n", TAG);
}

shutdown :: (base: *C.Metaprogram_Plugin) {
    print("shutdown %\n", TAG);
    free(base);
}

help :: (base: *C.Metaprogram_Plugin) {
    print("help %\n", TAG);
}
