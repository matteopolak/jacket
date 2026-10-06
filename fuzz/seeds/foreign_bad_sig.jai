strlen :: (s: [100] u8, x: Any) -> [3] float #foreign libc;
libc :: #system_library "libc";
main :: () { a: [100] u8; x := strlen(a, 1); }
