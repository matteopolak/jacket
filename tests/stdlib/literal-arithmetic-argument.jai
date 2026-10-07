// An argument such as `.{300, -1} * scale` builds the untyped literal as the parameter's
// type before the operator is looked up, and a `name:` marker inside a declaration
// statement (`ok, value:, extra := f()`) declares that name like the others.
// rules: cast.10
#import "Basic";
#import "Math";

width_of :: (size: Vector2) -> float {
    return size.x;
}

width_or_default :: (size := Vector2.{ -1, -1 }) -> float {
    return size.x;
}

three :: () -> bool, int, float {
    return true, 7, 0.5;
}

main :: () {
    scale := 2.0;
    assert(width_of(.{ 300, -1 } * scale) == 600);
    assert(width_or_default(.{ 10, 0 } * scale + .{ 1, 0 }) == 21);

    ok := false;
    {
        ok, value:, extra := three();
        assert(ok && value == 7 && extra == 0.5);
    }
    assert(!ok);
    print("ok\n");
}
