// rules: scope.16
#import "Basic";

// `type_of(field)` in a procedure nested inside a struct names the field's type.

Thing :: struct {
    x: float;
    z: s16;

    inside_struct :: () -> bool {
        t :: type_of(x);
        a: t = 111;
        a /= 10;
        assert(a > 11 && a < 12);

        inner :: () -> bool {
            return size_of(type_of(z)) == 2 && type_of(t) == Type;
        }
        return inner() && t == float;
    }
}

main :: () {
    assert(Thing.inside_struct());
}
