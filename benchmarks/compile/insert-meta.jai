// Metaprogramming: `#insert` of source built at compile time (procedures, structs, enums and a dispatcher).
#import "Basic";

GROUPS :: 150;

generate :: () -> string {
    builder: String_Builder;

    for 0..GROUPS - 1 {
        print_to_builder(*builder, "Record_% :: struct { a: int; b: s32; c: float; }\n", it);
        print_to_builder(*builder, "Kind_% :: enum { first; second; third; }\n", it);
        print_to_builder(*builder, "weigh_% :: (r: Record_%, k: Kind_%) -> int { return r.a * % + r.b + cast(int) r.c + cast(int) k; }\n", it, it, it, it + 1);
    }

    append(*builder, "total :: () -> int {\n    sum := 0;\n");

    for 0..GROUPS - 1 {
        print_to_builder(*builder, "    sum += weigh_%(.{%, %, 1.5}, .second);\n", it, it, it * 2);
    }

    append(*builder, "    return sum;\n}\n");

    return builder_to_string(*builder);
}

#insert #run generate();

main :: () {
    print("%\n", total());
}
