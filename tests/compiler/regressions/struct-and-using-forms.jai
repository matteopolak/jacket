// Struct features: defaults, literals, nested overrides, layout, polymorphic structs, unions,
// and `using` in its several forms.
#import "Basic";

Color :: struct {
    r, g, b: float;
}

Spaceship :: struct {
    fuel_level := 85.0;
    model := "falcon";
    num_engines := 4;
    hull_type := "titanium";
    has_shields := false;
}

Hangar_Display :: struct {
    luminance: float;
    ship: Spaceship;
    ship.model = "interceptor";
    tint: Color;
    tint.b = 1.5;
}

Gradient :: struct {
    start: Color;
    end: Color;
    name: string;
}

Header :: struct {
    name: string;
}

Payload :: struct {
    using header: Header;
    readings: [3] int;
    footer := "End.";
}

Bounds :: struct {
    min_x, min_y, max_x, max_y: float;
    label: string;
}

Padded :: struct {
    a: u8;
    b: s64;
    c: u8;
}

Buffer :: struct(T: Type, N: int) {
    items: [N] T;
    cursor: int;
}

Grid :: struct(Cell: Type, Width: int = 10, Height: int = 10) {
    cells: [Width][Height] Cell;
}

Pair :: struct(A: Type, B: Type) {
    first: A;
    second: B;
}

Node :: struct(T: Type) {
    value: T;
    prev: *Node(T);
    next: *Node(T);
}

List :: struct(T: Type) {
    head: *Node(T);
    tail: *Node(T);
}

Value :: union {
    as_int: s64;
    as_float: float64;
    as_bytes: [8] u8;
}

Token_Kind :: enum {
    INTEGER;
    FLOAT;
}

Token_Value :: union {
    as_int: s64;
    as_float: float64;
}

Token :: struct {
    kind: Token_Kind;
    value: Token_Value;
}

Coordinate :: struct {
    x, y, z: float;
}

Unit :: struct {
    using location: Coordinate;
    id: s64;
}

Robot :: struct {
    using unit: Unit;
    serial_code: string;
    weight_in_kg: float;
}

Vehicle :: struct {
    make: string;
}

Truck :: struct {
    using #as v: Vehicle;
    payload_capacity: float;
}

Helicopter :: struct {
    model := "Apache";
    rotors := 2;
}

Gyroscope :: struct {
    pitch, yaw, roll: float;
}

make_spaceship :: () -> *Spaceship {
    return New(Spaceship);
}

upgrade :: (using ship: *Spaceship) {
    num_engines *= 3;
    has_shields = true;
}

vehicle_make :: (v: Vehicle) -> string {
    return v.make;
}

main :: () {
    // Fields are zero unless a default is given.
    b: Bounds;

    assert(b.min_x == 0 && b.label == "");
    b.max_x = 50;
    c := b;
    c.max_x = 7;
    assert(b.max_x == 50 && c.max_x == 7);

    craft: Spaceship;

    assert(craft.fuel_level == 85 && craft.model == "falcon" && craft.num_engines == 4);
    assert(craft.hull_type == "titanium" && !craft.has_shields);

    // Overriding the defaults of nested members inside the struct declaration.
    hangar: Hangar_Display;

    assert(hangar.ship.model == "interceptor" && hangar.ship.num_engines == 4);
    assert(hangar.tint.b == 1.5 && hangar.tint.r == 0);

    g: Gradient;
    g.start.r = 0.5;
    g.name = "dawn";
    assert(g.start.r == 0.5 && g.end.r == 0 && g.name == "dawn");

    // Copies hold pointers and strings shallowly.
    text := "shared";
    p1: Header;
    p1.name = text;
    p2 := p1;

    assert(p2.name.data == p1.name.data);

    // Named and positional literals.
    loadout := Spaceship.{ model = "interceptor", num_engines = 6 };

    assert(loadout.model == "interceptor" && loadout.num_engines == 6 && loadout.fuel_level == 85);
    other: Spaceship;
    other = .{ model = "cruiser", num_engines = 12 };
    assert(other.num_engines == 12 && other.hull_type == "titanium");
    empty: Spaceship = .{};

    assert(empty.num_engines == 4);
    col := Color.{ 0.2, 0.5, 0.9 };

    assert(col.g == 0.5);
    blue: Color = .{ 0, 0, 1 };

    assert(blue.b == 1);

    payload := Payload.{ name = "Sensor_A", readings[1] = 7 };

    assert(payload.name == "Sensor_A" && payload.readings[1] == 7 && payload.readings[0] == 0);
    assert(payload.footer == "End.");

    // Size and layout.
    assert(size_of(Color) == 12);
    assert(size_of(Padded) == 24);
    assert(size_of(type_of(b)) == size_of(Bounds));
    assert(size_of(Value) == 8);
    p: Padded;

    assert(cast(*u8) *p.b - cast(*u8) *p == 8);
    assert(cast(*u8) *b == cast(*u8) *b.min_x);

    // Polymorphic structs.
    buf: Buffer(float, 8);

    assert(buf.items.count == 8 && buf.N == 8 && buf.T == float);
    large: Buffer(int, 4096);

    assert(size_of(type_of(large)) == 4096 * 8 + 8);
    assert(type_of(buf) != type_of(large));

    grid: Grid(int);

    assert(grid.Width == 10 && grid.Height == 10 && grid.Cell == int);
    sized: Grid(int, 5, 8);

    assert(sized.Width == 5 && sized.Height == 8);
    assert(type_of(sized) == Grid(int, 5, 8));
    assert(type_of(sized) != Grid(int, 8, 5));

    Square_Grid :: #bake_arguments Grid(Width = 16, Height = 16);
    inventory: Square_Grid(Cell = string);

    assert(inventory.Width == 16 && inventory.Cell == string);

    numbers: List(int);
    words: List(string);

    assert(numbers.head == null && words.tail == null);
    node: Node(int);
    node.value = 3;
    numbers.head = *node;
    assert(numbers.head.value == 3);

    p3 := Pair(int, string).{ 42, "hello" };

    assert(p3.first == 42 && p3.second == "hello");
    p4: Pair(float, float) = .{ 1.5, 2.5 };

    assert(p4.first + p4.second == 4);

    // Unions share their storage.
    v: Value;
    v.as_int = 42;
    assert(v.as_bytes[0] == 42 && v.as_bytes[1] == 0);
    v.as_float = 1.0;
    assert(v.as_int == 0x3FF0000000000000);

    t: Token;
    t.kind = .FLOAT;
    t.value.as_float = 2.5;
    assert(t.kind == .FLOAT && t.value.as_float == 2.5);

    // using on a variable.
    ship: Spaceship;

    {
        using ship;
        num_engines = 6;
        model = "interceptor";
    }

    assert(ship.num_engines == 6 && ship.model == "interceptor");

    {
        using ship;
        fuel_level = 50;
        assert(ship.fuel_level == 50);
        ship.fuel_level = 30;
        assert(fuel_level == 30);
    }

    // using on a declaration, and through a pointer.
    using held: Spaceship;
    assert(model == "falcon");
    num_engines = 9;
    assert(held.num_engines == 9);

    pointer := make_spaceship();

    {
        using pointer;
        num_engines *= 2;
        num_engines += 1;
    }

    assert(pointer.num_engines == 9);
    free(pointer);

    // using on a procedure parameter.
    upgrade(*ship);
    assert(ship.num_engines == 18 && ship.has_shields);

    // using in a struct promotes the members through several levels.
    bot: Robot;
    bot.id = 1337;
    bot.x = 3;
    bot.y = 9;
    assert(bot.unit.id == 1337 && bot.unit.location.x == 3 && bot.location.y == 9);
    assert(bot.z == 0);
    bot.location.z = 4;
    assert(bot.unit.z == 4);

    // #as allows the implicit conversion to the inner type.
    truck: Truck;
    truck.make = "Ford";
    veh: Vehicle;
    veh = truck;
    assert(veh.make == "Ford");
    assert(vehicle_make(truck) == "Ford");

    // using,except and using,only.
    {
        using,except(model) heli: Helicopter;
        assert(rotors == 2);
        rotors = 4;
        assert(heli.rotors == 4 && heli.model == "Apache");
    }

    {
        using,only(pitch, yaw) rotation: Gyroscope;
        pitch = 1;
        yaw = 2;
        assert(rotation.pitch == 1 && rotation.yaw == 2 && rotation.roll == 0);
    }

    // using on an enum brings its members into scope.
    Planets :: enum {
        MARS;
        VENUS;
        JUPITER;
    }

    {
        using Planets;
        x := VENUS;
        assert(x == Planets.VENUS);
    }

    print("ok\n");
}
