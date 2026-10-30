// A `,,` context modifier after a call argument is not a union tag, even when an array
// literal earlier in the statement ends with a `]` that follows a `.[`.
#import "Basic";

total :: (values: [] int) -> int {
    sum := 0;

    for values sum += it;

    return sum;
}

pick :: (values: [] int) -> [] int {
    return values;
}

main :: () {
    a := total(pick(.[1, 2, 3]),, temp);

    assert(a == 6);

    a = total(pick(.[4, 5]),, temp);
    assert(a == 9);

    reset_temporary_storage();

    print("ok\n");
}
