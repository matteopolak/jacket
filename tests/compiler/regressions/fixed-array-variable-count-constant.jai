// The count of a fixed-size array variable is part of its type, so `variable.count` sizes
// other arrays even though the variable itself is not a constant.
#import "Basic";

Slot :: struct {
    id: int;
}

slots := Slot.[.{ 1 }, .{ 2 }, .{ 3 }];

history: [slots.count] int;
doubled: [2 * slots.count] int;

main :: () {
    assert(history.count == 3 && doubled.count == 6);
    print("ok\n");
}
