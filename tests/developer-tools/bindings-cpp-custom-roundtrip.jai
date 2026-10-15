#import "Basic";
G :: #import "developer-tools-cpp-custom-generated";

main :: () {
    assert(G.custom_integer(a = 10, b = 3) == 103);
    assert(G.custom_floating(a = 1.25, b = 0.5) == 13.0);
    record: G.Custom_Record;

    G.make_custom_record(*record, value = 41);
    assert(record.number == 41);
    pointer := G.reference_custom_record(*record);

    assert(pointer == *record && record.number == 42);
    print("ok\n");
}
