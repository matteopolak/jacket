// `#bake_arguments` on a parameter whose type declares `$T` keeps `T` for the body and the
// other types, taken from the baked value; a baked procedure may name a `$T` of a parameter left
// open (`names: *[..] string` fixes `T` of `user_data: $T`).
#import "Basic";
#import "File_Utilities";

identity :: ($v: $T) -> T {
    return v;
}

three :: #bake_arguments identity(v = 3);

pair :: ($a: $T, b: T) -> T {
    return a + b;
}

add_ten :: #bake_arguments pair(a = 10);

collect :: #bake_arguments visit_files(
    recursive = false,
    visit_files = true,
    visit_directories = true,
    proc = (info: *File_Visit_Info, names: *[..] string) {
        array_add(names, copy_string(info.short_name));
    }
);

main :: () {
    assert(three() == 3, "baked value");
    assert(type_of(three()) == s64, "T is the value's type");
    assert(add_ten(5) == 15, "T names the remaining parameter's type");

    names: [..] string;

    collect(#filepath, *names);
    assert(names.count > 0, "no entries visited");
    found := false;
    for names if it == "bake-arguments-type-variables.jai" found = true;

    assert(found, "this file was not visited");
    print("ok\n");
}
