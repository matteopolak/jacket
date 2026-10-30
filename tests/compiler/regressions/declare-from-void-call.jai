// `x := call();` of a procedure that returns nothing makes the call and declares `x` as a `void`
// variable, which takes no space.
// rules: decl.26
#import "Basic";

calls := 0;

touch :: () {
    calls += 1;
}

main :: () {
    result := touch();

    assert(calls == 1);
    assert(type_of(result) == void);
    assert(size_of(type_of(result)) == 0);
    kept := touch();

    assert(calls == 2);
    assert(type_of(kept) == void);
    print("ok\n");
}
