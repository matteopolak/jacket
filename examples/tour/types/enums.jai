// Enums, enum_flags, and switches the compiler checks for you.

Weather :: enum u8 {
    SUNNY;
    CLOUDY;
    RAINY;
    SNOWY;
    THUNDERSTORM :: 10; // Values can be given explicitly.
}

// Each member of an enum_flags gets its own bit: 1, 2, 4, 8...
Gear :: enum_flags {
    UMBRELLA;
    SUNGLASSES;
    SCARF;
    BOOTS;
}

tour_enums :: () {
    // Iterate over every member. Printing an enum prints its name.
    for weather: enum_values_as_enum(Weather) {
        print("%-ish? Pack: %\n", weather, pack_for(weather));
    }

    // Flags combine with `|` and print as a list of names.
    bag := Gear.UMBRELLA | .BOOTS;
    bag |= .SCARF;
    bag &= ~Gear.UMBRELLA;
    bits := formatInt(cast(int) bag, base = 2, minimum_digits = 4);
    print("The bag holds % (bits: %)\n", bag, bits);
    if bag & .BOOTS then print("Boots are in the bag.\n");

    // `.X` is an enum literal; its type comes from context.
    tomorrow: Weather = .SNOWY;
    print("Tomorrow is % (value %); Weather has % members.\n",
        tomorrow, cast(int) tomorrow, enum_names(Weather).count);
}

pack_for :: (weather: Weather) -> Gear {
    // `#complete` makes the compiler insist every member has a case.
    // Delete a case below and the program stops compiling.
    if #complete weather == {
        case .SUNNY;
            return .SUNGLASSES;
        case .CLOUDY;
            #through; // fall into the next case
        case .RAINY;
            return .UMBRELLA | .BOOTS;
        case .SNOWY;
            return .SCARF | .BOOTS;
        case .THUNDERSTORM;
            return 0; // Better stay inside.
    }
    return 0;
}
