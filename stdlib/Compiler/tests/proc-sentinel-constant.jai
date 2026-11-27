// test: only(.LINUX,.MACOS)
// Small integers cast to a procedure type (`SIG_IGN`, `SIG_ERR`) are constants that can be
// stored and compared; they never point at code.
#import "Basic";
#import "POSIX";

Handler :: #type (signal: s32) -> void #c_call;
IGNORE :: cast(Handler) 1;
ERROR :: cast(Handler) -1;

Table :: struct {
    on_signal: Handler = IGNORE;
}

main :: () {
    t: Table;

    assert(t.on_signal == IGNORE);
    assert(cast(s64) cast(*void) t.on_signal == 1);
    // All ones at the pointer's width; a 4-byte pointer zero-extends to `s64`.
    assert(cast(s64) cast(*void) ERROR == ifx size_of(*void) == 8 then -1 else 0xFFFF_FFFF);

    // The reviewed macOS dispositions are raw pointers; Linux types handlers.
    action: sigaction_t;
    action.sa_handler = cast(Handler) SIG_IGN;
    assert(action.sa_handler == cast(Handler) SIG_IGN && action.sa_handler != cast(Handler) SIG_DFL);
    print("ok\n");
}
