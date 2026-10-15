// Reviewed module filters expose only selected short names. The named module
// remains available for explicit qualified access to its other public exports.
// rules: import.17 import.18
using,only(print, String_Builder) B :: #import "Basic";
using,only(join) S :: #import "String";

#assert !#exists(contains);

assert :: (ok: bool) {
    if !ok exit(1);
} // Basic's assert is not the one picked

exit :: (code: s32) #foreign libc;
libc :: #system_library "libc";

main :: () {
    assert(join("a", "b") == "ab");
    assert(S.contains("hello", "ell"));
    builder: String_Builder; // jailint: allow(unused_variable) names a listed type
    print("ok\n");
}
