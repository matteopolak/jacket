// The generated functions that return procedures, called against C: the second argument of a call
// reaches the native function and the returned procedure is the one the native side kept.
#import "Basic";
G :: #import "developer-tools-procedure-results-generated";

handle :: (display: *G.Display) -> s32 #c_call {
    return 33;
}

signal_seen :: (number: s32) #c_call {}

main :: () {
    display: *G.Display;

    // The initial value is what the native side stored before.
    previous := G.set_after_function(display, handle);

    assert(previous != null && previous(display) == 11);
    again := G.set_after_function(display, previous);

    assert(again != null && again(display) == 33);

    // `enable` is the second parameter, so zero returns no procedure and one returns the native one.
    assert(G.set_synchronize(display, 0) == null);
    synchronize := G.set_synchronize(display, 1);

    assert(synchronize != null && synchronize(display) == 1);
    assert(G.set_signal(1, signal_seen) == null);
    assert(G.set_signal(1, null) == signal_seen);

    first := G.set_error_handler(handle);

    assert(first != null && first(display) == 11);
    second := G.set_error_handler(null);

    assert(second != null && second(display) == 33);
    print("ok\n");
}
