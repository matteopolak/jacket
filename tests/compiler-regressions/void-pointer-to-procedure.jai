// A `*void` holding an address converts to a procedure type without a cast, as when a window
// procedure is looked up from a table of untyped pointers (focus's Window_Creation).
#import "Basic";

Handlers :: struct {
    on_event: (a: s64, b: u32) -> s64 #c_call;
}

handle :: (a: s64, b: u32) -> s64 #c_call {
    return a + b;
}

main :: () {
    address: *void = xx handle;
    handlers: Handlers;

    handlers.on_event = address;

    direct: (a: s64, b: u32) -> s64 #c_call = address;

    push_context {
        assert(handlers.on_event(40, 2) == 42);
        assert(direct(1, 2) == 3);
    }

    print("ok\n");
}
