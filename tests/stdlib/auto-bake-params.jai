#import "Basic";

stub :: ($$x: int) -> bool {
    return is_constant(x);
}

sum :: ($$n: int, k: int) -> int {
    return n + k;
}

main :: () {
    v := 7;
    assert(stub(5));
    assert(!stub(v));
    assert(sum(3, 4) == 7);
    assert(sum(v, 1) == 8);

    p1 :: #procedure_of_call stub(5);
    p2 :: #procedure_of_call stub(173);
    p3 :: #procedure_of_call stub(v);
    p4 :: #procedure_of_call stub(5);
    p5 :: #procedure_of_call stub(v + 1);
    assert(p1 == p4);
    assert(p1 != p2);
    assert(p3 == p5);
    print("ok\n");
}
