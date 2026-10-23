// `operator *[]` takes a pointer to the receiver. Indexing a constant, or a value a call
// returns, passes a temporary copy.
#import "Basic";

Color :: enum {
    Red;
    Green;
}

Enumerated_Array :: struct(E: Type, T: Type) {
    data: [2] T;
}

operator *[] :: (array: *Enumerated_Array($E, $T), index: E) -> *T {
    return *array.data[cast(s64) index];
}

TABLE :: Enumerated_Array(Color, int).{ data = .[10, 20] };

make :: () -> Enumerated_Array(Color, int) {
    return TABLE;
}

main :: () {
    local := TABLE;

    assert(local[Color.Green] == 20);
    assert(TABLE[Color.Green] == 20);
    assert(TABLE[Color.Red] + 1 == 11);
    assert(make()[Color.Red] == 10);

    local[Color.Red] = 3;
    assert(local[Color.Red] == 3);
    assert(TABLE[Color.Red] == 10);
    print("ok\n");
}
