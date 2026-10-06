// `using,only(...) #import "M";` brings the listed names in first; M's other names stay
// reachable as a last resort (code such as Epic_Fail relies on it), while a local
// declaration of the same name wins over M's.
using,only(print, String_Builder) #import "Basic";
using,only(join) #import "String";

assert :: (ok: bool) {
    if !ok exit(1);
} // Basic's assert is not the one picked
exit :: (code: s32) #foreign libc;
libc :: #system_library "libc";

main :: () {
    assert(join("a", "b") == "ab");
    assert(contains("hello", "ell")); // not listed: found through the fallback
    builder: String_Builder;
    print("ok\n");
}
