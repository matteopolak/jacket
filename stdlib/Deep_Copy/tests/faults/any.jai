// test: should_fail("Deep_Copy does not support this reflected value kind")
D :: #import "Deep_Copy";

main :: () {
    integer: u32;
    value: Any = integer;
    D.Deep_Copy(*value, .{});
}
