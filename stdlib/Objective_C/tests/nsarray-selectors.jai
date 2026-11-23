// test: only(.MACOS)
// NSArray built from NSString objects reports count, order and content-based containment, and selectors answer respondsToSelector: and isKindOfClass: as expected.
#import "Basic";
#import "Objective_C";

libobjc :: #library,system "objc";
msg_send :: () #foreign libobjc "objc_msgSend";
sel_name :: (name: *u8) -> *void #foreign libobjc "sel_registerName";

sel_register :: (name: string) -> *void {
    return sel_name(to_c_string(name));
}

SendSelectorQuery :: #type (receiver: *void, op: *void, a0: *void) -> s8 #c_call;

to_jai_string :: (s: *NSString) -> string {
    p := NSString.UTF8String(s);
    return string.{ cast(s64) c_style_strlen(p), p };
}

main :: () {
    init_objective_c();
    init_objective_c_foundation();

    items: [3] *NSString;
    items[0] = NSString.getTempString("one");
    items[1] = NSString.getTempString("two");
    items[2] = NSString.getTempString("three");

    array := NSArray(*NSString).arrayWithObjects(*items[0], 3);

    print("count: %\n", NSArray(*NSString).count(array));
    print("second: %\n", to_jai_string(NSArray(*NSString).objectAtIndex(array, 1)));
    print("contains two: %\n", NSArray(*NSString).containsObject(array, NSString.getTempString("two")) != NO);
    print("contains four: %\n", NSArray(*NSString).containsObject(array, NSString.getTempString("four")) != NO);

    query := cast(SendSelectorQuery) msg_send;

    print("responds to length: %\n", query(cast(*void) items[0], sel_register("respondsToSelector:"), sel_register("length")) != NO);
    print("responds to nonexistent: %\n", query(cast(*void) items[0], sel_register("respondsToSelector:"), sel_register("noSuchSelectorForThisTest")) != NO);
    print("is NSString: %\n", query(cast(*void) items[0], sel_register("isKindOfClass:"), cast(*void) objc_getClass("NSString")) != NO);
    print("is NSArray: %\n", query(cast(*void) array, sel_register("isKindOfClass:"), cast(*void) objc_getClass("NSArray")) != NO);
}
