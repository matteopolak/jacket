#import "Basic";

Inner :: struct {
    amount: int = 31;
}

Outer :: struct {
    prefix: u8;
    using inner: Inner;
    suffix: int;
}

Choice :: enum s32 {
    NEGATIVE :: -10;
    HIGH :: 73;
    ZERO :: 0;
}

Empty :: enum u8 {}

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_sequence_to_builder(*builder, "A", 15, true, "Z");
    text := builder_to_string(*builder);

    defer free(text);
    assert(text == "\"A\"15true\"Z\"");

    input := "abcde";

    destination: [4] u8 = .[99, 99, 99, 99];

    assert(consume_u8_and_length(*input, destination.data, 2) && input == "cde");
    assert(destination[0] == 97 && destination[1] == 98 && destination[2] == 99);
    before := input;

    assert(!consume_u8_and_length(*input, destination.data, -1));
    assert(input.data == before.data && input.count == before.count);
    assert(!consume_u8_and_length(*input, destination.data, 4));
    assert(destination[0] == 97 && destination[1] == 98 && destination[2] == 99);
    assert(consume_u8_and_length(*input, null, 0) && input == "cde");

    object: Outer;
    member, offset := get_field(cast(*Type_Info_Struct) type_info(Outer), "amount");

    assert(member != null && member.name == "amount");
    assert((cast(*int) ((cast(*u8) *object) + offset)).* == 31);
    direct, direct_offset := get_field(cast(*Type_Info_Struct) type_info(Outer), "suffix");

    assert(direct != null && direct_offset == direct.offset_in_bytes);
    absent, absent_offset := get_field(cast(*Type_Info_Struct) type_info(Outer), "absent");

    assert(absent == null && absent_offset == 0);
    low, high := enum_range(Choice);

    assert(low == -10 && high == 73 && enum_highest_value(Choice) == 73);
    info_low, info_high := enum_range_given_info(cast(*Type_Info_Enum) type_info(Choice));

    assert(info_low == low && info_high == high);
    empty_low, empty_high := enum_range(Empty);

    assert(empty_low == 0 && empty_high == 0 && enum_highest_value(Empty) == 0);

    values: [] Inner;

    array_resize(*values, 3);
    defer array_free(values);
    assert(values.count == 3 && values[0].amount == 31 && values[2].amount == 31);
    values[0].amount = 5;
    array_resize(*values, 1);
    assert(values.count == 1 && values[0].amount == 5);
    array_resize(*values, 4);
    assert(values.count == 4 && values[0].amount == 5 && values[3].amount == 31);
    print("ok\n");
}
