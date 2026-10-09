// The libraries a metaprogram's custom link command receives arrive in the lists named for them:
// system libraries in `system_libraries`, objects in `compiler_generated_object_files`. A
// program with several of each used to see them misfiled (the list was picked by the position
// inside the list being read, not by which list it was), and a build with five or more
// libraries stopped on an index error. forbear's build.jai links its playground this way.
// rules: compiler.17
#import "Basic";
#import "Compiler";

#if OS == .MACOS || OS == .LINUX {
    SOURCE :: #string END
    #import "Basic";
    libm :: #library,system "m";
    libz :: #library,system "z";
    libpthread :: #library,system "pthread";
    cbrt :: (x: float64) -> float64 #foreign libm;
    zlibVersion :: () -> *u8 #foreign libz;
    sched_yield :: () -> s32 #foreign libpthread;
    start :: () {
        print("%\n", cbrt(27));
        zlibVersion();
        sched_yield();
    }
    END

    link_lists :: () -> system: [] string, reached: bool {
        set_build_options_dc(.{ do_output = false });
        w := compiler_create_workspace("lists");
        options := get_build_options(w);
        options.output_executable_name = "lists";
        options.use_custom_link_command = true;
        options.entry_point_name = "start";
        set_build_options(options, w);
        compiler_begin_intercept(w);
        add_build_string(SOURCE, w);
        system: [..] string;
        reached := false;
        while true {
            message := compiler_wait_for_message();
            if message.kind == .PHASE {
                phase := cast(*Message_Phase) message;
                if phase.phase == .READY_FOR_CUSTOM_LINK_COMMAND {
                    for phase.system_libraries array_add(*system, copy_string(it));
                    reached = true;
                    compiler_custom_link_command_is_complete(w, 0);
                }
            }
            if message.kind == .COMPLETE break;
        }
        compiler_end_intercept(w);
        return system, reached;
    }

    contains :: (list: [] string, text: string) -> bool {
        for list if it == text return true;
        return false;
    }

    #run {
        // The sweep builds workspaces without output (-no_workspace_output), which never link.
        system, reached := link_lists();
        if !reached return;
        assert(contains(system, "-lm"), "system libraries lack -lm: %", system);
        assert(contains(system, "-lz"), "system libraries lack -lz: %", system);
        assert(contains(system, "-lpthread"), "system libraries lack -lpthread: %", system);
    }
}

main :: () {
    print("ok\n");
}
