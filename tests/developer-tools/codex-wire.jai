#import "Basic";
C :: #import "Codex";

short_roundtrip :: ($T: Type) {
    builder: String_Builder;

    defer free_buffers(*builder);
    C.put_short_string(*builder, "abc", T, true);
    bytes := builder_to_string(*builder);

    defer free(bytes);
    input := bytes;
    error := true;
    owned := C.get_short_string(*input, T, *error, true, true);

    defer free(owned);
    assert(error && owned == "abc" && owned.data != bytes.data + size_of(T));
    assert(owned.data[owned.count] == 0 && input.count == 0);
}

main :: () {
    short_roundtrip(s8);
    short_roundtrip(u8);
    short_roundtrip(s16);
    short_roundtrip(u16);
    short_roundtrip(s32);
    short_roundtrip(u32);
    builder: String_Builder;

    defer free_buffers(*builder);
    C.put(*builder, cast(u32) 0x78563412);
    C.put_short_string(*builder, "héllo", u16, true);
    C.put(*builder, "owned");
    data := builder_to_string(*builder);

    defer free(data);
    assert(data[0] == 0x12 && data[1] == 0x34 && data[2] == 0x56 && data[3] == 0x78);
    input := data;
    number: u32;

    C.get(*input, *number);
    assert(number == 0x78563412);
    error := false;
    borrowed := C.get_short_string(*input, u16, *error, false, true);

    assert(!error && borrowed == "héllo" && borrowed.data == data.data + 6);
    result: string;

    C.get(*input, *result, *error);
    defer free(result);
    assert(!error && result == "owned" && input.count == 0);
    short := string.{ 1, data.data };
    invalid := C.get_short_string(*short, u16, *error);

    assert(error && invalid.count == 0);
    header: C.Header_Record;
    header.magic = 0x05040003;
    header.version = 2;
    bytes: [] u8 = .{ size_of(C.Header_Record), cast(*u8) *header };
    ok, help, message := C.make_help(bytes);

    assert(!ok && message.count > 0); // Missing four required trailing NULs.
    header.version = 3;
    ok, help, message = C.make_help(bytes);
    assert(!ok && message.count > 0);
    free_buffers(*builder);
    C.put(*builder, cast(s64) -1);
    negative := builder_to_string(*builder);

    defer free(negative);
    input = negative;
    error = false;
    prior := result;

    C.get(*input, *result, *error);
    assert(error && input.count == 0 && result.data == prior.data && result.count == prior.count);
    print("ok\n");
}
