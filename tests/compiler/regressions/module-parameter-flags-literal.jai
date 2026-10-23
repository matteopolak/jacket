// A module parameter of an `enum_flags` type takes `.A | .B`, not only one `.Member`.
#import "Basic";

Default :: #import "Flag_Param";
One :: #import "Flag_Param"(MODES = .READ);
Two :: #import "Flag_Param"(MODES = .READ | .APPEND);
Three :: #import "Flag_Param"(MODES = .READ | .WRITE | .APPEND);

main :: () {
    assert(!Default.can_read() && Default.can_write() && !Default.can_append());
    assert(One.can_read() && !One.can_write() && !One.can_append());
    assert(Two.can_read() && !Two.can_write() && Two.can_append());
    assert(Three.can_read() && Three.can_write() && Three.can_append());
    print("ok\n");
}
