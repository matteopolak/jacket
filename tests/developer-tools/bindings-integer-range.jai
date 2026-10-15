// Public signed-range helper, including the represented128-bit primitive width.
#import "Basic";
B :: #import "Bindings_Generator";

native :: #library "../../target/developer_tools_integer_range";
signed_range_exceeded :: (value: u64, bytes: u32) -> s32 #foreign native;

main :: () {
    values := u64.[0, 1, 127, 128, 32767, 32768, 0x7fffffff, 0x80000000, 0x7fffffffffffffff, 0x8000000000000000, 0xffffffffffffffff];

    for bytes: s64.[1, 2, 4, 8, 16] {
        target := B.CType.{ size = bytes, number_flags = .SIGNED };
        alias := B.Typedef.{ kind = .TYPEDEF, type = *target };
        alias_type := B.CType.{ type_of_typedef = *alias };
        enumeration := B.Enum.{ kind = .ENUM, type = *alias_type };
        enum_type := B.CType.{ type_of_enum = *enumeration };

        for value: values {
            expected := signed_range_exceeded(value, cast(u32) bytes) != 0;
            assert(B.exceeds_signed_int_value(value, *target) == expected);
            assert(B.exceeds_signed_int_value(value, *enum_type) == expected);
        }
    }

    assert(!B.exceeds_signed_int_value(0x7fffffffffffffff, null));
    assert(B.exceeds_signed_int_value(0x8000000000000000, null));
    print("ok\n");
}
