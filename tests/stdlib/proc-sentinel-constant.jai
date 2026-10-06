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
    assert(cast(s64) t.on_signal == 1);
    assert(cast(s64) ERROR == -1);

    // The POSIX dispositions have the handler type on every platform.
    action: sigaction_t;
    action.sa_handler = SIG_IGN;
    assert(action.sa_handler == SIG_IGN && action.sa_handler != SIG_DFL);
    print("ok\n");
}
