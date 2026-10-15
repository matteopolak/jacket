// SDK integration evidence: choose the actual private native domain and drive
// its configured C error thunk. Public callback tests keep public operations.
#import "Basic";
RPM :: #import "rpmalloc"(USE_RAW = true);
Settings :: #import "internal/bindings/rpmalloc-build";
Native :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = true, RAW = true, PUBLIC_HEAPS = Settings.FIRST_CLASS_HEAPS);
Frames :: #import "internal/bindings/callback-context";

#add_context error_callback_marker: s64;
errors: s64;
expected_marker: s64;

error :: (message: string) {
    assert(message == "independent native error probe");
    assert(context.error_callback_marker == expected_marker);
    context.error_callback_marker = 72;
    errors += 1;
}

main :: () {
    input: RPM.rpmalloc_config_t;
    input.error_callback = error;
    assert(RPM.initialize_config(*input) == 0);
    actual := Native.rpmalloc_config();

    assert(actual.error_callback != null);
    message := "independent native error probe\0";

    context.error_callback_marker = 61;
    expected_marker = 61;
    previous := Frames.enter(cast(*void) *context);

    actual.error_callback(message.data);
    Frames.leave(previous);
    assert(errors == 1 && context.error_callback_marker == 72);
    context.error_callback_marker = 91;
    expected_marker = 0;
    actual.error_callback(message.data);
    assert(errors == 2 && context.error_callback_marker == 91);
    RPM.finalize();
}
