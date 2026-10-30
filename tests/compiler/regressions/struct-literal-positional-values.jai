// A positional struct literal may leave off trailing fields that have an initializer of their own
// (the official compiler rejects it, so this warns with positional-literal-defaults),
// and a literal may always name its fields instead.
#import "Basic";

Format :: struct {
    internal, external: u32;
    compressed := false;
    scale := 2;
}

Frame :: struct {
    id: int;
    next := -1;
}

Pair :: struct {
    a, b: int;
}

Either :: union {
    small: u8;
    big: u64;
}

main :: () {
    full := Format.{ 1, 2, true, 4 };

    assert(full.internal == 1 && full.external == 2 && full.compressed && full.scale == 4);

    short := Format.{ 1, 2 };

    assert(!short.compressed && short.scale == 2);

    frame := Frame.{ 7 };

    assert(frame.id == 7 && frame.next == -1);

    named := Pair.{ b = 3 };

    assert(named.a == 0 && named.b == 3);

    empty := Pair.{};

    assert(empty.a == 0 && empty.b == 0);

    either := Either.{ 5 };

    assert(either.small == 5);

    print("ok\n");
}
