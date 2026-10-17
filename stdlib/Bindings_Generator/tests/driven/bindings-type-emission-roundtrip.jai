#import "Basic";
T :: #import "developer-tools-type-emission-generated";

callback :: (value: float64) -> s32 #c_call {
    return cast(s32) (value * 10);
}

main :: () {
    assert(type_info(T.Printed_Callback) == type_info(T.Printed_Function));
    assert(T.default_value() == 42 && T.default_value(9) == 9);
    assert(T.default_bool() == 1 && T.default_bool(false) == 0);
    values := s32.[2, 3, 4, 5];

    assert(T.printed_call(callback, 1.25) == 12);
    assert(T.printed_array(values.data) == 11);
    assert(values[2] == 11);
    print("ok\n");
}
