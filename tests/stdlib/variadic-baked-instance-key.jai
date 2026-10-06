// Calls of a procedure with a baked variadic parameter (`$args: ..T`) share an instance only when
// their argument lists are equal; lists of the same length used to share one.
#import "Basic";

alpha := 3;
gamma := "text";
A :: #code alpha;
G :: #code gamma;

show :: ($args: ..Code) -> string #expand {
    #insert -> string {
        builder: String_Builder;
        append(*builder, "builder: String_Builder;\n");
        for args print_to_builder(*builder, "print_to_builder(*builder, \"%\", #insert args[%]);\n", "%", it_index);
        append(*builder, "return builder_to_string(*builder);\n");
        return builder_to_string(*builder);
    };
}

names :: ($types: ..Type) -> string {
    builder: String_Builder;
    for types print_to_builder(*builder, "% ", it);
    return builder_to_string(*builder);
}

total :: ($values: ..s64) -> s64 {
    result := 0;
    for values result += it;
    return result;
}

main :: () {
    assert(show(A) == "3");
    assert(show(G) == "text");
    assert(show(A, G) == "3text");
    assert(show(G, A) == "text3");

    assert(names(s64) == "s64 ");
    assert(names(string) == "string ");
    assert(names(s64, string) == "s64 string ");
    assert(total(1, 2) == 3);
    assert(total(5, 7) == 12);

    // Equal lists still share one instance.
    p1 :: #procedure_of_call names(s64, u8);
    p2 :: #procedure_of_call names(s64, u8);
    p3 :: #procedure_of_call names(u8, s64);
    assert(p1 == p2);
    assert(p1 != p3);
    print("ok\n");
}
