puts :: (s: *u8) -> s32 #foreign libc;
libc :: #system_library "libc";
main :: () { puts("hi".data); }
