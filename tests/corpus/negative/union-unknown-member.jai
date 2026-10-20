// A union only has the members it declares.
#import "Basic";
U :: union { a: int; b: float; }
main :: () { u: U; u.c = 1; }
