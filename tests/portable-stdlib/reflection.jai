#import "Basic";
R :: #import "Reflection";
P :: #import "Preload";

Kind :: enum u8 {
    RED :: 3;
    GREEN :: 7;
}

Sample :: struct {
    first: s32;
    second: string;
}

#assert P.Allocator == Allocator;

main :: () {
    assert(type_info(String_Builder).name == "String_Builder");
    flag := false;
    text := "truest";

    assert(R.set_value_from_string(flag, *text) && flag && text == "st");
    text = "TRUE";
    assert(!R.set_value_from_string(flag, *text) && text == "TRUE");
    v: float64;

    text = "1.25e2rest";
    assert(R.set_value_from_string(v, *text) && v == 125 && text == "rest");
    value, ok := R.enum_name_to_value(Kind, "GREEN");

    assert(ok && value == .GREEN && R.enum_value_to_name(value) == "GREEN");
    text = "RED tail";
    assert(R.set_value_from_string(value, *text) && value == .RED && text == " tail");
    quoted: string;

    text = "\"copper\"tail";
    assert(R.set_value_from_string(quoted, *text) && quoted == "copper" && text == "tail");
    free(quoted);
    member := R.get_struct_field_info(type_info(Sample), "second");

    assert(member != null && member.offset_in_bytes == 8);
    array: [3] u32;
    count, data := R.get_array_count_and_data(*array, type_info(type_of(array)));

    assert(count == 3 && data == array.data);
    print("ok\n");
}
