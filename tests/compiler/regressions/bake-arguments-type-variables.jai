// `#bake_arguments` on a parameter whose type declares `$T` keeps `T` for the body and the
// other types, taken from the baked value; a baked procedure may name a `$T` of a parameter left
// open (`names: *[..] string` fixes `T` of `user_data: $T`).
#import "Basic";

identity :: ($v: $T) -> T {
    return v;
}

three :: #bake_arguments identity(v = 3);

pair :: ($a: $T, b: T) -> T {
    return a + b;
}

add_ten :: #bake_arguments pair(a = 10);

walk :: (names: [] string, user_data: $T, proc: (name: string, data: T), skip_hidden := true) {
    for names {
        if skip_hidden && it[0] == #char "." continue;
        proc(it, user_data);
    }
}

collect :: #bake_arguments walk(
    skip_hidden = false,
    proc = (name: string, names: *[..] string) {
        array_add(names, name);
    }
);

main :: () {
    assert(three() == 3, "baked value");
    assert(type_of(three()) == s64, "T is the value's type");
    assert(add_ten(5) == 15, "T names the remaining parameter's type");

    names: [..] string;

    collect(.["a", ".b"], *names);
    assert(names.count == 2, "both names visited");
    assert(names[1] == ".b", "the baked argument was used");
    print("ok\n");
}
