#import "Basic";

Named :: struct {
    value: int;
}

Choice :: enum {
    FIRST;
    SECOND;
}

main :: () {
    assert(sprint("% % % %", int, float32, bool, string) == "s64 float32 bool string");
    assert(type_to_string(type_info(*Named)) == "*Named");
    assert(type_to_string(type_info([3] u16)) == "[3] u16");
    assert(type_to_string(type_info([] Choice)) == "[] Choice");
    assert(type_to_string(type_info([..] s8)) == "[..] s8");
    assert(type_to_string(type_info((s64, bool) -> string)) == "(s64, bool) -> string");
    print("ok\n");
}
