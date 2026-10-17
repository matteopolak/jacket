using #import "Basic";
A :: #import "Deferred_Probe"(VALUE = 41);
B :: #import "Deferred_Probe"(VALUE = 42);

#run,stallable {
    assert(A.evaluate(1) == 42);
    assert(B.evaluate(2) == 44);
    print("scoped bodies ready\n");
}

main :: () {
    assert(A.compute(1) == 42 && B.compute(2) == 44);
    print("scoped bodies runtime ok\n");
}
