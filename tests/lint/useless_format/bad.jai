#import "Basic";

label :: (name: string, other: string) -> string {
    a := tprint("%", name);

    b := sprint("%", other);
    defer free(b);

    return tprint("%", a);
}

main :: () {
    print("%\n", label("x", "y"));
}
