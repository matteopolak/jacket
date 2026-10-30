// An array literal of names, where the element type is `Code`, holds the code of each name:
// as a declaration's value, an argument, or a polymorphic struct argument that binds the length.
// rules: macro.14
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

main :: () {
    systems: Systems(.[first, second]);

    assert(systems.count == 2);

    both: [2] Code = .[first, second];
    list := names(both);

    assert(list.count == 2);
    assert(list[0] == "first" && list[1] == "second");
    assert(names(.[second]).count == 1);
    print("ok\n");
}
