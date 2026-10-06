// A string constant can be cast to a fixed [N] u8 / [N] s8 of the same length.
#import "Basic";

main :: () {
    a := cast([5] u8) "Hello";
    b := cast([3] s8) "abc";
    if a[0] != #char "H" || a[4] != #char "o" {
        print("FAIL u8\n");
        exit(1);
    }
    if b[1] != 98 {
        print("FAIL s8\n");
        exit(1);
    }
    print("ok\n");
}
