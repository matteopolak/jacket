#import "Basic";
I :: #import "Icon";
C :: #import "Compiler";

Capture :: struct {
    count: int;
    errors: int;
}

capture :: (text: string, data: *void, info: Log_Info) {
    assert(text.count > 0);
    counts := cast(*Capture) data;
    counts.count += 1;
    if (info.common_flags & .ERROR) != 0 counts.errors += 1;
}

#run verify_phase();

verify_phase :: () {
    plugin := I.get_plugin();

    assert(plugin != null);
    defer I.shutdown(plugin);
    options: [2] string = .["-icon", "borrowed.ico"];

    assert(I.init(plugin, options));
    counts: Capture;

    push_context {
        context.logger = capture;
        context.logger_data = *counts;
        phase := C.Message_Phase.{};
        phase.kind = .PHASE;
        phase.workspace = -1;
        phase.executable_name = "original-output";
        phase.phase = .PRE_WRITE_EXECUTABLE;
        I.message(plugin, cast(*C.Message) *phase);
        assert(counts.count == 0);
        phase.phase = .POST_WRITE_EXECUTABLE;
        I.message(plugin, cast(*C.Message) *phase);

        #if OS == .WINDOWS && BUILD_OS == .WINDOWS {
            // The ICO is deliberately absent: File and Icon both report failure.
            assert(counts.count == 2 && counts.errors == 2);
        } else assert(counts.count == 1 && counts.errors == 0);
    }
}

main :: () {
    print("ok\n");
}
