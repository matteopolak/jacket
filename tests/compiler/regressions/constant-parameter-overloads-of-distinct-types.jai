// Overloads whose constant parameters (`$c: A`, `$c: B`) have distinct types pick the one whose
// type the constant argument has.
#import "Basic";
#import "Math";

Position :: #type,distinct Vector2;
Velocity :: #type,distinct Vector2;
Health :: #type,distinct float32;

POSITION :: #run -> Position {
    value: Position;
    return value;
};

VELOCITY :: #run -> Velocity {
    value: Velocity;
    return value;
};

HEALTH :: #run -> Health {
    value: Health;
    return value;
};

get :: inline (id: u8, $component: Position) -> int {
    return 1;
}

get :: inline (id: u8, $component: Velocity) -> int {
    return 2;
}

get :: inline (id: u8, $component: Health) -> int {
    return 3;
}

main :: () {
    assert(get(0, POSITION) == 1);
    assert(get(0, VELOCITY) == 2);
    assert(get(0, HEALTH) == 3);

    print("ok\n");
}
