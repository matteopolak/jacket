// test: only(.MACOS)
// NSString built from UTF-8 reports its UTF-16 length, returns the same UTF-8 bytes, and matches a copy built with alloc and initWithString.
#import "Basic";
#import "Objective_C";

main :: () {
    init_objective_c();
    init_objective_c_foundation();

    text := "héllo, 世界";
    s := NSString.getTempString(text);

    print("length: %\n", NSString.length(s));

    bytes := NSString.UTF8String(s);
    n := c_style_strlen(bytes);

    print("utf8 bytes: %\n", n);
    returned := string.{ cast(s64) n, bytes };

    print("round trip equal: %\n", returned == text);

    empty := NSString.getTempString("");

    print("empty length: %\n", NSString.length(empty));

    copy := cast(*NSString) objc_alloc(objc_getClass("NSString"));
    copy = NSString.initWithString(copy, text);
    print("copy equal: %\n", isEqual(cast(id) copy, cast(id) s));
    print("copy length: %\n", NSString.length(copy));
    release(copy);
}
