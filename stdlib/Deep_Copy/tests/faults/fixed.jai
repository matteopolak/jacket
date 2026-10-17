D :: #import "Deep_Copy";

main :: () {
    value: [2] u32;
    D.Deep_Copy(*value, .{});
}
