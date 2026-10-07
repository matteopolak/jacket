// Any code can run at compile time. The results become constants in the program.

// `#run` calls a procedure while compiling; the answer is baked into the binary.
FIBONACCI :: #run fibonacci_table();

fibonacci_table :: () -> [20] int {
    table: [20] int;
    table[0], table[1] = 0, 1;
    for 2..table.count - 1 table[it] = table[it - 1] + table[it - 2];
    return table;
}

// A sine lookup table, computed once by the compiler instead of at startup.
SINE_STEPS :: 16;

SINE_TABLE :: #run -> [SINE_STEPS] float {
    table: [SINE_STEPS] float;
    for *table it.* = sin(cast(float) it_index * TAU / SINE_STEPS);
    return table;
};

// `#assert` checks a condition while compiling. Break it and the build fails.
#assert(FIBONACCI[10] == 55);
#assert(size_of(Vec2) == 8);

// `#if` keeps or drops code at compile time; the dropped branch is never even type-checked.
#if OS == .WASM {
    PLATFORM_NOTE :: "compiled for WebAssembly: you are probably in a browser";
} else {
    PLATFORM_NOTE :: "compiled for a native target";
}

// `#insert` pastes code into the program. Here a #run builds that code as a
// string: one `Planet` enum member and one gravity entry per line of data.
PLANET_DATA :: #string DONE
MERCURY 3.7
VENUS 8.9
EARTH 9.8
MARS 3.7
JUPITER 24.8
DONE

Planet :: enum {
    #insert #run planet_members();
}

PLANET_GRAVITY :: #insert #run planet_gravity_literal();

planet_members :: () -> string {
    builder: String_Builder;
    for line: split(PLANET_DATA, "\n") {
        if !line continue;
        name := split(line, " ")[0];
        print_to_builder(*builder, "%;\n", name);
    }
    return builder_to_string(*builder);
}

planet_gravity_literal :: () -> string {
    builder: String_Builder;
    append(*builder, "float.[");
    for line: split(PLANET_DATA, "\n") {
        if !line continue;
        print_to_builder(*builder, "%, ", split(line, " ")[1]);
    }
    append(*builder, "]");
    return builder_to_string(*builder);
}

// `#code` captures code as a value without running it, so it can be inserted later.
// Each statement below is pasted into a generated procedure.
CHECKS :: #code {
    print("  inserted code says: 6 * 7 = %\n", 6 * 7);
}

run_checks :: () {
    #insert CHECKS;
}

tour_compile_time :: () {
    print("Fibonacci, computed by the compiler: %\n", FIBONACCI);

    print("A sine wave from a compile-time table:\n");
    for value: SINE_TABLE {
        column := cast(int) ((value + 1) * 20);
        print("  %*\n", pad(column));
    }

    print("%\n", PLATFORM_NOTE);
    print("% planets were generated from a here-string:\n", enum_names(Planet).count);
    for planet: enum_values_as_enum(Planet) {
        gravity := formatFloat(PLANET_GRAVITY[planet], trailing_width = 1);
        print("  % pulls at % m/s²\n", planet, gravity);
    }

    run_checks();
    // Even the compile-time flag is a value you can test.
    print("Is this running at compile time? %. And during #run? %\n",
        #compile_time, #run is_compiling());
}

is_compiling :: () -> bool {
    return #compile_time;
}

pad :: (count: int) -> string {
    spaces := talloc_string(count);
    for 0..count - 1 spaces[it] = #char " ";
    return spaces;
}
