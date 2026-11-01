// test: only(.MACOS)
#import "Basic";
C :: #import "internal/window/cocoa";
O :: #import "internal/bindings/objc-api";

main :: () {
    pool := C.autorelease_pool();

    defer C.release(pool);
    rect := O.NSRect.{ .{ 12, 34 }, .{ 56, 78 } };
    value := C.argument_result_message(O.objc_getClass("NSValue"), "valueWithRect:", rect, *void);
    result := C.argument_result_message(value, "rectValue", cast(*void) null, O.NSRect);

    assert(result.origin.x == 12 && result.origin.y == 34);
    assert(result.size.width == 56 && result.size.height == 78);
}
