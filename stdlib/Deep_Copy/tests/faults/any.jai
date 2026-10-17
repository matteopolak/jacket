D :: #import "Deep_Copy";

main :: () {
    integer: u32;
    value: Any = integer;
    D.Deep_Copy(*value, .{});
}
