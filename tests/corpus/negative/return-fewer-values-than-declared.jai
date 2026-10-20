// A `return` supplies every declared result.
#import "Basic";
f :: () -> int, int { return 1; }
main :: () { a, b := f(); }
