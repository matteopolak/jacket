// A statement in a struct body that assigns one of the struct's own members sets its default.
#import "Basic";

Light :: struct {
    luminance: float;
    luminance = 101.2;

    count: int = 3;
    count = 9;

    name: string;
    name = "lamp";
}

Base :: struct {
    level: int;
}

Derived :: struct {
    using base: Base;
    level = 12;
    extra: int = 5;
}

main :: () {
    light: Light;

    assert(light.luminance == 101.2);
    assert(light.count == 9);
    assert(light.name == "lamp");

    explicit := Light.{ luminance = 1 };

    assert(explicit.luminance == 1 && explicit.count == 9);

    derived: Derived;

    assert(derived.level == 12 && derived.extra == 5);

    print("ok\n");
}
