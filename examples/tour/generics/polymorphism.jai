// Polymorphism: write it once, the compiler stamps out a copy per type.

// `$T` introduces a type variable, inferred from the call.
largest :: (values: [] $T) -> T {
    best := values[0];
    for values if it > best best = it;
    return best;
}

swap_values :: (a: *$T, b: *T) {
    a.*, b.* = b.*, a.*;
}

// Structs take parameters too: here a type and a compile-time size.
Ring :: struct(Item: Type, CAPACITY: int) {
    items: [CAPACITY] Item;
    start, count: int;
}

push :: (ring: *Ring, item: ring.Item) {
    slot := (ring.start + ring.count) % ring.CAPACITY;
    ring.items[slot] = item;
    if ring.count < ring.CAPACITY ring.count += 1;
    else ring.start = (ring.start + 1) % ring.CAPACITY; // full: forget the oldest
}

// `$R/Ring` accepts any Ring, whatever its parameters.
show_ring :: (ring: $R/Ring) {
    print("  ring of % % (holds % of %):", ring.count, R.Item, ring.count, R.CAPACITY);
    for 0..ring.count - 1 print(" %", ring.items[(ring.start + it) % ring.CAPACITY]);
    print("\n");
}

// `/interface` accepts any type that has the fields of Has_Position.
Has_Position :: struct {
    x, y: float;
}

Ship :: struct {
    name: string;
    x, y: float;
    cargo: int;
}

distance_from_origin :: (thing: $T/interface Has_Position) -> float {
    return sqrt(thing.x * thing.x + thing.y * thing.y);
}

// `#modify` runs at compile time on each instantiation and may change or reject it.
// Here, small integers are widened to s64 so the running total cannot overflow.
running_total :: (values: [] $T) -> $R #modify {
    R = T;
    info := cast(*Type_Info) T;
    if info.type == .INTEGER R = s64;
    return true;
} {
    total: R;
    for values total += cast(R) it;
    return total;
}

// `$$` bakes an argument when it is a compile-time constant, and passes it normally otherwise.
power :: (base: int, $$exponent: int) -> result: int, baked: bool {
    // Only a baked copy knows the exponent, so only it can check it at compile time.
    #if is_constant(exponent) #assert(exponent >= 0);
    result := 1;
    for 1..exponent result *= base;
    return result, is_constant(exponent);
}

// `#bake_arguments` makes a new procedure with some arguments fixed.
scale_by :: (value: float, factor: float) -> float {
    return value * factor;
}

double_it :: #bake_arguments scale_by(factor = 2);

tour_polymorphism :: () {
    print("largest int: %, float: %, u8: %\n",
        largest(int.[3, 9, 4]), largest(float.[2.5, -1, 0.5]), largest(u8.[7, 250, 13]));

    left, right := "left", "right";
    swap_values(*left, *right);
    print("after swap_values: % %\n", left, right);

    recent: Ring(string, 3);
    for string.["one", "two", "three", "four", "five"] push(*recent, it);
    show_ring(recent);
    dice: Ring(int, 4);
    for 1..6 push(*dice, it * 11);
    show_ring(dice);

    ship := Ship.{ name = "Rocinante", x = 3, y = 4, cargo = 12 };
    print("% is % units from home.\n", ship.name, distance_from_origin(ship));

    byte_total := running_total(u8.[200, 200, 200]);
    float_total := running_total(float.[0.5, 0.25]);
    print("u8 total: % (an %), float total: % (a %)\n",
        byte_total, type_of(byte_total), float_total, type_of(float_total));

    n := 5;
    a, a_baked := power(2, 10);
    b, b_baked := power(3, n);
    print("power(2, 10) = % (baked: %), power(3, n) = % (baked: %)\n", a, a_baked, b, b_baked);
    print("double_it(21) = %\n", double_it(21));
}
