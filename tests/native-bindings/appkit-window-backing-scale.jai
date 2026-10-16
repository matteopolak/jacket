A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";

Scale :: #type (self: O.id) -> O.CGFloat;
#assert type_of(A.NSWindow.backingScaleFactor) == type_of(A.NSView.backingScaleFactor);
#assert type_of(A.NSWindow.backingScaleFactor) == Scale;
#assert size_of(O.CGFloat) == 8;

window_receiver :: (window: *A.NSWindow) -> O.CGFloat {
    return A.NSWindow.backingScaleFactor(window);
}

probe :: #library,system "appkit_backing_scale_probe";
create_receiver :: () -> O.id #foreign probe "binding_scale_create_receiver";
release_receiver :: (receiver: O.id) #foreign probe "binding_scale_release_receiver";
native_scale :: (receiver: O.id) -> O.CGFloat #foreign probe "binding_scale_value";
sdk_contract :: () -> bool #foreign probe "binding_scale_contract";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    window_scale: Scale = A.NSWindow.backingScaleFactor;
    target_scale: Scale = A.NSView.backingScaleFactor;

    check(cast(*void) window_scale == cast(*void) target_scale);
    check(sdk_contract());
    // Exact id receiver permits a real SDK NSScreen; no NSWindow is created.
    receiver := create_receiver();

    check(receiver != null);
    defer release_receiver(receiver);
    expected := native_scale(receiver);

    check(expected > 0);
    check(window_scale(receiver) == expected);
    check(target_scale(receiver) == expected);
    check(A.NSWindow.backingScaleFactor(self = receiver) == expected);
    check(sdk_contract());
}
