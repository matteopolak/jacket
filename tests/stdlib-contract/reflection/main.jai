// The public boolean parser recognizes a leading keyword without checking a boundary.
#import "Basic";
Basic_Module :: #import "Basic";
Reflection :: #import "Reflection";
Unicode :: #import "Unicode";
Bit_Array :: #import "Bit_Array";

main :: () {
    assert(type_info(Basic_Module.String_Builder).name == "String_Builder");
    assert(type_info(Unicode.Unicode_Result).name == "Unicode_Result");
    assert(type_info(Bit_Array.Bit_Array).name == "Bit_Array");

    flag := false;
    rest := "truest";

    assert(Reflection.set_value_from_string(flag, *rest));
    assert(flag && rest == "st");

    rest = "TRUE";
    assert(!Reflection.set_value_from_string(flag, *rest));
    assert(rest == "TRUE");
    print("ok\n");
}
