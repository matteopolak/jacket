// `#overlay` on a member inside a statement-form `#if ... then`.
#import "Basic";

S :: struct {
    a: s64;
    #if true then #overlay(a) b: s64;
    else b: s64;
}

main :: () {
    s: S;
    s.a = 7;
    assert(s.b == 7);
    assert(size_of(S) == 8);
}
