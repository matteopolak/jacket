// A value with no name in its enum prints as `(enum out of range: N)`, whatever the enum is called.
#import "Basic";

Color :: enum {
    RED;
    GREEN;
}

main :: () {
    text := sprint("%", cast(Color) 7);
    assert(text == "(enum out of range: 7)", "got '%'", text);
    print("ok\n");
}
