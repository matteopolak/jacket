B :: #import "Basic";
Quiet :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);

calls: int;

capture :: (message: string, data: *void, info: Log_Info) {
    calls += 1;
}

main :: () {
    push_context {
        context.logger = capture;
        B.vis_init();
        B.maybe_add_string("disabled");
        B.vis_update();
        B.vis_update_if_it_has_been_long_enough();
        B.vis_fini();
        Quiet.vis_init();
        Quiet.maybe_add_string("quiet");
        Quiet.vis_update();
        Quiet.vis_update_if_it_has_been_long_enough();
        Quiet.memory_visualizer_per_frame_update();
        Quiet.vis_fini();
    }

    B.assert(calls == 0);
    B.print("ok\n");
}
