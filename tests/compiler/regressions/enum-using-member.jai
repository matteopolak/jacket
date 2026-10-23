// `using` inside an enum body brings in the members of another enum.
#import "Basic";

A :: enum {
    X;
    Y;
}

E :: enum {
    using A;
    Z;
}

main :: () {
    print("%\n", E.Z);
    assert(E.Z == .Z);
    assert(cast(s64) E.Z == 2);
}
