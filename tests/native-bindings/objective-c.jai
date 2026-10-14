O :: #import "Objective_C";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    O.init_objective_c();
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    original := "A UTF-8 string: café";
    text := O.NSString.getTempString(original);

    check(text != null && O.to_string(text) == original);
    info := O.NSProcessInfo.processInfo();

    check(info != null);
    check(O.NSProcessInfo.processIdentifier(info) > 0);
    version := O.NSProcessInfo.operatingSystemVersion(info);

    check(version.majorVersion > 0);
    check(O.NSProcessInfo.systemUptime(info) > 0.0);
    check(O.selector("length") == O.sel_registerName("length"));

    selectors: struct {
        length: O.Selector;
        initWithBytes_length_encoding: O.Selector;
    }

    O.init_objective_c_selector_struct(*selectors);
    check(selectors.length == O.selector("length"));
    check(selectors.initWithBytes_length_encoding == O.selector("initWithBytes:length:encoding"));
}
