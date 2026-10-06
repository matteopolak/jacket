// rules: opov.8
// `s[i] = v` on a type with only `operator *[]` assigns through the returned pointer, even
// when an `operator []=` for another type is in scope.
#import "Basic";

Slots :: struct {
    cells: [3] int;
}
operator *[] :: (s: *Slots, i: int) -> *int {
    return *s.cells[i];
}

Other :: struct {
    v: int;
}
operator []= :: (o: *Other, i: int, value: int) {
    o.v = value;
}

main :: () {
    s: Slots;
    s[1] = 9;
    s[2] += 4;
    o: Other;
    o[0] = 5;
    assert(s.cells[1] == 9 && s.cells[2] == 4 && o.v == 5);
    print("ok\n");
}
