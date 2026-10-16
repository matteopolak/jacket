// Reviewed class naming, first-member reservation, warning/null policy and
// real instance/metaclass dispatch. Mismatched records never back an object.
O :: #import "Objective_C";

libc :: #library,system "c";
abort :: () #foreign libc;
objc :: #library,system "objc";
send_address :: () #foreign objc "objc_msgSend";

check :: (condition: bool) {
    if !condition abort();
}

Logs :: struct {
    warnings: s64;
    errors: s64;
}

observe :: (message: string, data: *void, info: Log_Info) {
    check(message.count > 0 && data != null);
    logs := cast(*Logs) data;
    if (info.common_flags & .WARNING) != 0 logs.warnings += 1;
    if (info.common_flags & .ERROR) != 0 logs.errors += 1;
}

ReviewedRegisteredObject :: struct {
    using parent: O.NSObject;
    value: s64 = 713;
}

MissingReviewedParent :: struct {
    isa: O.Class;
}

MissingReviewedChild :: struct {
    using parent: MissingReviewedParent;
    value: s64;
}

// Same reflected name as the real native parent, intentionally different size.
// The reviewed API warns; no instance is accessed through this record.
NSObject :: struct {
    isa: O.Class;
    extra: s64;
}

ReviewedWarningChild :: struct {
    using parent: NSObject;
    value: s64;
}

instance_value :: (self: *ReviewedRegisteredObject, selector: O.Selector, amount: s32) -> s32 #c_call {
    return cast(s32) (self.value + amount);
}

class_value :: (self: O.Class, selector: O.Selector, amount: s32) -> s32 #c_call {
    return amount - 13;
}

main :: () {
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    previous_logger := context.logger;
    previous_data := context.logger_data;
    logs: Logs;

    context.logger = observe;
    context.logger_data = cast(*void) *logs;

    defer {
        context.logger = previous_logger;
        context.logger_data = previous_data;
    }

    check(O.objc_create_class(MissingReviewedChild, MissingReviewedParent) == null);
    check(logs.errors == 1 && logs.warnings == 0);
    created := O.objc_create_class(ReviewedRegisteredObject, O.NSObject);

    check(created != null && logs.errors == 1 && logs.warnings == 0);
    check(O.class_getInstanceSize(created) == size_of(ReviewedRegisteredObject));
    check(O.ivar_getOffset(O.class_getInstanceVariable(created, "value")) == 8);
    O.objc_add_instance_method(created, instance_value, "number:");
    O.objc_add_class_method(created, class_value, "classNumber:");
    O.objc_finalize_class(created);
    object := O.objc_new(ReviewedRegisteredObject);

    check(object != null && O.class(cast(O.id) object) == created);
    // Native object construction does not copy Jai record field defaults.
    check(object.value == 0);
    object.value = -17;
    Send :: #type (self: *ReviewedRegisteredObject, selector: O.Selector, amount: s32) -> s32 #c_call;
    send := cast(Send) send_address;

    check(send(object, O.selector("number:"), -3) == -20);
    ClassSend :: #type (self: O.Class, selector: O.Selector, amount: s32) -> s32 #c_call;
    class_send := cast(ClassSend) send_address;

    check(class_send(created, O.selector("classNumber:"), -7) == -20);
    O.release(object);
    check(O.objc_create_class(ReviewedRegisteredObject, O.NSObject) == null);
    check(logs.errors == 2 && logs.warnings == 0);

    mismatch := O.objc_create_class(ReviewedWarningChild, NSObject);

    check(mismatch != null && logs.warnings == 2 && logs.errors == 2);
    check(O.class_getInstanceSize(mismatch) != size_of(ReviewedWarningChild));
    check(O.ivar_getOffset(O.class_getInstanceVariable(mismatch, "value")) == 8);
    O.objc_disposeClassPair(mismatch);
}
