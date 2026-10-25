#import "Basic";

Slot :: struct {
    offset: u32;
}

measure :: () -> s64 {
    return 12;
}

main :: () {
    slot := Slot.{ offset = #run measure() };
    assert(slot.offset == 12, "a #run integer takes the field's integer type when it fits");
    print("ok\n");
}
