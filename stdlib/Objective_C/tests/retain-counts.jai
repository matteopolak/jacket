// test: only(.MACOS)
// A new NSObject has retain count one; retain adds one and release subtracts one.
#import "Basic";
#import "Objective_C";

libobjc :: #library,system "objc";
msg_send :: () #foreign libobjc "objc_msgSend";
sel_name :: (name: *u8) -> *void #foreign libobjc "sel_registerName";

sel_register :: (name: string) -> *void {
    return sel_name(to_c_string(name));
}

RetainCountQuery :: #type (receiver: *void, op: *void) -> u64 #c_call;

retain_count :: (object: *void) -> u64 {
    query := cast(RetainCountQuery) msg_send;
    return query(object, sel_register("retainCount"));
}

main :: () {
    init_objective_c();

    object := objc_init(cast(*NSObject) objc_alloc(objc_getClass("NSObject")));

    assert(object != null);
    print("new: %\n", retain_count(object));

    retain(object);
    print("after retain: %\n", retain_count(object));
    retain(object);
    print("after second retain: %\n", retain_count(object));

    release(object);
    print("after release: %\n", retain_count(object));
    release(object);
    print("after second release: %\n", retain_count(object));
    release(object);
}
