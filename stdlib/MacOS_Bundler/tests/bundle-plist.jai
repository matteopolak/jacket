#import "Basic";
M :: #import "MacOS_Bundler";

main :: () {
    #assert !#exists(M.checked_append);
    #assert !#exists(M.xml_text_valid);
    #assert !#exists(M.execute);
    builder: String_Builder;

    defer free_buffers(*builder);
    M.plist_start(*builder);
    assert(M.plist_write_key_value(*builder, "a<&\"'", "value<&\"'\r\né"));
    assert(M.plist_write_key_value(*builder, "true", true));
    M.plist_write_key(*builder, "false");
    M.plist_write_bool(*builder, false);
    assert(M.plist_write_key_value(*builder, "negative", cast(s64) -9223372036854775807 - 1));
    assert(M.plist_write_key_value(*builder, "unsigned", cast(u64) 18446744073709551615));

    #if #exists(print_float) {
        assert(M.plist_write_key_value(*builder, "real", cast(float64) 0.125));
        assert(M.plist_write_key_value(*builder, "large-real", cast(float64) 1.25e100));
        bits: u64 = 0x7ff0000000000000;
        nonfinite: float64;

        memcpy(*nonfinite, *bits, 8);
        assert(!M.plist_write_key_value(*builder, "infinity", nonfinite));
        bits = 0x7ff8000000000000;
        memcpy(*nonfinite, *bits, 8);
        assert(!M.plist_write_key_value(*builder, "nan", nonfinite));
    } else {
        before_real := builder_string_length(*builder);
        assert(!M.plist_write_key_value(*builder, "real", cast(float64) 0.125));
        assert(before_real == builder_string_length(*builder));
    }

    before := builder_string_length(*builder);
    invalid: [1] u8 = .[0xff];

    assert(!M.plist_write_key_value(*builder, "bad", cast(string) .{ 1, invalid.data }));
    invalid[0] = 1;
    assert(!M.plist_write_key_value(*builder, "bad", cast(string) .{ 1, invalid.data }));

    record: struct {
        value: s64;
    };

    assert(!M.plist_write_key_value(*builder, "unsupported", record));
    assert(builder_string_length(*builder) == before);
    M.plist_end(*builder);
    value := builder_to_string(*builder);

    defer free(value);
    write_strings(value);
}
