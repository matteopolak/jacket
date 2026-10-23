#import "Basic";
R :: #import "internal/crypto/random";

main :: () {
    a: [1000] u8;
    b: [1000] u8;

    assert(R.random_bytes(a));
    assert(R.random_bytes(b));

    nonzero := false;
    for a if it != 0 nonzero = true;

    assert(nonzero);
    assert(memcmp(a.data, b.data, 1000) != 0);

    empty: [] u8;

    assert(R.random_bytes(empty));
    print("ok\n");
}
