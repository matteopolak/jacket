// Polymorphism: one generic struct and procedure instantiated with many distinct constants and types.
#import "Basic";

Fixed :: struct(N: int, T: Type) {
    values: [N] T;
}

fill :: (f: *Fixed($N, $T), seed: T) {
    for 0..N - 1 f.values[it] = seed + cast(T) it;
}

sum :: (f: Fixed($N, $T)) -> T {
    total: T;
    for f.values total += it;
    return total;
}

scale :: ($factor: int, value: $T) -> T {
    return value * cast(T) factor;
}

instantiate :: () -> string {
    builder: String_Builder;

    for 1..180 {
        print_to_builder(*builder, "{ f: Fixed(%1, s64); fill(*f, %1); total += scale(%1, sum(f)); }\n", it);
        print_to_builder(*builder, "{ g: Fixed(%1, float); fill(*g, 0.5); total += cast(s64) scale(%1, sum(g)); }\n", it % 60 + 1);
    }

    return builder_to_string(*builder);
}

run :: () -> s64 {
    total: s64;
    #insert #run instantiate();
    return total;
}

main :: () {
    print("%\n", run());
}
