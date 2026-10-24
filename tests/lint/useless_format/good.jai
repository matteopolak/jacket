#import "Basic";

label :: (name: string, count: int) -> string {
    a := tprint("%", count);

    b := tprint("[%]", name);

    c := tprint("% %", name, count);

    d := tprint("%%", name);

    return c;
}

shout :: (format: string, value: string) -> string {
    return value;
}

main :: () {
    // Not the library's formatter.
    print("%\n", shout("%", "x"));
    print("%\n", label("x", 1));
}
