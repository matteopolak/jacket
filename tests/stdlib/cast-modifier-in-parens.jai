// A parenthesized expression whose cast carries a modifier is not a parameter list: the comma
// in `cast,no_check` must not make `(-cast,no_check(int) x)` look like `(a, b)`.
#import "Basic";

main :: () {
    loop: u64 = 12;
    lowest := loop & cast,no_check(u64)(-cast,no_check(int)loop);
    assert(lowest == 4);
    narrow: u8 = xx (-xx,trunc loop);
    assert(narrow == 244);
    print("ok\n");
}
