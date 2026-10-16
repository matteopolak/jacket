#import "Basic";
G :: #import "developer-tools-cpp-custom-bridge";
C :: #import "developer-tools-cpp-callback-generated";

main :: () {
    assert(G.custom_integer(a = 10, b = 3) == 103);
    assert(G.custom_floating(a = 1.25, b = 0.5) == 13.0);
    record: G.Custom_Record;

    G.make_custom_record(*record, value = 41);
    assert(record.number == 41);
    pointer := G.reference_custom_record(*record);

    assert(pointer == *record && record.number == 42);
    // The ordinary generated wrapper takes a value, addresses it for the real
    // foreign pointer parameter, and returns the independently compiled result.
    assert(C.original_callback_value(10) == 103);
    print("ok\n");
}
