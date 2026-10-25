#import "Basic";

Ring :: struct($CAPACITY: s64) {
    head: u64;
}

wrap :: ($LIMIT: s64, index: u64) -> u64 {
    return index % LIMIT;
}

main :: () {
    ring: Ring(8);
    index: u64 = 19;

    assert(index % ring.CAPACITY == 3, "a struct constant converts to the integer type it meets");
    assert(wrap(8, index) == 3, "a baked procedure constant converts to the integer type it meets");
    print("ok\n");
}
