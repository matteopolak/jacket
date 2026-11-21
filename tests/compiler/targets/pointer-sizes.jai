#import "Basic";

Pair :: struct {
    a: u8;
    p: *void;
    b: s64;
}

main :: () {
    print("pointer %\n", size_of(*void));
    print("string %\n", size_of(string));
    print("array %\n", size_of([..] u8));
    print("any %\n", size_of(Any));
    print("allocator %\n", size_of(Allocator));
    print("pair % %\n", size_of(Pair), align_of(Pair));
    text := tprint("%-%", "heap", 42);

    print("%\n", text);
}
