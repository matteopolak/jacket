#import "Basic";

capture :: () -> [] Stack_Trace_Node {
    return pack_stack_trace();
}

quiet_capture :: () -> [] Stack_Trace_Node #no_debug {
    return pack_stack_trace();
}

check :: () {
    trace := capture();

    defer free(trace.data);
    assert(trace.count >= 3);
    assert(trace[0].info.name == "pack_stack_trace");
    assert(trace[1].info.name == "capture");
    assert(trace[2].info.name == "check");
    assert(trace[0].next == trace.data + 1);
    assert(trace[trace.count - 1].next == null);

    explicit := pack_stack_trace(*trace[1]);

    defer free(explicit.data);
    assert(explicit.count == trace.count - 1);
    assert(explicit[0].info.name == "capture");

    quiet := quiet_capture();

    defer free(quiet.data);
    assert(quiet.count >= 2);
    assert(quiet[0].info.name == "pack_stack_trace");
    assert(quiet[1].info.name == "check");
}

#run check();

main :: () {
    check();
}
