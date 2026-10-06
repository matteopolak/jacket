// An `ifx` argument picks the overload its branches fit.
#import "Basic";
T :: struct {
    n: int;
}
eat :: inline (using t: *T, c: u8) {
    n += 1;
}
eat :: (using t: *T, s: string) {
    n += s.count;
}
main :: () {
    t: T;
    html := true;
    eat(*t, ifx html then "-->" else "--");
    eat(*t, #char "a");
    assert(t.n == 4);
}
