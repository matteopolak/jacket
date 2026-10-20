// Only a string constant converts to `*u8`; a variable needs `to_c_string`.
#import "Basic";
strlen :: (s: *u8) -> s64 #foreign;
main :: () {
    x := "Hello";
    n := strlen(x);
}
