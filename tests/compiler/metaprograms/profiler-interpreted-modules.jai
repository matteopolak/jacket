#import "Basic";
#import "Compiler";
I :: #import "Iprof"(USE_GRAPHICS = false);

#run {
    p := I.get_plugin();
    p.workspace = compiler_create_workspace("Original interpreted edited profiler body");
    options := get_build_options(p.workspace);
    options.output_type = .NO_OUTPUT;
    options.import_path = .[tprint("%../profiler-modules", #filepath)];
    set_build_options(options, p.workspace);
    destination := tprint("%../../../target/profiler-codex-interpreted-edited.csv", #filepath);

    assert(I.init(p, .["-min_size", "0", "-modules", "-csv", destination]));
    flags: Intercept_Flags;

    I.before_intercept(p, *flags);
    compiler_begin_intercept(p.workspace, flags);
    I.add_source(p);
    add_build_file(tprint("%../../../stdlib/Iprof/tests/driven/iprof-modules-input.jai", #filepath), p.workspace);
    run_added, completed: bool;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        I.message(p, event);

        if event.kind == .PHASE && (cast(*Message_Phase) event).phase == .TYPECHECKED_ALL_WE_CAN && I.injected_final_profiling_code && !run_added {
            add_build_string("#run main();", p.workspace);
            run_added = true;
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            completed = true;
            break;
        }
    }

    compiler_end_intercept(p.workspace);
    I.finish(p);
    I.shutdown(p);
    assert(completed && run_added);
}

main :: () {
    print("ok\n");
}
