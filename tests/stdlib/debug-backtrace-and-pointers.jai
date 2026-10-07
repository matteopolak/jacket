// Debug's stack capture, frame names, pointer checks and crash handler switches. Debug does not
// build for WASM (no native stack or memory map): tests/stdlib-runtime-skips.txt lists those runs.
#import "Basic";

Debug :: #import "Debug"(USE_GRAPHICS = false);

capture_here :: no_inline () -> int {
    frames: [32] *void;
    return Debug.backtrace(frames, frames.count, 0);
}

main :: () {
    frames: [16] *void;
    count := Debug.backtrace(frames);
    assert(count > 0 && count <= frames.count, "a stack has frames");
    for 0..count - 1 assert(frames[it] != null, "captured addresses are code");

    // A limit below the array's size is honoured, and nothing is captured into no room.
    assert(Debug.backtrace(frames, 1, 0) == 1);
    assert(Debug.backtrace(frames, 0, 0) == 0);
    empty: [] *void;
    assert(Debug.backtrace(empty) == 0);
    assert(capture_here() > 0);

    names := Debug.backtrace_symbols(frames, count);
    assert(names.count == count, "one name per frame");
    for names assert(it.count > 0, "frames have names");
    Debug.free_backtrace(names);
    // More names asked for than frames given: only the frames are named.
    names = Debug.translate_symbols(array_view(frames, 0, count), count + 5);
    assert(names.count == count);
    Debug.free_backtrace(names);

    lines := Debug.backtrace();
    assert(lines.count > 0);
    Debug.free_backtrace(lines);

    local := 7;
    assert(Debug.is_valid_pointer(*local), "a stack variable is mapped");
    heap := alloc(64);
    assert(Debug.is_valid_pointer(heap), "an allocation is mapped");
    free(heap);
    assert(!Debug.is_valid_pointer(null), "null is not");
    assert(!Debug.is_valid_pointer(cast(*void) 16), "the first page is never mapped");

    // The crash handler installs and uninstalls cleanly, any number of times.
    Debug.enable_signal_handler();
    Debug.enable_signal_handler(interactive = true);
    Debug.set_report_mode(false);
    Debug.disable_signal_handler();
    Debug.disable_signal_handler();
    Debug.enable_signal_handler();
    Debug.disable_signal_handler();

    Debug.set_minidump_state(.Full);
    assert(Debug.get_written_minidump_filename() == "");
    assert(!Debug.break_once(), "no debugger is attached to the tests");
    assert(!Debug.is_debugger_present());
    print("ok\n");
}
