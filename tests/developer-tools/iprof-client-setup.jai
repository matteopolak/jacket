#import "Basic";
#import "Compiler";
#import "File";
Default :: #import "Iprof";
Configured :: #import "Iprof"(MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);
C000 :: #import "Iprof"(MANUAL_MODE = false, USE_GRAPHICS = false, DO_HISTORY = false);
C001 :: #import "Iprof"(MANUAL_MODE = false, USE_GRAPHICS = false, DO_HISTORY = true);
C010 :: #import "Iprof"(MANUAL_MODE = false, USE_GRAPHICS = true, DO_HISTORY = false);
C101 :: #import "Iprof"(MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = true);
C110 :: #import "Iprof"(MANUAL_MODE = true, USE_GRAPHICS = true, DO_HISTORY = false);
C111 :: #import "Iprof"(MANUAL_MODE = true, USE_GRAPHICS = true, DO_HISTORY = true);

#run {
    input, input_ok := read_entire_file(tprint("%iprof-client-input.jai.inc", #filepath));

    assert(input_ok);
    defer free(input);

    for text, configuration: string.[C000.TO_INSERT, C001.TO_INSERT, C010.TO_INSERT, Default.TO_INSERT, Configured.TO_INSERT, C101.TO_INSERT, C110.TO_INSERT, C111.TO_INSERT] {
        expected_manual := (configuration & 4) != 0;
        expected_graphics := (configuration & 2) != 0;
        expected_history := (configuration & 1) != 0;
        source := sprint("%\n#scope_export\nOriginal_Expected_Manual :: %; Original_Expected_Graphics :: %; Original_Expected_History :: %;\n%", text, expected_manual, expected_graphics, expected_history, input);

        defer free(source);
        filename := tprint("%../../target/developer-tools-iprof-client-input-c%.jai", #filepath, configuration);

        assert(write_entire_file(filename, source));

        for mode, index: Optimization_Type.[.DEBUG, .OPTIMIZED] {
            workspace := compiler_create_workspace("Original profiling client setup");
            options := get_build_options(workspace);

            copy_commonly_propagated_fields(get_build_options(), *options);
            set_optimization(*options, mode);
            options.output_type = .EXECUTABLE;
            options.output_path = tprint("%../../target", #filepath);
            options.output_executable_name = tprint("developer-tools-iprof-client-c%-o%", configuration, ifx index == 0 then 0 else 2);
            set_build_options(options, workspace);
            compiler_begin_intercept(workspace);
            add_build_file(filename, workspace);
            complete := false;

            while true {
                message := compiler_wait_for_message();
                if !message break;

                if message.kind == .COMPLETE {
                    assert((cast(*Message_Complete) message).error_code == .NONE);
                    complete = true;
                    break;
                }
            }

            compiler_end_intercept(workspace);
            assert(complete);
        }
    }
}

main :: () {
    print("ok\n");
}
