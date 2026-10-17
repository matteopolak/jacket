// Native-ready public Windows fixture; the SDK C oracle checks dump contents.
#import "Basic";
D :: #import "Debug";
N :: #import "Debug"(USE_GRAPHICS = false);
W :: #import "Windows";

main :: () {
    #assert OS == .WINDOWS;
    #assert !#exists(D.init_crash_handler_thread);
    #assert !#exists(N.interactive_assert);
    assert(*D.crash_handler == *N.crash_handler);
    assert(*D.ignored_asserts == *N.ignored_asserts);
    assert(D.get_written_minidump_filename().count == 0);
    N.set_minidump_state(.None);
    mini := D.write_minidump(cast(*W.EXCEPTION_POINTERS) null, false);

    assert(mini.count > 0);
    defer free(mini.data);
    assert(N.get_written_minidump_filename().count == 0);
    full := N.write_minidump(cast(*void) null, true);

    assert(full.count > 0 && full != mini);
    defer free(full.data);
    assert(D.get_written_minidump_filename().count == 0);
    if D.is_debugger_present() assert(D.attach_to_debugger());

    print("%\n%\n", mini, full);
}
