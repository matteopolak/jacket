// `#modify` on a polymorphic struct may change its parameters (values and types); the final
// values select the instance.
#import "Basic";

Holder :: struct(N: int, T: Type)
#modify {
    if N < 8 N = 8;
    if T == u8 T = u16;
    return true;
}

{
    values: [N] T;
}

Positive :: struct(N: int) #modify {
    return N > 0, "N must be positive";
} {
    x: [N] int;
}

main :: () {
    a: Holder(3, float);
    b: Holder(8, float);
    c: Holder(20, u8);
    assert(a.N == 8 && a.values.count == 8);
    assert(type_of(a) == type_of(b));
    assert(c.T == u16 && c.values.count == 20);
    p: Positive(2);
    assert(p.x.count == 2);
    #assert size_of(Holder(1, u8)) == 16;
    print("ok\n");
}
