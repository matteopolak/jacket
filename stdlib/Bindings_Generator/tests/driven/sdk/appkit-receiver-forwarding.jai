A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";

Convert :: #type (self: O.id, point: O.NSPoint, fromView: *A.NSView) -> O.NSPoint;
Repeat :: #type (self: *A.NSEvent) -> O.NSTimeInterval;
Buttons :: #type (self: *A.NSEvent) -> O.NSUInteger;
#assert type_of(A.NSView.convertPoint) == Convert;
#assert type_of(A.NSEvent.keyRepeatDelay) == Repeat;
#assert type_of(A.NSEvent.keyRepeatInterval) == Repeat;
#assert type_of(A.NSEvent.pressedMouseButtons) == Buttons;
#assert size_of(O.NSPoint) == 16;
#assert size_of(O.NSTimeInterval) == 8;
#assert size_of(O.NSUInteger) == 8;

probe :: #library,system "appkit_receiver_probe";
create :: (parent: *O.id, child: *O.id, event: *O.id) -> bool #foreign probe "receiver_create";
destroy :: (parent: O.id, child: O.id, event: O.id) #foreign probe "receiver_destroy";
native_point :: (view: O.id, point: O.NSPoint, from: O.id) -> O.NSPoint #foreign probe "receiver_point";
native_delay :: () -> O.NSTimeInterval #foreign probe "receiver_delay";
native_interval :: () -> O.NSTimeInterval #foreign probe "receiver_interval";
native_buttons :: () -> O.NSUInteger #foreign probe "receiver_buttons";
sdk_contract :: () -> bool #foreign probe "receiver_contract";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

main :: () {
    check(sdk_contract());
    parent, child, event: O.id;

    check(create(*parent, *child, *event));
    defer destroy(parent, child, event);

    point: O.NSPoint = .{ 17, 29 };
    from := cast(*A.NSView) child;
    actual := A.NSView.convertPoint(self = parent, point = point, fromView = from);
    expected := native_point(parent, point, child);

    check(actual.x == expected.x && actual.y == expected.y);
    check(actual.x == 27 && actual.y == 41);
    reverse := A.NSView.convertPoint(child, actual, cast(*A.NSView) parent);

    check(reverse.x == point.x && reverse.y == point.y);

    // The legacy parameter remains public, but neither null nor an actual
    // event changes the class receiver. The helper checks all native encodings.
    receivers: [2] *A.NSEvent = .[null, cast(*A.NSEvent) event];
    for self: receivers {
        delay := A.NSEvent.keyRepeatDelay(self = self);
        interval := A.NSEvent.keyRepeatInterval(self = self);

        check(delay >= 0 && delay == native_delay());
        check(interval > 0 && interval == native_interval());
        check(A.NSEvent.pressedMouseButtons(self = self) == native_buttons());
    }

    check(sdk_contract());
}
