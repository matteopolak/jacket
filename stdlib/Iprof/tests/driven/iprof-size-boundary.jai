#import "Basic";
#import "Compiler";
I :: #import "Iprof"(USE_GRAPHICS = false);

#run {
    control := compiler_create_workspace("Original compiler expression count");
    options := get_build_options(control);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, control);
    compiler_begin_intercept(control);
    add_build_file(tprint("%iprof-plugin-input.jai", #filepath), control);
    expression_count: s64 = -1;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            if tc.expression.header && tc.expression.header.name == "leaf" expression_count = tc.subexpressions.count;
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(control);
    assert(expression_count > 0);

    for mode, index: Optimization_Type.[.DEBUG, .OPTIMIZED] {
        for offset: 0..1 {
            p := I.get_plugin();
            p.workspace = compiler_create_workspace("Original minimum-expression boundary");
            options = get_build_options(p.workspace);
            set_optimization(*options, mode);
            options.output_path = tprint("%../../../../target", #filepath);
            options.output_executable_name = tprint("profiler-codex-size-o%-d%", ifx index == 0 then 0 else 2, offset);
            set_build_options(options, p.workspace);
            destination := tprint("%../../../../target/profiler-codex-size-o%-d%.csv", #filepath, ifx index == 0 then 0 else 2, offset);

            assert(I.init(p, .["-min_size", tprint("%", expression_count + offset), "-csv", destination]));
            flags: Intercept_Flags;

            I.before_intercept(p, *flags);
            compiler_begin_intercept(p.workspace, flags);
            I.add_source(p);
            add_build_file(tprint("%iprof-plugin-input.jai", #filepath), p.workspace);
            completed := false;
            frame_hook, editor_hook: bool;

            while true {
                event := compiler_wait_for_message();
                if !event break;

                I.message(p, event);

                if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_headers {
                    if tc.expression.name == "iprof_per_frame_hook" frame_hook = true;
                    if tc.expression.name == "iprof_editor_hook" editor_hook = true;
                }

                if event.kind == .COMPLETE {
                    assert((cast(*Message_Complete) event).error_code == .NONE);
                    completed = true;
                    break;
                }
            }

            compiler_end_intercept(p.workspace);
            assert(completed && frame_hook && editor_hook && I.instrumented_entry_point && I.injected_final_profiling_code);
            I.finish(p);
            metadata_count := I.zone_infos.count;

            assert(metadata_count == (ifx offset == 0 then 2 else 1));
            I.shutdown(p);
            assert(I.zone_infos.count == metadata_count);
        }
    }
}

main :: () {
    print("ok\n");
}
