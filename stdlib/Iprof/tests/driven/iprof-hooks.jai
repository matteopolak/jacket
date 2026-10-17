#import "Basic";
#import "Compiler";
I :: #import "Iprof"(USE_GRAPHICS = false);

#run {
    for mode, index: Optimization_Type.[.DEBUG, .OPTIMIZED] {
        w := compiler_create_workspace("Original public profiling hooks");
        options := get_build_options(w);

        set_optimization(*options, mode);
        options.output_path = tprint("%../../../../target", #filepath);
        options.output_executable_name = tprint("profiler-codex-hook-child-o%", ifx index == 0 then 0 else 2);
        set_build_options(options, w);
        compiler_begin_intercept(w);
        I.add_hook(w, "runtime_frame", "()", "client_frame", "");
        I.add_hook(w, "runtime_editor", "(filename: string, line: s64)", "client_editor", "filename, line + 1");
        I.add_hook(w, "optional_hook", "()", "", "");
        add_build_file(tprint("%iprof-hook-input.jai", #filepath), w);
        completed := false;

        while true {
            event := compiler_wait_for_message();
            if !event break;

            if event.kind == .COMPLETE {
                assert((cast(*Message_Complete) event).error_code == .NONE);
                completed = true;
                break;
            }
        }

        compiler_end_intercept(w);
        assert(completed);
    }
}

main :: () {
    print("ok\n");
}
