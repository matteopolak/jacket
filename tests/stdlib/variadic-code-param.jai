// A baked variadic Code parameter (`$args: ..Code`) quotes each call-site expression, as a
// single `$c: Code` parameter does, so the arguments may use locals.
#import "Basic";

count_codes :: ($args: ..Code) -> s64 #expand {
    return args.count;
}

sum_codes :: ($args: ..Code) -> s64 #expand {
    #insert -> string {
        builder: String_Builder;
        append(*builder, "total := 0;\n");
        for args print_to_builder(*builder, "total += #insert args[%];\n", it_index);
        append(*builder, "return total;\n");
        return builder_to_string(*builder);
    };
}

pass_on :: ($c: Code) -> s64 #expand {
    return sum_codes(c, c);
}

main :: () {
    alpha := 3;
    beta := 10;
    assert(count_codes(alpha) == 1);
    assert(count_codes(alpha, alpha + 1, beta) == 3);
    assert(sum_codes(alpha, alpha + 1) == 7);
    assert(sum_codes(beta * 2, alpha) == 23);
    assert(pass_on(alpha + beta) == 26);
    print("ok\n");
}
