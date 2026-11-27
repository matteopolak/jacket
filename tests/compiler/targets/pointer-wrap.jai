// Pointer arithmetic and integers cast to pointers wrap at the target's pointer width, and a
// procedure value read back from memory equals the constant it was set from.
Handler :: #type (signal: s32) #c_call;

Table :: struct {
    on_signal: Handler;
}

main :: () {
    top := cast(*u8) cast(s64) -16;
    step: s64 = 32;
    low := cast(*u8) 16;
    up := top + 32;
    stepped := top + step;
    down := low - 32;

    print("% % % %\n", cast(u64) top, cast(u64) up, cast(u64) stepped, cast(u64) down);

    t: Table;
    t.on_signal = cast(Handler) cast(s64) -1;
    print("% %\n", t.on_signal == cast(Handler) cast(s64) -1, cast(u64) cast(*void) t.on_signal);
}

#import "Basic";
