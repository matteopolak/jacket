// Declarations, constants and procedures: the bread and butter.

// `::` makes a constant. It lives at compile time and takes no storage.
LIGHT_SPEED :: 299_792_458; // metres per second; `_` separates digits
SHIP_NAME :: "Heart of Gold";

tour_basics :: () {
    // `:=` declares a variable and infers its type; `name: Type = value` spells it out.
    crew := 4;
    fuel: float = 0.75;
    log_entry: string; // Variables start zeroed unless you write `= ---`.
    print("The % carries % crew and % of a tank of fuel; log entry: \"%\"\n",
        SHIP_NAME, crew, fuel, log_entry);

    // Sized integers convert explicitly with `cast`; `xx` casts to whatever is expected.
    byte := cast,trunc(u8) 300; // keeps the low byte: 44
    small: s16 = xx crew;
    print("cast,trunc(u8) 300 = %, small = %, light speed = % km/s\n", byte, small, LIGHT_SPEED / 1000);

    // Procedures can be declared anywhere, including inside other procedures.
    cube :: (x: int) -> int {
        return x * x * x;
    }
    print("cube(7) = %\n", cube(7));

    // Several return values, with names, without tuples or out-parameters.
    quotient, remainder := divide(47, 5);
    print("47 / 5 is % remainder %\n", quotient, remainder);
    // Keep only the first return value if that is all you want.
    rounded_down := divide(99, 10);
    print("99 / 10 rounds down to %\n", rounded_down);

    // Named and default arguments.
    print("%\n", hail("Arthur"));
    print("%\n", hail("Ford", times = 3, ending = "?!"));

    // Overloading: the same name, picked by argument type.
    describe(42);
    describe(2.5);
    describe("forty-two");

    // Procedures are values: store them, pass them, call them later.
    operation := cube;
    print("operation(3) = %, apply_twice(cube, 2) = %\n", operation(3), apply_twice(cube, 2));

    // Anonymous procedures work as arguments too.
    print("apply_twice(x + 10, 1) = %\n", apply_twice((x: int) -> int {
        return x + 10;
    }, 1));
}

divide :: (a: int, b: int) -> quotient: int, remainder: int {
    return a / b, a % b;
}

hail :: (who: string, times := 1, ending := "!") -> string {
    // Everything allocated below comes from temporary storage (see memory/memory.jai).
    push_allocator(temp);
    builder: String_Builder;
    for 1..times append(*builder, "Hey ");
    print_to_builder(*builder, "%%", who, ending);
    return builder_to_string(*builder);
}

describe :: (value: int) {
    print("  % is an int\n", value);
}

describe :: (value: float) {
    print("  % is a float\n", value);
}

describe :: (value: string) {
    print("  \"%\" is a string of % bytes\n", value, value.count);
}

apply_twice :: (f: (int) -> int, x: int) -> int {
    return f(f(x));
}
