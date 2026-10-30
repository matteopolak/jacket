// An element of a constant array of types names that type: it can be a cast target, a pointer or
// array element type, and a `size_of` argument. A variable array of types is still read at run time.
#import "Basic";

Health :: struct {
    hits: int;
}

Armor :: struct {
    points: int;
    extra: int;
}

registry :: Type.[Health, Armor];

main :: () {
    health: Health;
    health.hits = 7;
    raw: *void = *health;

    as_health := cast(*registry[0]) raw;

    assert(as_health.hits == 7);

    none: *registry[1];

    assert(none == null);

    plates: [2] registry[1];
    plates[1].points = 3;
    assert(plates[1].points == 3);

    assert(size_of(registry[0]) == 8);
    assert(size_of(registry[1]) == 16);

    chosen: [2] Type = .[Health, Armor];

    assert(chosen[1] == Armor);

    print("ok\n");
}
