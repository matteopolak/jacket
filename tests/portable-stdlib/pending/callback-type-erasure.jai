#import "Basic";

Callback :: #type (data: *void);

make_callback :: ($T: Type) -> Callback {
    callback :: (data: *void) {
        value := cast(*T) data;
        value.* += 1;
    }

    return callback;
}

main :: () {
    integer: s64 = 9;
    real: float64 = 0.5;
    first := make_callback(s64);
    second := make_callback(float64);

    first(*integer);
    second(*real);
    assert(integer == 10 && real == 1.5);
    print("ok\n");
}
