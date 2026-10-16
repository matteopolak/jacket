// Default Type values denote void, including record fields.
// Independently confirmed by the coordinator official native oracle.
#import "Basic";

Tagged :: struct {
    tag: Type;
    value: s64;
}

main :: () {
    t: Type;
    print("%\n", t);
    x: Tagged;
    print("%\n", x);
}
