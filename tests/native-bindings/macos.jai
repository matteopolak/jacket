M :: #import "macOS";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    original := "UTF-8: café\0tail";
    text := M.cfstring_from_string(original);

    check(text != null);
    defer M.CFRelease(text);
    result := M.to_string(text);

    defer context.allocator.proc(.FREE, 0, 0, result.data, context.allocator.data);
    check(result == original);
    empty := M.cfstring_from_string("");

    check(empty != null);
    defer M.CFRelease(empty);
    check(M.to_string(empty).count == 0);
}
