// Compiler: polymorphic instantiation and `#run` work done while compiling.
#import "Basic";

Vec :: struct(N: int, T: Type) {
    values: [N] T;
}

sum :: (v: Vec($N, $T)) -> T {
    total: T;
    for v.values total += it;
    return total;
}

table :: #run -> [256] u32 {
    result: [256] u32;
    for 0..255 {
        c := cast(u32) it;
        for 1..8 c = ifx c & 1 then 0xEDB88320 ^ (c >> 1) else c >> 1;
        result[it] = c;
    }
    return result;
};

#insert #run -> string {
    builder: String_Builder;
    for 1..200 print_to_builder(*builder, "generated_%1 :: (x: int) -> int { return x * %1 + sum(Vec(%2, int).{}); }\n", it, it % 8 + 1);
    return builder_to_string(*builder);
};

main :: () {
    print("% %\n", table[255], generated_200(2));
}
