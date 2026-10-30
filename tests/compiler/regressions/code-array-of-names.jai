// An array literal of names, where the element type is `Code`, holds the code of each name:
// as a declaration's value, an argument, or a polymorphic struct argument that binds the length.
// rules: macro.17
#import "Basic";
#import "Compiler";

first :: (count: s64, total: *s64) {
    total.* += count;
}

second :: (count: s64, total: *u8) {
    total.* += cast(u8) count;
}

Systems :: struct($sources: [$N] Code) {
    count :: N;
}

names :: (sources: [] Code) -> [..] string {
    result: [..] string;

    for sources {
        ident := cast(*Code_Ident) compiler_get_nodes(it);
        array_add(*result, ident.name);
    }

    return result;
}

// `compiler_get_nodes` exists only at compile time, so the names are read there.
check :: () {
    both: [2] Code = .[first, second];
    list := names(both);

    assert(list.count == 2);
    assert(list[0] == "first" && list[1] == "second");
    assert(names(.[second]).count == 1);
}

#run check();

main :: () {
    systems: Systems(.[first, second]);

    assert(systems.count == 2);
    print("ok\n");
}
