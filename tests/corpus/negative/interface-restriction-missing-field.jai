// `$T/interface S` rejects a struct that lacks one of S's fields.
#import "Basic";
Named :: struct { name: string; }
greet :: (x: $T/interface Named) {}
Other :: struct { id: int; }
main :: () {
    o: Other;
    greet(o);
}
