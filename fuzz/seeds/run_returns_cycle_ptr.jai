N :: struct { next: *N; }
f :: () -> *N { n := New(N); n.next = n; return n; }
#import "Basic";
V :: #run f();
main :: () { x := V.next.next; }
